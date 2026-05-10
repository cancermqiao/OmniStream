use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use shared::{DownloadConfig, TaskStatus, UploadConfig, UploadTemplate};
use std::path::Path as FsPath;
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

async fn load_download_for_manual_upload(
    state: &SharedState,
    id: &str,
) -> Result<DownloadConfig, (StatusCode, String)> {
    let downloads = state.db.get_downloads().await.map_err(|e| {
        tracing::error!("Failed to load downloads: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load downloads".to_string())
    })?;

    downloads.into_iter().find(|d| d.id == id).ok_or_else(|| {
        tracing::warn!("Manual upload rejected: download config not found, id={}", id);
        (StatusCode::NOT_FOUND, "download config not found".to_string())
    })
}

async fn resolve_manual_upload_configs(
    state: &SharedState,
    download: &DownloadConfig,
) -> Result<Vec<UploadConfig>, (StatusCode, String)> {
    let uploads = state.db.get_uploads().await.map_err(|e| {
        tracing::error!("Failed to load uploads: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load uploads".to_string())
    })?;

    Ok(select_upload_configs(&download.linked_upload_ids, &uploads))
}

fn select_upload_configs(
    linked_upload_ids: &[String],
    uploads: &[UploadTemplate],
) -> Vec<UploadConfig> {
    linked_upload_ids
        .iter()
        .filter_map(|uid| uploads.iter().find(|u| &u.id == uid))
        .map(|u| u.config.clone())
        .collect()
}

async fn resolve_auto_cleanup_after_upload(state: &SharedState, download: &DownloadConfig) -> bool {
    if download.use_custom_recording_settings {
        download.recording_settings.as_ref().map(|s| s.auto_cleanup_after_upload).unwrap_or(false)
    } else {
        state.recording_settings.read().await.auto_cleanup_after_upload
    }
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

#[derive(Debug)]
enum ScanRecordingFilesError {
    NotFound(String),
    ReadFailed(String),
}

fn is_recording_file(path: &FsPath) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(ext.as_str(), "mp4" | "flv" | "mkv" | "ts")
}

async fn scan_recording_files(task_dir: &FsPath) -> Result<Vec<String>, ScanRecordingFilesError> {
    let mut files = Vec::new();
    let mut entries = tokio::fs::read_dir(task_dir)
        .await
        .map_err(|e| ScanRecordingFilesError::NotFound(e.to_string()))?;

    loop {
        match entries.next_entry().await {
            Ok(Some(entry)) => {
                let path = entry.path();
                if !is_recording_file(&path) {
                    continue;
                }
                if let Ok(meta) = entry.metadata().await
                    && meta.is_file()
                {
                    files.push(path.to_string_lossy().to_string());
                }
            }
            Ok(None) => break,
            Err(e) => return Err(ScanRecordingFilesError::ReadFailed(e.to_string())),
        }
    }

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::{
        is_recording_file, scan_recording_files, select_upload_configs, status_label_for_tasks,
    };
    use shared::{TaskStatus, UploadConfig, UploadTemplate};
    use std::path::Path as FsPath;
    use uuid::Uuid;

    #[test]
    fn recording_file_filter_accepts_supported_extensions_case_insensitively() {
        assert!(is_recording_file(FsPath::new("a.mp4")));
        assert!(is_recording_file(FsPath::new("a.MKV")));
        assert!(is_recording_file(FsPath::new("a.ts")));
        assert!(is_recording_file(FsPath::new("a.flv")));
        assert!(!is_recording_file(FsPath::new("a.txt")));
        assert!(!is_recording_file(FsPath::new("a")));
    }

    #[tokio::test]
    async fn scan_recording_files_returns_only_supported_files_sorted() {
        let dir = std::env::temp_dir().join(format!("omnistream-scan-{}", Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.expect("create temp dir");
        tokio::fs::write(dir.join("b.mp4"), b"x").await.expect("write mp4");
        tokio::fs::write(dir.join("a.MKV"), b"x").await.expect("write mkv");
        tokio::fs::write(dir.join("c.txt"), b"x").await.expect("write txt");
        tokio::fs::create_dir(dir.join("nested.ts")).await.expect("create nested dir");

        let files = scan_recording_files(&dir).await.expect("scan files");

        assert_eq!(files.len(), 2);
        assert!(files[0].ends_with("a.MKV"));
        assert!(files[1].ends_with("b.mp4"));

        tokio::fs::remove_dir_all(&dir).await.expect("cleanup temp dir");
    }

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

    #[test]
    fn select_upload_configs_keeps_only_linked_templates_in_order() {
        let uploads = vec![
            UploadTemplate {
                id: "u1".to_string(),
                name: "one".to_string(),
                config: UploadConfig {
                    title: Some("t1".to_string()),
                    ..Default::default()
                },
            },
            UploadTemplate {
                id: "u2".to_string(),
                name: "two".to_string(),
                config: UploadConfig {
                    title: Some("t2".to_string()),
                    ..Default::default()
                },
            },
        ];

        let selected = select_upload_configs(
            &["u2".to_string(), "missing".to_string(), "u1".to_string()],
            &uploads,
        );

        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].title.as_deref(), Some("t2"));
        assert_eq!(selected[1].title.as_deref(), Some("t1"));
    }
}
