mod runtime;
mod segment;
mod task_state;

use chrono::Local;
use shared::{TaskStatus, UploadConfig};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use self::runtime::{RecorderRuntimeConfig, build_runtime_config};
use self::segment::{
    SegmentLoopAction, decide_next_segment_action, record_segment, update_recorded_files,
};
use self::task_state::{
    clear_task_handle, finish_recording_without_files, resolve_task_name, set_task_filename,
    set_task_status,
};
use crate::{
    state::{RecorderHandle, SharedState},
    storage_guard::recording_storage_below_min_free_percent,
    uploader::UploadTarget,
};

const MAX_BATCH_UPLOAD_ATTEMPTS: usize = 3;
const RETRY_BACKOFF_SECS: &[u64] = &[30, 120];

#[derive(Debug, Clone, Copy)]
pub struct UploadRunOptions {
    pub auto_cleanup_after_upload: bool,
    pub min_upload_file_size_bytes: u64,
}

async fn prepare_segment_file(
    state: &SharedState,
    task_id: &str,
) -> Result<String, std::io::Error> {
    let task_name = resolve_task_name(state, task_id);
    let task_dir = recording_task_dir(&task_name);
    tokio::fs::create_dir_all(&task_dir).await?;

    let basename = format!(
        "{}-{}.mp4",
        sanitize_for_filename(&task_name),
        Local::now().format("%Y%m%d_%H%M%S")
    );
    let current_filename = task_dir.join(basename).to_string_lossy().to_string();
    set_task_filename(state, task_id, &current_filename).await;
    Ok(current_filename)
}

async fn stop_segment_process(child: &mut tokio::process::Child, task_id: &str, reason: &str) {
    if let Err(e) = child.kill().await {
        tracing::warn!("Task {} failed to kill recorder after {}: {}", task_id, reason, e);
    }
}

pub async fn run_upload(
    task_id: String,
    filenames: Vec<String>,
    state: SharedState,
    update_status: bool,
    configs: Vec<UploadConfig>,
    live_title: Option<String>,
    task_name: String,
    options: UploadRunOptions,
) {
    if filenames.is_empty() {
        let message = format!("Task {task_id} has no files to upload");
        tracing::error!("{}", message);
        if update_status {
            set_task_status(&state, &task_id, TaskStatus::Error(message)).await;
            clear_task_handle(&state, &task_id);
        }
        return;
    }

    if configs.is_empty() {
        tracing::info!("Task {} has no upload configs, skipping upload", task_id);
        if update_status {
            set_task_status(&state, &task_id, TaskStatus::Completed).await;
            clear_task_handle(&state, &task_id);
        }
        return;
    }

    let filenames =
        prepare_upload_files(&task_id, filenames, options.min_upload_file_size_bytes).await;
    if filenames.is_empty() {
        tracing::warn!(
            "Task {} has no files left after small-file cleanup, marking completed",
            task_id
        );
        if update_status {
            set_task_status(&state, &task_id, TaskStatus::Completed).await;
            clear_task_handle(&state, &task_id);
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
    for (config_index, config) in configs.iter().enumerate() {
        tracing::info!(
            "Task {} uploading {} files as one multi-part archive with config {}/{}: title_template={:?}",
            task_id,
            filenames.len(),
            config_index + 1,
            configs.len(),
            config.title
        );

        if let Err(e) = upload_batch_with_retry(
            uploader.as_ref(),
            &task_id,
            &filenames,
            config,
            live_title.as_deref(),
            &task_name,
            config_index,
            configs.len(),
        )
        .await
        {
            tracing::error!(
                "Task {} multi-part upload failed for config {}/{}: task_name={}, files={:?}, title_template={:?}: {:?}",
                task_id,
                config_index + 1,
                configs.len(),
                task_name,
                filenames,
                config.title,
                e
            );
            all_success = false;
            error_msg = format!(
                "Multi-part upload config {}/{} failed: {:?}",
                config_index + 1,
                configs.len(),
                e
            );
            break;
        }
    }

    let final_status = if all_success {
        if options.auto_cleanup_after_upload {
            cleanup_uploaded_files(&task_id, &filenames).await;
        }
        TaskStatus::Completed
    } else {
        TaskStatus::Error(error_msg)
    };

    if update_status {
        set_task_status(&state, &task_id, final_status).await;
        clear_task_handle(&state, &task_id);
    }
}

async fn upload_batch_with_retry(
    uploader: &dyn crate::uploader::Uploader,
    task_id: &str,
    filenames: &[String],
    config: &UploadConfig,
    live_title: Option<&str>,
    task_name: &str,
    config_index: usize,
    config_count: usize,
) -> anyhow::Result<()> {
    let mut last_error = None;

    for attempt in 1..=MAX_BATCH_UPLOAD_ATTEMPTS {
        match uploader.upload(filenames.to_vec(), config, live_title, task_name).await {
            Ok(()) => {
                tracing::info!(
                    "Task {} submitted multi-part upload successfully: file_count={}, config={}/{}, attempt={}/{}",
                    task_id,
                    filenames.len(),
                    config_index + 1,
                    config_count,
                    attempt,
                    MAX_BATCH_UPLOAD_ATTEMPTS
                );
                return Ok(());
            }
            Err(e) => {
                let message = format!("{e:?}");
                let transient = is_transient_upload_error(&message);
                tracing::warn!(
                    "Task {} multi-part upload attempt failed: file_count={}, config={}/{}, attempt={}/{}, transient={}, error={}",
                    task_id,
                    filenames.len(),
                    config_index + 1,
                    config_count,
                    attempt,
                    MAX_BATCH_UPLOAD_ATTEMPTS,
                    transient,
                    message
                );
                last_error = Some(e);

                if !transient || attempt == MAX_BATCH_UPLOAD_ATTEMPTS {
                    break;
                }

                let backoff = RETRY_BACKOFF_SECS
                    .get(attempt.saturating_sub(1))
                    .copied()
                    .unwrap_or_else(|| *RETRY_BACKOFF_SECS.last().unwrap_or(&120));
                tracing::warn!(
                    "Task {} will retry multi-part upload after {}s: file_count={}, config={}/{}, next_attempt={}",
                    task_id,
                    backoff,
                    filenames.len(),
                    config_index + 1,
                    config_count,
                    attempt + 1
                );
                sleep(Duration::from_secs(backoff)).await;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("upload failed without error detail")))
}

async fn cleanup_uploaded_files(task_id: &str, filenames: &[String]) {
    let mut unique_files = HashSet::new();
    for file in filenames {
        if !unique_files.insert(file) {
            continue;
        }
        if let Err(e) = tokio::fs::remove_file(file).await {
            tracing::warn!("Task {} cleanup failed for {}: {}", task_id, file, e);
        } else {
            tracing::info!(
                "Task {} cleaned up local file after multi-part submission: {}",
                task_id,
                file
            );
        }
    }
}

fn is_transient_upload_error(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    [
        "500",
        "502",
        "503",
        "504",
        "gateway time-out",
        "gateway timeout",
        "timeout",
        "timed out",
        "error sending request",
        "connection reset",
        "connection closed",
        "connection refused",
        "connection aborted",
        "temporarily unavailable",
        "network",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}

async fn prepare_upload_files(
    task_id: &str,
    filenames: Vec<String>,
    min_upload_file_size_bytes: u64,
) -> Vec<String> {
    if min_upload_file_size_bytes == 0 {
        return filenames;
    }

    let mut kept = Vec::new();
    let mut seen = HashSet::new();
    for file in filenames {
        if !seen.insert(file.clone()) {
            continue;
        }

        match tokio::fs::metadata(&file).await {
            Ok(meta) if meta.is_file() && meta.len() < min_upload_file_size_bytes => {
                match tokio::fs::remove_file(&file).await {
                    Ok(()) => tracing::warn!(
                        "Task {} deleted small recording before upload: file={}, size={} bytes, threshold={} bytes",
                        task_id,
                        file,
                        meta.len(),
                        min_upload_file_size_bytes
                    ),
                    Err(e) => tracing::warn!(
                        "Task {} failed to delete small recording before upload, skipping upload for file={}, size={} bytes, threshold={} bytes: {}",
                        task_id,
                        file,
                        meta.len(),
                        min_upload_file_size_bytes,
                        e
                    ),
                }
            }
            Ok(meta) if meta.is_file() => kept.push(file),
            Ok(_) => tracing::warn!("Task {} skipped non-file upload path: {}", task_id, file),
            Err(e) => {
                tracing::warn!(
                    "Task {} failed to inspect upload file {}, keeping it for uploader validation: {}",
                    task_id,
                    file,
                    e
                );
                kept.push(file);
            }
        }
    }

    kept
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
    let runtime = build_runtime_config(&url, &effective_settings);

    let upload_configs = if let Some(task) = state.tasks.get(&task_id) {
        task.upload_configs.clone()
    } else {
        tracing::warn!("Task {} not found when spawning recorder", task_id);
        return;
    };

    let handle = tokio::spawn(async move {
        tracing::info!("Task {} preparing to record: {}", task_id, url);

        set_task_status(&state_for_task, &task_id, TaskStatus::Recording).await;

        let mut recorded_files = Vec::new();
        let mut live_title = state_for_task.checker.fetch_live_title(&url).await;
        let mut consecutive_empty_segments = 0u8;
        let mut terminal_error: Option<String> = None;

        loop {
            match recording_storage_below_min_free_percent().await {
                Ok(Some(snapshot)) => {
                    tracing::warn!(
                        "Task {} skipped starting next segment because recording storage is below 2% free: path={}, available_kb={}, total_kb={}, free_percent={:.2}; recorded_files={} will be uploaded if available",
                        task_id,
                        snapshot.path.display(),
                        snapshot.available_kb,
                        snapshot.total_kb,
                        snapshot.free_percent,
                        recorded_files.len()
                    );
                    break;
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(
                        "Task {} could not check recording storage before next segment: {}",
                        task_id,
                        e
                    );
                }
            }

            let result = record_segment(&task_id, &url, &state_for_task, &runtime).await;

            if !result.filename.is_empty() {
                update_recorded_files(
                    &task_id,
                    result.filename,
                    &mut recorded_files,
                    &mut consecutive_empty_segments,
                );
            }

            if result.disk_full {
                terminal_error = result
                    .terminal_error
                    .or_else(|| Some("Recording stopped because disk storage is full".to_string()));
                tracing::warn!(
                    "Task {} stopped recording because disk storage is full; recorded_files={} will be uploaded if available",
                    task_id,
                    recorded_files.len()
                );
                break;
            }

            if result.storage_guard_triggered {
                tracing::warn!(
                    "Task {} stopped recording because recording storage is below 2% free; recorded_files={} will be uploaded if available",
                    task_id,
                    recorded_files.len()
                );
                break;
            }

            if let Some(message) = result.terminal_error {
                terminal_error = Some(message);
                break;
            }

            if result.limit_reached {
                continue;
            }

            match decide_next_segment_action(
                &state_for_task,
                &task_id,
                &url,
                consecutive_empty_segments,
            )
            .await
            {
                SegmentLoopAction::Continue => continue,
                SegmentLoopAction::Stop => break,
            }
        }

        if !recorded_files.is_empty() {
            let refreshed_title = state_for_task.checker.fetch_live_title(&url).await;
            if refreshed_title.is_some() {
                live_title = refreshed_title;
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
                UploadRunOptions {
                    auto_cleanup_after_upload: runtime.auto_cleanup_after_upload,
                    min_upload_file_size_bytes: runtime.min_upload_file_size_bytes,
                },
            )
            .await;
        } else {
            finish_recording_without_files(&state_for_task, &task_id, terminal_error).await;
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
    if u.contains("tiktok.com") {
        return quality.tiktok.clone();
    }
    if u.contains("douyin.com") {
        return quality.douyin.clone();
    }
    if u.contains("twitch.tv") {
        return quality.twitch.clone();
    }
    if u.contains("youtube.com") || u.contains("youtu.be") {
        return quality.youtube.clone();
    }
    if u.contains("kick.com") {
        return quality.kick.clone();
    }
    quality.default_quality.clone()
}

pub(crate) fn recording_root_dir() -> PathBuf {
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

#[cfg(test)]
mod tests {
    use super::{is_transient_upload_error, prepare_upload_files, quality_for_url};
    use shared::PlatformQualityConfig;
    use uuid::Uuid;

    #[test]
    fn quality_for_url_supports_requested_platforms() {
        let quality = PlatformQualityConfig {
            bilibili: "bili".to_string(),
            douyu: "douyu".to_string(),
            huya: "huya".to_string(),
            tiktok: "tiktok".to_string(),
            douyin: "douyin".to_string(),
            twitch: "twitch".to_string(),
            youtube: "youtube".to_string(),
            kick: "kick".to_string(),
            default_quality: "default".to_string(),
        };

        assert_eq!(quality_for_url("https://live.bilibili.com/6", &quality), "bili");
        assert_eq!(quality_for_url("https://www.douyu.com/74960", &quality), "douyu");
        assert_eq!(
            quality_for_url("https://www.tiktok.com/@diemhuynh_2003/live", &quality),
            "tiktok"
        );
        assert_eq!(quality_for_url("https://live.douyin.com/393646574978", &quality), "douyin");
        assert_eq!(quality_for_url("https://www.twitch.tv/seucreysonreborn", &quality), "twitch");
        assert_eq!(quality_for_url("https://kick.com/topson", &quality), "kick");
    }

    #[tokio::test]
    async fn prepare_upload_files_deletes_files_smaller_than_threshold() {
        let dir = std::env::temp_dir().join(format!("omnistream-upload-filter-{}", Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.expect("create temp dir");
        let small = dir.join("small.mp4");
        let large = dir.join("large.mp4");
        tokio::fs::write(&small, vec![1_u8; 4]).await.expect("write small file");
        tokio::fs::write(&large, vec![1_u8; 8]).await.expect("write large file");

        let files = prepare_upload_files(
            "test-task",
            vec![small.to_string_lossy().to_string(), large.to_string_lossy().to_string()],
            5,
        )
        .await;

        assert_eq!(files, vec![large.to_string_lossy().to_string()]);
        assert!(!small.exists());
        assert!(large.exists());

        tokio::fs::remove_dir_all(&dir).await.expect("cleanup temp dir");
    }

    #[tokio::test]
    async fn prepare_upload_files_keeps_all_files_when_threshold_disabled() {
        let dir =
            std::env::temp_dir().join(format!("omnistream-upload-filter-off-{}", Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.expect("create temp dir");
        let small = dir.join("small.mp4");
        tokio::fs::write(&small, vec![1_u8; 1]).await.expect("write small file");

        let files =
            prepare_upload_files("test-task", vec![small.to_string_lossy().to_string()], 0).await;

        assert_eq!(files, vec![small.to_string_lossy().to_string()]);
        assert!(small.exists());

        tokio::fs::remove_dir_all(&dir).await.expect("cleanup temp dir");
    }

    #[test]
    fn transient_upload_error_detects_temporary_bilibili_failures() {
        assert!(is_transient_upload_error("HTTP status server error (504 Gateway Time-out)"));
        assert!(is_transient_upload_error(
            "error sending request for url (https://upos-sz-upcdnbda2.bilivideo.com)"
        ));
        assert!(is_transient_upload_error("connection reset by peer"));
    }

    #[test]
    fn transient_upload_error_ignores_permanent_config_failures() {
        assert!(!is_transient_upload_error("account_file is required"));
        assert!(!is_transient_upload_error("copyright must be 1 (Original) or 2 (Reprint)"));
    }
}
