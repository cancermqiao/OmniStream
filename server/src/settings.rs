use axum::{Json, extract::State, http::StatusCode};
use shared::RecordingSettings;

use crate::state::SharedState;

const MAX_SEGMENT_SIZE_MB: u64 = 102_400;
const MAX_SEGMENT_TIME_SEC: u64 = 86_400;

pub async fn get_recording_settings(State(state): State<SharedState>) -> Json<RecordingSettings> {
    Json(get_recording_settings_service(&state).await)
}

pub async fn get_recording_settings_service(state: &SharedState) -> RecordingSettings {
    state.recording_settings.read().await.clone()
}

pub async fn set_recording_settings(
    State(state): State<SharedState>,
    Json(payload): Json<RecordingSettings>,
) -> (StatusCode, String) {
    match set_recording_settings_service(&state, payload).await {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(response) => response,
    }
}

pub async fn set_recording_settings_service(
    state: &SharedState,
    payload: RecordingSettings,
) -> Result<(), (StatusCode, String)> {
    let settings = match sanitize_recording_settings(payload) {
        Ok(settings) => settings,
        Err(message) => {
            tracing::warn!("Rejected recording settings update: {}", message);
            return Err((StatusCode::BAD_REQUEST, message));
        }
    };

    if let Err(e) = state.db.save_recording_settings(&settings).await {
        tracing::error!("Failed to save recording settings: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to save recording settings".to_string(),
        ));
    }
    {
        let mut lock = state.recording_settings.write().await;
        *lock = settings;
    }
    Ok(())
}

pub(crate) fn sanitize_recording_settings(
    mut settings: RecordingSettings,
) -> Result<RecordingSettings, String> {
    if settings.segment_size_mb == Some(0) {
        settings.segment_size_mb = None;
    }
    if settings.segment_time_sec == Some(0) {
        settings.segment_time_sec = None;
    }

    normalize_quality(&mut settings.quality.bilibili);
    normalize_quality(&mut settings.quality.douyu);
    normalize_quality(&mut settings.quality.huya);
    normalize_quality(&mut settings.quality.tiktok);
    normalize_quality(&mut settings.quality.douyin);
    normalize_quality(&mut settings.quality.twitch);
    normalize_quality(&mut settings.quality.youtube);
    normalize_quality(&mut settings.quality.kick);
    normalize_quality(&mut settings.quality.default_quality);

    if let Some(size) = settings.segment_size_mb
        && size > MAX_SEGMENT_SIZE_MB
    {
        return Err(format!(
            "segment_size_mb exceeds maximum allowed value: {}",
            MAX_SEGMENT_SIZE_MB
        ));
    }
    if let Some(time) = settings.segment_time_sec
        && time > MAX_SEGMENT_TIME_SEC
    {
        return Err(format!(
            "segment_time_sec exceeds maximum allowed value: {}",
            MAX_SEGMENT_TIME_SEC
        ));
    }

    Ok(settings)
}

fn normalize_quality(v: &mut String) {
    let trimmed = v.trim();
    if trimmed.is_empty() {
        *v = "best".to_string();
    } else {
        *v = trimmed.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_recording_settings;
    use shared::RecordingSettings;

    #[test]
    fn sanitize_recording_settings_normalizes_zero_and_blank_quality() {
        let mut settings = RecordingSettings {
            segment_size_mb: Some(0),
            segment_time_sec: Some(0),
            ..Default::default()
        };
        settings.quality.bilibili = "  ".to_string();
        settings.quality.default_quality = "  best  ".to_string();

        let sanitized = sanitize_recording_settings(settings).expect("settings are valid");

        assert_eq!(sanitized.segment_size_mb, None);
        assert_eq!(sanitized.segment_time_sec, None);
        assert_eq!(sanitized.quality.bilibili, "best");
        assert_eq!(sanitized.quality.default_quality, "best");
    }

    #[test]
    fn sanitize_recording_settings_rejects_extreme_values() {
        let settings = RecordingSettings { segment_size_mb: Some(102_401), ..Default::default() };

        assert!(sanitize_recording_settings(settings).is_err());
    }
}
