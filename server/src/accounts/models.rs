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
