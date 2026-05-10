use shared::{DownloadConfig, UploadConfig, UploadTemplate};
use std::path::Path;

use crate::state::SharedState;

#[derive(Debug)]
pub(crate) enum ScanRecordingFilesError {
    NotFound(String),
    ReadFailed(String),
}

pub(crate) async fn load_download_for_manual_upload(
    state: &SharedState,
    id: &str,
) -> Result<DownloadConfig, (axum::http::StatusCode, String)> {
    let downloads = state.db.get_downloads().await.map_err(|e| {
        tracing::error!("Failed to load downloads: {}", e);
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to load downloads".to_string(),
        )
    })?;

    downloads.into_iter().find(|d| d.id == id).ok_or_else(|| {
        tracing::warn!("Manual upload rejected: download config not found, id={}", id);
        (axum::http::StatusCode::NOT_FOUND, "download config not found".to_string())
    })
}

pub(crate) async fn resolve_manual_upload_configs(
    state: &SharedState,
    download: &DownloadConfig,
) -> Result<Vec<UploadConfig>, (axum::http::StatusCode, String)> {
    let uploads = state.db.get_uploads().await.map_err(|e| {
        tracing::error!("Failed to load uploads: {}", e);
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to load uploads".to_string(),
        )
    })?;

    Ok(select_upload_configs(&download.linked_upload_ids, &uploads))
}

pub(crate) fn select_upload_configs(
    linked_upload_ids: &[String],
    uploads: &[UploadTemplate],
) -> Vec<UploadConfig> {
    linked_upload_ids
        .iter()
        .filter_map(|uid| uploads.iter().find(|u| &u.id == uid))
        .map(|u| u.config.clone())
        .collect()
}

pub(crate) async fn resolve_auto_cleanup_after_upload(
    state: &SharedState,
    download: &DownloadConfig,
) -> bool {
    if download.use_custom_recording_settings {
        download.recording_settings.as_ref().map(|s| s.auto_cleanup_after_upload).unwrap_or(false)
    } else {
        state.recording_settings.read().await.auto_cleanup_after_upload
    }
}

pub(crate) fn is_recording_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(ext.as_str(), "mp4" | "flv" | "mkv" | "ts")
}

pub(crate) async fn scan_recording_files(
    task_dir: &Path,
) -> Result<Vec<String>, ScanRecordingFilesError> {
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
    use super::{is_recording_file, scan_recording_files, select_upload_configs};
    use shared::{UploadConfig, UploadTemplate};
    use std::path::Path;
    use uuid::Uuid;

    #[test]
    fn recording_file_filter_accepts_supported_extensions_case_insensitively() {
        assert!(is_recording_file(Path::new("a.mp4")));
        assert!(is_recording_file(Path::new("a.MKV")));
        assert!(is_recording_file(Path::new("a.ts")));
        assert!(is_recording_file(Path::new("a.flv")));
        assert!(!is_recording_file(Path::new("a.txt")));
        assert!(!is_recording_file(Path::new("a")));
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
