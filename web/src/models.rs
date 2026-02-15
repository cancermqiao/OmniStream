use serde::{Deserialize, Serialize};
use shared::{DownloadConfig, RecordingSettings, UploadAccount, UploadTemplate};

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
}

#[derive(Clone, Deserialize, PartialEq)]
pub struct QrStartResponse {
    pub session_id: String,
    pub qr_url: String,
}

#[derive(Serialize)]
pub struct QrConfirmRequest {
    pub session_id: String,
}

#[derive(Serialize)]
pub struct AccountRenameRequest {
    pub account_file: String,
    pub display_name: String,
}

#[derive(Serialize)]
pub struct AccountDeleteRequest {
    pub account_file: String,
}
