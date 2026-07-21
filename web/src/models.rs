use shared::{DownloadConfig, RecordingSettings, StorageStats, UploadAccount, UploadTemplate};

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Downloads,
    Accounts,
    Uploads,
    Settings,
}

#[derive(Clone, Default)]
pub struct AppData {
    pub downloads: Vec<DownloadConfig>,
    pub uploads: Vec<UploadTemplate>,
    pub accounts: Vec<UploadAccount>,
    pub recording_settings: RecordingSettings,
    pub storage_stats: Option<StorageStats>,
}

pub use shared::QrStartResponse;
