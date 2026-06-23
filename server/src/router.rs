use axum::{
    Router,
    http::StatusCode,
    routing::{any, delete, get, post},
};
use std::path::PathBuf;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

use crate::{accounts, downloads, settings, state::SharedState, tasks, uploads};

pub fn build_router(state: SharedState) -> Router {
    let web_dir = resolve_web_dir();
    let index_file = web_dir.join("index.html");

    if index_file.exists() {
        tracing::info!("Serving Web UI from {}", web_dir.display());
    } else {
        tracing::warn!(
            "Web UI index.html not found at {}; API routes will still run",
            index_file.display()
        );
    }

    Router::new()
        .route("/api/tasks", get(tasks::list_tasks).post(tasks::add_task))
        .route("/api/tasks/{id}/stop", post(tasks::stop_task))
        .route("/api/downloads", get(downloads::list_downloads).post(downloads::add_download))
        .route("/api/downloads/{id}", delete(downloads::delete_download))
        .route("/api/downloads/{id}/upload", post(downloads::trigger_manual_upload))
        .route("/api/downloads/{id}/stop", post(downloads::stop_download))
        .route("/api/downloads/{id}/resume", post(downloads::resume_download))
        .route("/api/downloads/{id}/files", delete(downloads::clear_download_files))
        .route("/api/uploads", get(uploads::list_uploads).post(uploads::add_upload))
        .route("/api/uploads/{id}", delete(uploads::delete_upload))
        .route("/api/accounts", get(accounts::list_accounts))
        .route("/api/accounts/rename", post(accounts::rename_account))
        .route("/api/accounts/delete", post(accounts::delete_account))
        .route("/api/accounts/qrcode/start", post(accounts::start_account_qrcode_login))
        .route("/api/accounts/qrcode/confirm", post(accounts::confirm_account_qrcode_login))
        .route(
            "/api/settings/recording",
            get(settings::get_recording_settings).post(settings::set_recording_settings),
        )
        .route("/api/{*path}", any(api_not_found))
        .fallback_service(ServeDir::new(web_dir).fallback(ServeFile::new(index_file)))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn api_not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

fn resolve_web_dir() -> PathBuf {
    if let Ok(v) = std::env::var("BILIUP_WEB_DIR") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    [
        "web/public",
        "target/dx/app/release/web/public",
        "target/dx/web/release/web/public",
        "web/dist",
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|dir| dir.join("index.html").exists())
    .unwrap_or_else(|| PathBuf::from("web/public"))
}
