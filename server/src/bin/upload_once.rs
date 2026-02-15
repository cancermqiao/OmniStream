use anyhow::{Context, Result};
use shared::UploadConfig;

#[path = "../uploader/mod.rs"]
mod uploader;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let filename = std::env::args()
        .nth(1)
        .context("usage: cargo run -p server --bin upload_once -- <file>")?;
    let account_file =
        std::env::var("BILIUP_COOKIE_FILE").unwrap_or_else(|_| "cookies.json".to_string());
    let title = std::env::var("BILIUP_TEST_TITLE").ok();
    let tags = std::env::var("BILIUP_TEST_TAGS")
        .ok()
        .map(|s| {
            s.split(',').map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| vec!["omnistream".to_string()]);

    let uploader = uploader::UploadTarget::Bilibili.create_uploader();
    let config = UploadConfig { title, tags, account_file, ..Default::default() };

    uploader.upload(vec![filename], &config, None, "upload_once").await?;
    println!("upload test finished successfully");
    Ok(())
}
