use axum::{Json, extract::State, http::StatusCode};
use shared::RecordingSettings;

use crate::state::SharedState;

pub async fn get_recording_settings(State(state): State<SharedState>) -> Json<RecordingSettings> {
    Json(state.recording_settings.read().await.clone())
}

pub async fn set_recording_settings(
    State(state): State<SharedState>,
    Json(payload): Json<RecordingSettings>,
) -> StatusCode {
    let settings = sanitize(payload);
    {
        let mut lock = state.recording_settings.write().await;
        *lock = settings.clone();
    }
    if let Err(e) = state.db.save_recording_settings(&settings).await {
        tracing::error!("Failed to save recording settings: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

fn sanitize(mut settings: RecordingSettings) -> RecordingSettings {
    if settings.segment_size_mb == Some(0) {
        settings.segment_size_mb = None;
    }
    if settings.segment_time_sec == Some(0) {
        settings.segment_time_sec = None;
    }

    normalize_quality(&mut settings.quality.bilibili);
    normalize_quality(&mut settings.quality.douyu);
    normalize_quality(&mut settings.quality.huya);
    normalize_quality(&mut settings.quality.twitch);
    normalize_quality(&mut settings.quality.youtube);
    normalize_quality(&mut settings.quality.default_quality);

    settings
}

fn normalize_quality(v: &mut String) {
    let trimmed = v.trim();
    if trimmed.is_empty() {
        *v = "best".to_string();
    } else {
        *v = trimmed.to_string();
    }
}
