use chrono::Local;
use shared::{TaskStatus, UploadConfig};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

use crate::{
    checker::STREAMLINK_PATH,
    state::{RecorderHandle, SharedState},
    uploader::UploadTarget,
};

pub async fn run_upload(
    task_id: String,
    filenames: Vec<String>,
    state: SharedState,
    update_status: bool,
    configs: Vec<UploadConfig>,
    live_title: Option<String>,
    task_name: String,
    auto_cleanup_after_upload: bool,
) {
    if filenames.is_empty() {
        tracing::warn!("Task {} has no files to upload", task_id);
        return;
    }
    if configs.is_empty() {
        tracing::info!("Task {} has no upload configs, skipping upload", task_id);
        if update_status {
            if let Some(mut task) = state.tasks.get_mut(&task_id) {
                task.status = TaskStatus::Completed;
            }
            let _ = state.db.update_status(&task_id, &TaskStatus::Completed).await;
            state.handles.remove(&task_id);
        }
        return;
    }

    tracing::info!(
        "Task {} start uploading files: {:?} with {} configs",
        task_id,
        filenames,
        configs.len()
    );

    if update_status && let Some(mut task) = state.tasks.get_mut(&task_id) {
        task.status = TaskStatus::Uploading;
    }

    let target = UploadTarget::Bilibili;
    let uploader = target.create_uploader();

    let mut all_success = true;
    let mut error_msg = String::new();

    for (i, config) in configs.iter().enumerate() {
        tracing::info!(
            "Task {} uploading config {}/{}: {:?}",
            task_id,
            i + 1,
            configs.len(),
            config.title
        );
        match uploader.upload(filenames.clone(), config, live_title.as_deref(), &task_name).await {
            Ok(_) => tracing::info!("Task {} upload {} completed", task_id, i + 1),
            Err(e) => {
                tracing::error!("Task {} upload {} failed: {:?}", task_id, i + 1, e);
                all_success = false;
                error_msg = format!("Upload {} failed: {:?}", i + 1, e);
            }
        }
    }

    let final_status =
        if all_success { TaskStatus::Completed } else { TaskStatus::Error(error_msg) };

    if all_success && auto_cleanup_after_upload {
        let mut uniq = HashSet::new();
        for file in filenames {
            if !uniq.insert(file.clone()) {
                continue;
            }
            if let Err(e) = tokio::fs::remove_file(&file).await {
                tracing::warn!("Task {} cleanup failed for {}: {}", task_id, file, e);
            } else {
                tracing::info!("Task {} cleaned up local file: {}", task_id, file);
            }
        }
    }

    if update_status {
        if let Some(mut task) = state.tasks.get_mut(&task_id) {
            task.status = final_status.clone();
        }
        let _ = state.db.update_status(&task_id, &final_status).await;
    }

    if update_status {
        state.handles.remove(&task_id);
    }
}

pub async fn spawn_recorder(
    task_id: String,
    url: String,
    _initial_filename: String,
    state: SharedState,
    custom_recording_settings: Option<shared::RecordingSettings>,
) {
    let task_id_clone = task_id.clone();
    let state_for_task = state.clone();

    let effective_settings = if let Some(v) = custom_recording_settings {
        v
    } else {
        state.recording_settings.read().await.clone()
    };
    let (segment_size_bytes, segment_time_sec, quality, auto_cleanup_after_upload) = {
        let config = &effective_settings;
        (
            config.segment_size_mb.and_then(|mb| mb.checked_mul(1024 * 1024)).filter(|v| *v > 0),
            config.segment_time_sec.filter(|v| *v > 0),
            quality_for_url(&url, &config.quality),
            config.auto_cleanup_after_upload,
        )
    };

    let upload_configs = if let Some(task) = state.tasks.get(&task_id) {
        task.upload_configs.clone()
    } else {
        tracing::warn!("Task {} not found when spawning recorder", task_id);
        return;
    };

    let handle = tokio::spawn(async move {
        tracing::info!("Task {} preparing to record: {}", task_id, url);

        if let Some(mut task) = state_for_task.tasks.get_mut(&task_id) {
            task.status = TaskStatus::Recording;
        }
        let _ = state_for_task.db.update_status(&task_id, &TaskStatus::Recording).await;

        let mut recorded_files = Vec::new();
        let mut live_title = state_for_task.checker.fetch_live_title(&url).await;
        let mut consecutive_empty_segments = 0u8;

        loop {
            let task_name = if let Some(task) = state_for_task.tasks.get(&task_id) {
                task.name.clone()
            } else {
                task_id.clone()
            };
            let task_dir = recording_task_dir(&task_name);
            if let Err(e) = tokio::fs::create_dir_all(&task_dir).await {
                tracing::error!(
                    "Task {} failed to create recording directory {}: {}",
                    task_id,
                    task_dir.display(),
                    e
                );
                break;
            }

            let now = Local::now();
            let basename = format!(
                "{}-{}.mp4",
                sanitize_for_filename(&task_name),
                now.format("%Y%m%d_%H%M%S")
            );
            let current_filename = task_dir.join(basename).to_string_lossy().to_string();

            if let Some(mut task) = state_for_task.tasks.get_mut(&task_id) {
                task.filename = current_filename.clone();
            }

            tracing::info!("Task {} starting segment: {}", task_id, current_filename);

            let mut command = Command::new(STREAMLINK_PATH);
            command.arg("-o").arg(&current_filename).arg(&url).arg(&quality);
            command.kill_on_drop(true);

            let mut child = command.spawn().expect("failed to spawn streamlink");
            let mut segment_limit_reached = false;

            loop {
                tokio::select! {
                    status = child.wait() => {
                        match status {
                            Ok(s) => tracing::info!("Task {} segment finished with status: {}", task_id, s),
                            Err(e) => tracing::error!("Task {} segment error: {}", task_id, e),
                        }
                        break;
                    }
                    _ = sleep(Duration::from_secs(segment_time_sec.unwrap_or(u64::MAX))), if segment_time_sec.is_some() => {
                        tracing::info!("Task {} segment time limit reached", task_id);
                        segment_limit_reached = true;
                        let _ = child.kill().await;
                        break;
                    }
                    _ = sleep(Duration::from_secs(5)) => {
                        if let Some(limit) = segment_size_bytes
                            && let Ok(meta) = tokio::fs::metadata(&current_filename).await
                            && meta.len() > limit
                        {
                            tracing::info!("Task {} segment size limit reached: {} > {}", task_id, meta.len(), limit);
                            segment_limit_reached = true;
                            let _ = child.kill().await;
                            break;
                        }
                    }
                }
            }

            if std::path::Path::new(&current_filename).exists() {
                recorded_files.push(current_filename);
                consecutive_empty_segments = 0;
            } else {
                consecutive_empty_segments = consecutive_empty_segments.saturating_add(1);
                tracing::warn!(
                    "Task {} segment produced no file (consecutive: {})",
                    task_id,
                    consecutive_empty_segments
                );
            }

            if segment_limit_reached {
                continue;
            }

            if consecutive_empty_segments >= 3 {
                tracing::error!(
                    "Task {} stopped after {} empty segments: stream appears unavailable",
                    task_id,
                    consecutive_empty_segments
                );
                break;
            }

            match state_for_task.checker.check_live(&url).await {
                Ok(true) => {
                    tracing::warn!(
                        "Task {} streamlink exited but stream is still live, restarting...",
                        task_id
                    );
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
                _ => {
                    tracing::info!(
                        "Task {} stream ended or check failed, stopping recorder",
                        task_id
                    );
                    break;
                }
            }
        }

        if !recorded_files.is_empty() {
            if live_title.is_none() {
                live_title = state_for_task.checker.fetch_live_title(&url).await;
            }
            let final_task_name = if let Some(task) = state_for_task.tasks.get(&task_id) {
                task.name.clone()
            } else {
                task_id.clone()
            };
            run_upload(
                task_id.clone(),
                recorded_files,
                state_for_task.clone(),
                true,
                upload_configs,
                live_title,
                final_task_name,
                auto_cleanup_after_upload,
            )
            .await;
        } else {
            if let Some(mut task) = state_for_task.tasks.get_mut(&task_id) {
                task.status = TaskStatus::Error("No files generated".to_string());
            }
            let _ = state_for_task
                .db
                .update_status(&task_id, &TaskStatus::Error("No files generated".to_string()))
                .await;

            state_for_task.handles.remove(&task_id);
        }
    });

    state.handles.insert(task_id_clone, RecorderHandle { abort_handle: handle.abort_handle() });
}

fn quality_for_url(url: &str, quality: &shared::PlatformQualityConfig) -> String {
    let u = url.to_ascii_lowercase();
    if u.contains("bilibili.com") || u.contains("b23.tv") {
        return quality.bilibili.clone();
    }
    if u.contains("douyu.com") {
        return quality.douyu.clone();
    }
    if u.contains("huya.com") {
        return quality.huya.clone();
    }
    if u.contains("twitch.tv") {
        return quality.twitch.clone();
    }
    if u.contains("youtube.com") || u.contains("youtu.be") {
        return quality.youtube.clone();
    }
    quality.default_quality.clone()
}

fn recording_root_dir() -> PathBuf {
    if let Ok(v) = std::env::var("BILIUP_RECORDINGS_DIR") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    PathBuf::from("data/recordings")
}

pub(crate) fn recording_task_dir(task_name: &str) -> PathBuf {
    recording_root_dir().join(sanitize_for_filename(task_name))
}

fn sanitize_for_filename(raw: &str) -> String {
    let mut out = raw
        .chars()
        .map(|c| {
            let illegal =
                c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|');
            if !illegal { c } else { '_' }
        })
        .collect::<String>();
    out = out.trim_matches('_').to_string();
    if out.is_empty() { "task".to_string() } else { out }
}
