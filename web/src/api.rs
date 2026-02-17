use crate::models::{
    AccountDeleteRequest, AccountRenameRequest, QrConfirmRequest, QrStartResponse,
};
use shared::{DownloadConfig, RecordingSettings, UploadAccount, UploadTemplate};

fn join_url(base: &str, path: &str) -> String {
    format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'))
}

fn push_unique(bases: &mut Vec<String>, base: String) {
    if !bases.iter().any(|b| b == &base) {
        bases.push(base);
    }
}

fn api_bases(api_url: &str) -> Vec<String> {
    let primary = api_url.trim_end_matches('/').to_string();
    let mut bases = Vec::new();

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(origin) = window.location().origin()
        {
            // Prefer same-origin /api when a reverse proxy is configured.
            let same_origin = format!("{origin}/api");
            push_unique(&mut bases, same_origin);

            // Fallback for split-port deployment: web on :8080, api on :3000.
            if let Ok(hostname) = window.location().hostname()
                && !hostname.is_empty()
                && let Ok(protocol) = window.location().protocol()
            {
                let proto = protocol.trim_end_matches(':');
                let host_3000 = format!("{proto}://{hostname}:3000/api");
                push_unique(&mut bases, host_3000);

                // Avoid trying loopback first when page is opened from a public host.
                let page_is_loopback = hostname == "127.0.0.1" || hostname == "localhost";
                let primary_is_loopback =
                    primary.contains("127.0.0.1") || primary.contains("localhost");
                if !(primary_is_loopback && !page_is_loopback) {
                    push_unique(&mut bases, primary.clone());
                }
            } else {
                push_unique(&mut bases, primary.clone());
            }
        } else {
            push_unique(&mut bases, primary.clone());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        push_unique(&mut bases, primary.clone());
    }

    if bases.is_empty() {
        push_unique(&mut bases, primary);
    }

    bases
}

pub async fn fetch_downloads(api_url: &str) -> Option<Vec<DownloadConfig>> {
    for base in api_bases(api_url) {
        if let Ok(resp) = reqwest::get(join_url(&base, "/downloads")).await
            && let Ok(v) = resp.json().await
        {
            return Some(v);
        }
    }
    None
}

pub async fn fetch_uploads(api_url: &str) -> Option<Vec<UploadTemplate>> {
    for base in api_bases(api_url) {
        if let Ok(resp) = reqwest::get(join_url(&base, "/uploads")).await
            && let Ok(v) = resp.json().await
        {
            return Some(v);
        }
    }
    None
}

pub async fn fetch_accounts(api_url: &str) -> Option<Vec<UploadAccount>> {
    for base in api_bases(api_url) {
        if let Ok(resp) = reqwest::get(join_url(&base, "/accounts")).await
            && let Ok(v) = resp.json().await
        {
            return Some(v);
        }
    }
    None
}

pub async fn save_download(api_url: &str, payload: &DownloadConfig) {
    for base in api_bases(api_url) {
        if let Ok(resp) =
            reqwest::Client::new().post(join_url(&base, "/downloads")).json(payload).send().await
            && resp.status().is_success()
        {
            break;
        }
    }
}

pub async fn delete_download(api_url: &str, id: &str) {
    for base in api_bases(api_url) {
        if let Ok(resp) =
            reqwest::Client::new().delete(join_url(&base, &format!("/downloads/{id}"))).send().await
            && resp.status().is_success()
        {
            break;
        }
    }
}

pub async fn save_upload(api_url: &str, payload: &UploadTemplate) {
    for base in api_bases(api_url) {
        if let Ok(resp) =
            reqwest::Client::new().post(join_url(&base, "/uploads")).json(payload).send().await
            && resp.status().is_success()
        {
            break;
        }
    }
}

pub async fn delete_upload(api_url: &str, id: &str) {
    for base in api_bases(api_url) {
        if let Ok(resp) =
            reqwest::Client::new().delete(join_url(&base, &format!("/uploads/{id}"))).send().await
            && resp.status().is_success()
        {
            break;
        }
    }
}

pub async fn start_qr_login(api_url: &str) -> Result<QrStartResponse, String> {
    let mut last_err = "request not sent".to_string();

    for base in api_bases(api_url) {
        let endpoint = join_url(&base, "/accounts/qrcode/start");
        match reqwest::Client::new().post(&endpoint).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    last_err = format!("{endpoint} -> status {}", resp.status());
                    continue;
                }
                return resp
                    .json::<QrStartResponse>()
                    .await
                    .map_err(|e| format!("{endpoint} -> decode error: {e}"));
            }
            Err(e) => {
                last_err = format!("{endpoint} -> {e}");
            }
        }
    }

    Err(last_err)
}

pub async fn confirm_qr_login(api_url: &str, session_id: String) -> Result<(), String> {
    let mut last_err = "request not sent".to_string();

    for base in api_bases(api_url) {
        let endpoint = join_url(&base, "/accounts/qrcode/confirm");
        match reqwest::Client::new()
            .post(&endpoint)
            .json(&QrConfirmRequest { session_id: session_id.clone() })
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    return Ok(());
                }
                last_err = format!("{endpoint} -> status {}", resp.status());
            }
            Err(e) => {
                last_err = format!("{endpoint} -> {e}");
            }
        }
    }

    Err(last_err)
}

pub async fn rename_account(api_url: &str, account_file: String, display_name: String) {
    for base in api_bases(api_url) {
        if let Ok(resp) = reqwest::Client::new()
            .post(join_url(&base, "/accounts/rename"))
            .json(&AccountRenameRequest {
                account_file: account_file.clone(),
                display_name: display_name.clone(),
            })
            .send()
            .await
            && resp.status().is_success()
        {
            break;
        }
    }
}

pub async fn delete_account(api_url: &str, account_file: String) {
    for base in api_bases(api_url) {
        if let Ok(resp) = reqwest::Client::new()
            .post(join_url(&base, "/accounts/delete"))
            .json(&AccountDeleteRequest { account_file: account_file.clone() })
            .send()
            .await
            && resp.status().is_success()
        {
            break;
        }
    }
}

pub async fn fetch_recording_settings(api_url: &str) -> Option<RecordingSettings> {
    for base in api_bases(api_url) {
        if let Ok(resp) = reqwest::get(join_url(&base, "/settings/recording")).await
            && let Ok(v) = resp.json().await
        {
            return Some(v);
        }
    }
    None
}

pub async fn save_recording_settings(api_url: &str, settings: &RecordingSettings) {
    for base in api_bases(api_url) {
        if let Ok(resp) = reqwest::Client::new()
            .post(join_url(&base, "/settings/recording"))
            .json(settings)
            .send()
            .await
            && resp.status().is_success()
        {
            break;
        }
    }
}

pub async fn trigger_manual_upload(api_url: &str, id: &str) -> Result<(), String> {
    let mut last_err = "request not sent".to_string();

    for base in api_bases(api_url) {
        let endpoint = join_url(&base, &format!("/downloads/{id}/upload"));
        match reqwest::Client::new().post(&endpoint).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    return Ok(());
                }
                last_err = format!("{endpoint} -> status {}", resp.status());
            }
            Err(e) => {
                last_err = format!("{endpoint} -> {e}");
            }
        }
    }

    Err(last_err)
}
