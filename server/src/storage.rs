use axum::{Json, http::StatusCode};
use shared::StorageStats;

use crate::storage_guard::recording_disk_space_snapshot;

pub async fn get_storage_stats() -> (StatusCode, Json<Option<StorageStats>>) {
    match get_storage_stats_service().await {
        Ok(stats) => (StatusCode::OK, Json(Some(stats))),
        Err(e) => {
            tracing::error!("Failed to get storage stats: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
        }
    }
}

pub(crate) async fn get_storage_stats_service() -> Result<StorageStats, String> {
    let snapshot = recording_disk_space_snapshot().await?;
    let total_bytes = snapshot.total_kb.saturating_mul(1024);
    let available_bytes = snapshot.available_kb.saturating_mul(1024);
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    let used_percent =
        if total_bytes == 0 { 0.0 } else { (used_bytes as f64 / total_bytes as f64) * 100.0 };

    Ok(StorageStats {
        path: snapshot.path.to_string_lossy().to_string(),
        total_bytes,
        used_bytes,
        available_bytes,
        used_percent,
    })
}
