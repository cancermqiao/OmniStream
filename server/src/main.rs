use anyhow::{Context, Result};
use std::{fs::OpenOptions, path::Path};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::{LevelFilter, filter_fn},
    fmt,
    prelude::*,
};

mod accounts;
mod app;
mod checker;
mod db;
mod downloads;
mod downloads_service;
mod monitor;
mod platform;
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
    let _log_guards = init_logging().context("failed to initialize logging")?;
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

fn init_logging() -> Result<Vec<WorkerGuard>> {
    std::fs::create_dir_all("logs").context("failed to create logs directory")?;

    let (info_writer, info_guard) = tracing_appender::non_blocking(open_log_file("logs/info.log")?);
    let (warn_writer, warn_guard) = tracing_appender::non_blocking(open_log_file("logs/warn.log")?);
    let (error_writer, error_guard) =
        tracing_appender::non_blocking(open_log_file("logs/error.log")?);

    let stdout_layer = fmt::layer().with_filter(LevelFilter::INFO);
    let info_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(info_writer)
        .with_filter(filter_fn(|metadata| *metadata.level() == Level::INFO));
    let warn_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(warn_writer)
        .with_filter(filter_fn(|metadata| *metadata.level() == Level::WARN));
    let error_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(error_writer)
        .with_filter(filter_fn(|metadata| *metadata.level() == Level::ERROR));

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(info_layer)
        .with(warn_layer)
        .with(error_layer)
        .init();

    Ok(vec![info_guard, warn_guard, error_guard])
}

fn open_log_file(path: impl AsRef<Path>) -> Result<std::fs::File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.as_ref())
        .with_context(|| format!("failed to open log file {}", path.as_ref().display()))
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
