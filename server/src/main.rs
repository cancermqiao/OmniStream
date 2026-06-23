use anyhow::{Context, Result};

mod accounts;
mod app;
mod checker;
mod db;
mod downloads;
mod downloads_service;
mod monitor;
mod recording;
mod router;
mod settings;
mod state;
mod task_launcher;
mod tasks;
mod uploader;
mod uploads;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let state = app::build_state().await.context("failed to build application state")?;

    tokio::spawn(monitor::run_monitor(state.clone()));

    let app = router::build_router(state);
    let bind_addr = resolve_bind_addr();

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind server to {bind_addr}"))?;
    match listener.local_addr() {
        Ok(addr) => tracing::info!("Server listening on {}", addr),
        Err(e) => tracing::warn!("Server started but local_addr lookup failed: {}", e),
    }
    axum::serve(listener, app).await.context("axum server exited with error")?;
    Ok(())
}

fn resolve_bind_addr() -> String {
    if let Ok(addr) = std::env::var("BILIUP_BIND_ADDR")
        && !addr.trim().is_empty()
    {
        return addr;
    }

    let port = std::env::var("API_PORT").unwrap_or_else(|_| "3000".to_string());
    format!("0.0.0.0:{port}")
}
