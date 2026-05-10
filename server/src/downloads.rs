use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use shared::{DownloadConfig, TaskStatus};
use uuid::Uuid;

use crate::{
    downloads_service::{
        ScanRecordingFilesError, load_download_for_manual_upload, resolve_auto_cleanup_after_upload,
        resolve_manual_upload_configs, scan_recording_files,
    },
    recording,
    state::SharedState,
};

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

    let download = match load_download_for_manual_upload(&state, &id).await {
        Ok(download) => download,
        Err(response) => return response,
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

    let upload_configs = match resolve_manual_upload_configs(&state, &download).await {
        Ok(configs) => configs,
        Err(response) => return response,
    };

    if upload_configs.is_empty() {
        tracing::warn!(
            "Manual upload rejected: linked templates missing in DB, id={}",
            download.id
        );
        return (StatusCode::BAD_REQUEST, "linked upload templates are missing".to_string());
    }

    let task_dir = recording::recording_task_dir(&download.name);
    let files = match scan_recording_files(&task_dir).await {
        Ok(files) => files,
        Err(ScanRecordingFilesError::NotFound(message)) => {
            tracing::error!("Failed to open recording dir {}: {}", task_dir.display(), message);
            return (
                StatusCode::NOT_FOUND,
                format!("recording directory not found: {}", task_dir.display()),
            );
        }
        Err(ScanRecordingFilesError::ReadFailed(message)) => {
            tracing::error!("Failed to read recording dir {}: {}", task_dir.display(), message);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to scan recording files".to_string(),
            );
        }
    };

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
    let auto_cleanup_after_upload =
        resolve_auto_cleanup_after_upload(&state, &download).await;

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
    let mut statuses = Vec::new();
    let is_checking = state.checking_urls.contains_key(url);

    for task in state.tasks.iter() {
        if task.value().url != url {
            continue;
        }
        statuses.push(task.value().status.clone());
    }

    status_label_for_tasks(&statuses, is_checking)
}

fn status_label_for_tasks(statuses: &[TaskStatus], is_checking: bool) -> String {
    let mut has_error = false;
    let mut has_completed = false;
    let mut has_stopped = false;

    for status in statuses {
        match status {
            TaskStatus::Recording => return "下载中".to_string(),
            TaskStatus::Uploading => return "上传中".to_string(),
            TaskStatus::Error(_) => has_error = true,
            TaskStatus::Completed => has_completed = true,
            TaskStatus::Stopped => has_stopped = true,
            TaskStatus::Idle => {}
        }
    }

    if is_checking {
        return "检测中".to_string();
    }
    if has_error {
        return "失败".to_string();
    }
    if has_completed {
        return "已完成".to_string();
    }
    if has_stopped {
        return "已停止".to_string();
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

#[cfg(test)]
mod tests {
    use super::status_label_for_tasks;
    use shared::TaskStatus;

    #[test]
    fn status_label_prioritizes_active_states() {
        let statuses = vec![TaskStatus::Completed, TaskStatus::Recording];
        assert_eq!(status_label_for_tasks(&statuses, false), "下载中");

        let statuses = vec![TaskStatus::Stopped, TaskStatus::Uploading];
        assert_eq!(status_label_for_tasks(&statuses, false), "上传中");
    }

    #[test]
    fn status_label_maps_terminal_states_and_checking() {
        assert_eq!(status_label_for_tasks(&[], true), "检测中");
        assert_eq!(status_label_for_tasks(&[TaskStatus::Error("x".to_string())], false), "失败");
        assert_eq!(status_label_for_tasks(&[TaskStatus::Completed], false), "已完成");
        assert_eq!(status_label_for_tasks(&[TaskStatus::Stopped], false), "已停止");
        assert_eq!(status_label_for_tasks(&[TaskStatus::Idle], false), "空闲");
    }
}
