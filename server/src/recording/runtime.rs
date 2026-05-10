use super::quality_for_url;

pub(super) struct RecorderRuntimeConfig {
    pub(super) segment_size_bytes: Option<u64>,
    pub(super) segment_time_sec: Option<u64>,
    pub(super) quality: String,
    pub(super) auto_cleanup_after_upload: bool,
}

pub(super) fn build_runtime_config(
    url: &str,
    settings: &shared::RecordingSettings,
) -> RecorderRuntimeConfig {
    RecorderRuntimeConfig {
        segment_size_bytes: settings
            .segment_size_mb
            .and_then(|mb| mb.checked_mul(1024 * 1024))
            .filter(|v| *v > 0),
        segment_time_sec: settings.segment_time_sec.filter(|v| *v > 0),
        quality: quality_for_url(url, &settings.quality),
        auto_cleanup_after_upload: settings.auto_cleanup_after_upload,
    }
}
