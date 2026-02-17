use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use shared::{DownloadConfig, TaskStatus, UploadConfig};
use uuid::Uuid;

use crate::{recording, state::SharedState};

pub async fn list_downloads(State(state): State<SharedState>) -> Json<Vec<DownloadConfig>> {
    match state.db.get_downloads().await {
        Ok(mut downloads) => {
            for d in &mut downloads {
                d.current_status = Some(resolve_download_status(&state, &d.url));
            }
            Json(downloads)
        }
        Err(e) => {
            tracing::error!("Failed to list downloads: {}", e);
            Json(vec![])
        }
    }
}

pub async fn add_download(
    State(state): State<SharedState>,
    Json(payload): Json<DownloadConfig>,
) -> StatusCode {
    let mut config = payload;
    if config.id.is_empty() {
        config.id = Uuid::new_v4().to_string();
    }
    if let Err(e) = state.db.save_download(&config).await {
        tracing::error!("Failed to save download: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

pub async fn trigger_manual_upload(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> (StatusCode, String) {
    tracing::info!("Manual upload request received, download_id={}", id);

    let downloads = match state.db.get_downloads().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to load downloads: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to load downloads".to_string());
        }
    };

    let Some(download) = downloads.into_iter().find(|d| d.id == id) else {
        tracing::warn!("Manual upload rejected: download config not found, id={}", id);
        return (StatusCode::NOT_FOUND, "download config not found".to_string());
    };

    tracing::info!(
        "Manual upload target resolved: id={}, name={}, url={}",
        download.id,
        download.name,
        download.url
    );

    if download.linked_upload_ids.is_empty() {
        tracing::warn!("Manual upload rejected: no linked upload templates, id={}", download.id);
        return (StatusCode::BAD_REQUEST, "no linked upload templates".to_string());
    }

    let uploads = match state.db.get_uploads().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to load uploads: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to load uploads".to_string());
        }
    };

    let upload_configs: Vec<UploadConfig> = download
        .linked_upload_ids
        .iter()
        .filter_map(|uid| uploads.iter().find(|u| &u.id == uid))
        .map(|u| u.config.clone())
        .collect();

    if upload_configs.is_empty() {
        tracing::warn!(
            "Manual upload rejected: linked templates missing in DB, id={}",
            download.id
        );
        return (StatusCode::BAD_REQUEST, "linked upload templates are missing".to_string());
    }

    let task_dir = recording::recording_task_dir(&download.name);
    let mut files = Vec::new();

    let mut entries = match tokio::fs::read_dir(&task_dir).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to open recording dir {}: {}", task_dir.display(), e);
            return (
                StatusCode::NOT_FOUND,
                format!("recording directory not found: {}", task_dir.display()),
            );
        }
    };

    loop {
        match entries.next_entry().await {
            Ok(Some(entry)) => {
                let path = entry.path();
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_ascii_lowercase())
                    .unwrap_or_default();
                let allow = matches!(ext.as_str(), "mp4" | "flv" | "mkv" | "ts");
                if !allow {
                    continue;
                }
                if let Ok(meta) = entry.metadata().await
                    && meta.is_file()
                {
                    files.push(path.to_string_lossy().to_string());
                }
            }
            Ok(None) => break,
            Err(e) => {
                tracing::error!("Failed to read recording dir {}: {}", task_dir.display(), e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to scan recording files".to_string(),
                );
            }
        }
    }

    files.sort();

    tracing::info!(
        "Manual upload file scan done: task={}, dir={}, file_count={}",
        download.name,
        task_dir.display(),
        files.len()
    );

    if files.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            format!("no recording files found in {}", task_dir.display()),
        );
    }

    let live_title = state.checker.fetch_live_title(&download.url).await;
    let task_name = download.name.clone();
    let auto_cleanup_after_upload = if download.use_custom_recording_settings {
        download.recording_settings.as_ref().map(|s| s.auto_cleanup_after_upload).unwrap_or(false)
    } else {
        state.recording_settings.read().await.auto_cleanup_after_upload
    };

    let manual_task_id = format!("manual-upload-{}", Uuid::new_v4());
    tracing::info!(
        "Manual upload accepted: manual_task_id={}, task_name={}, files={}, upload_configs={}",
        manual_task_id,
        task_name,
        files.len(),
        upload_configs.len()
    );

    let state_for_upload = state.clone();
    tokio::spawn(async move {
        recording::run_upload(
            manual_task_id,
            files,
            state_for_upload,
            false,
            upload_configs,
            live_title,
            task_name,
            auto_cleanup_after_upload,
        )
        .await;
    });

    (StatusCode::ACCEPTED, "manual upload started".to_string())
}

fn resolve_download_status(state: &SharedState, url: &str) -> String {
    let mut has_error = false;
    let mut has_completed = false;

    for task in state.tasks.iter() {
        if task.value().url != url {
            continue;
        }
        match &task.value().status {
            TaskStatus::Recording => return "下载中".to_string(),
            TaskStatus::Uploading => return "上传中".to_string(),
            TaskStatus::Error(_) => has_error = true,
            TaskStatus::Completed => has_completed = true,
            TaskStatus::Idle => {}
        }
    }

    if state.checking_urls.contains_key(url) {
        return "检测中".to_string();
    }
    if has_error {
        return "失败".to_string();
    }
    if has_completed {
        return "已完成".to_string();
    }
    "空闲".to_string()
}

pub async fn delete_download(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> StatusCode {
    if let Err(e) = state.db.delete_download(&id).await {
        tracing::error!("Failed to delete download: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}
