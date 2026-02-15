use anyhow::{Context, Result, anyhow};
use shared::UploadConfig;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Row, SqlitePool};
use std::path::Path;

#[path = "../uploader/mod.rs"]
mod uploader;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut args = std::env::args().skip(1);
    let template_name = args
        .next()
        .context("usage: cargo run -p server --bin upload_template_batch -- <template_name> <file1> [file2] ...")?;
    let files: Vec<String> = args.collect();
    if files.is_empty() {
        return Err(anyhow!("no files provided"));
    }

    for f in &files {
        if !Path::new(f).is_file() {
            return Err(anyhow!("file not found: {}", f));
        }
    }

    let db_path = resolve_db_path();
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::new().filename(&db_path).create_if_missing(false),
    )
    .await
    .with_context(|| format!("failed to open db: {db_path}"))?;

    let row = sqlx::query("SELECT name, config FROM uploads WHERE name = ? LIMIT 1")
        .bind(&template_name)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| anyhow!("upload template not found: {template_name}"))?;

    let task_name: String = row.get("name");
    let config_raw: String = row.get("config");
    let config: UploadConfig =
        serde_json::from_str(&config_raw).with_context(|| "invalid upload config json in db")?;

    println!(
        "using template '{}' account='{}' tid={} tags={:?}",
        task_name, config.account_file, config.tid, config.tags
    );
    println!("uploading {} files...", files.len());

    let uploader = uploader::UploadTarget::Bilibili.create_uploader();
    uploader.upload(files, &config, None, &task_name).await?;

    println!("batch upload finished successfully");
    Ok(())
}

fn resolve_db_path() -> String {
    let default_db_path = if Path::new("server").is_dir() {
        "server/omnistream.db".to_string()
    } else {
        "omnistream.db".to_string()
    };
    std::env::var("BILIUP_DB_PATH").unwrap_or(default_db_path)
}
