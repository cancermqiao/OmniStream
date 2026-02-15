use biliup::uploader::credential::LoginInfo;
use shared::UploadAccount;
use std::collections::BTreeMap;
use std::path::{Path as FsPath, PathBuf};

use super::{bili, models::CookieFile};

pub async fn scan_saved_accounts() -> Vec<UploadAccount> {
    let dir = cookies_dir();
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return vec![];
    };

    let mut files = Vec::<PathBuf>::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if path.is_file() && name.ends_with(".json") && name != "accounts_meta.json" {
            files.push(path);
        }
    }

    let meta = load_account_meta().await;
    let mut accounts = Vec::new();
    for file in files {
        let mut account = inspect_cookie_account(file.clone()).await;
        let key = file.file_name().and_then(|s| s.to_str()).map(str::to_string).unwrap_or_default();
        if let Some(display_name) = meta.get(&key) {
            account.name = display_name.clone();
        }
        accounts.push(account);
    }
    accounts.sort_by(|a, b| a.name.cmp(&b.name));
    accounts
}

pub async fn rename_account(account_file: &str, display_name: &str) -> Result<(), String> {
    let key = account_file_key(account_file);
    let name = display_name.trim();
    if key.is_empty() || name.is_empty() {
        return Err("invalid account_file or display_name".to_string());
    }

    let mut meta = load_account_meta().await;
    meta.insert(key, name.to_string());
    save_account_meta(&meta).await.map_err(|e| format!("Failed to save account meta: {e}"))
}

pub async fn delete_account(account_file: &str) -> Result<(), String> {
    let key = account_file_key(account_file);
    if key.is_empty() {
        return Err("invalid account_file".to_string());
    }

    let full_path = cookies_dir().join(&key);
    tokio::fs::remove_file(&full_path)
        .await
        .map_err(|e| format!("Failed to delete account file {}: {e}", full_path.display()))?;

    let mut meta = load_account_meta().await;
    meta.remove(&key);
    save_account_meta(&meta)
        .await
        .map_err(|e| format!("Failed to save account meta after delete: {e}"))
}

pub async fn save_login_info(login_info: LoginInfo) -> Result<(), Box<dyn std::error::Error>> {
    tokio::fs::create_dir_all(cookies_dir()).await?;
    let (name, mid) = bili::nav_from_cookie_info(&login_info.cookie_info)
        .await
        .unwrap_or_else(|| ("bilibili-user".to_string(), 0));

    let sanitized_name = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();

    let file_name = format!("cookies_{}_{}.json", mid, sanitized_name);
    let full_path = cookies_dir().join(file_name);
    let file = std::fs::File::create(full_path)?;
    serde_json::to_writer_pretty(file, &login_info)?;
    Ok(())
}

fn cookies_dir() -> PathBuf {
    std::env::var("BILIUP_COOKIES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data/cookies"))
}

fn account_meta_file() -> PathBuf {
    cookies_dir().join("accounts_meta.json")
}

fn account_file_key(path: &str) -> String {
    FsPath::new(path).file_name().and_then(|s| s.to_str()).map(str::to_string).unwrap_or_default()
}

async fn load_account_meta() -> BTreeMap<String, String> {
    let path = account_meta_file();
    let Ok(raw) = tokio::fs::read_to_string(path).await else {
        return BTreeMap::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

async fn save_account_meta(
    meta: &BTreeMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    tokio::fs::create_dir_all(cookies_dir()).await?;
    tokio::fs::write(account_meta_file(), serde_json::to_string_pretty(meta)?).await?;
    Ok(())
}

async fn inspect_cookie_account(path: PathBuf) -> UploadAccount {
    let account_file = path.to_string_lossy().to_string();
    let fallback_name =
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown-cookie").to_string();

    let raw = match tokio::fs::read_to_string(&path).await {
        Ok(v) => v,
        Err(e) => {
            return UploadAccount {
                id: account_file.clone(),
                name: fallback_name,
                mid: None,
                account_file,
                valid: false,
                error: Some(e.to_string()),
            };
        }
    };

    let parsed: CookieFile = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            return UploadAccount {
                id: account_file.clone(),
                name: fallback_name,
                mid: None,
                account_file,
                valid: false,
                error: Some(format!("Invalid cookie json: {e}")),
            };
        }
    };

    let cookie_header = parsed
        .cookie_info
        .cookies
        .iter()
        .map(|c| format!("{}={}", c.name, c.value))
        .collect::<Vec<_>>()
        .join("; ");

    if cookie_header.is_empty() {
        return UploadAccount {
            id: account_file.clone(),
            name: fallback_name,
            mid: None,
            account_file,
            valid: false,
            error: Some("cookie_info.cookies is empty".to_string()),
        };
    }

    match bili::nav_from_cookie_header(&cookie_header).await {
        Ok((uname, mid)) => UploadAccount {
            id: account_file.clone(),
            name: uname,
            mid: Some(mid),
            account_file,
            valid: true,
            error: None,
        },
        Err(err) => UploadAccount {
            id: account_file.clone(),
            name: fallback_name,
            mid: None,
            account_file,
            valid: false,
            error: Some(err),
        },
    }
}
