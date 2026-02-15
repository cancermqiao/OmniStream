use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CookieFile {
    pub cookie_info: CookieInfo,
}

#[derive(Debug, Deserialize)]
pub struct CookieInfo {
    pub cookies: Vec<CookieKV>,
}

#[derive(Debug, Deserialize)]
pub struct CookieKV {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NavResponse {
    pub code: i32,
    pub data: Option<NavData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NavData {
    pub uname: String,
    pub mid: u64,
}

#[derive(Debug, Serialize)]
pub struct QrStartResponse {
    pub session_id: String,
    pub qr_url: String,
}

#[derive(Debug, Deserialize)]
pub struct QrConfirmRequest {
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountRenameRequest {
    pub account_file: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountDeleteRequest {
    pub account_file: String,
}
