use biliup::uploader::credential::{Credential, LoginInfo};

use super::models::NavResponse;

pub async fn get_qrcode() -> Result<serde_json::Value, String> {
    let credential = Credential::new(None);
    credential.get_qrcode().await.map_err(|e| format!("Failed to get qrcode: {e}"))
}

pub async fn login_by_qrcode(qrcode_value: serde_json::Value) -> Result<LoginInfo, String> {
    let credential = Credential::new(None);
    credential
        .login_by_qrcode(qrcode_value)
        .await
        .map_err(|e| format!("QR login not completed: {e}"))
}

pub async fn nav_from_cookie_header(cookie_header: &str) -> Result<(String, u64), String> {
    let resp = reqwest::Client::new()
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header(reqwest::header::COOKIE, cookie_header)
        .send()
        .await
        .map_err(|e| format!("nav request failed: {e}"))?;

    let nav: NavResponse = resp.json().await.map_err(|e| format!("nav decode failed: {e}"))?;

    if nav.code != 0 {
        return Err(format!("nav code: {}", nav.code));
    }

    let data = nav.data.ok_or_else(|| "nav data missing".to_string())?;
    Ok((data.uname, data.mid))
}

pub async fn nav_from_cookie_info(cookie_info: &serde_json::Value) -> Option<(String, u64)> {
    let cookie_header = cookie_info["cookies"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    Some(format!("{}={}", v.get("name")?.as_str()?, v.get("value")?.as_str()?))
                })
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default();

    if cookie_header.is_empty() {
        return None;
    }

    nav_from_cookie_header(&cookie_header).await.ok()
}
