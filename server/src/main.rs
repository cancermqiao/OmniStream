use anyhow::{Context, Result};

mod accounts;
mod app;
mod checker;
mod db;
mod downloads;
mod monitor;
mod recording;
mod router;
mod settings;
mod state;
mod tasks;
mod uploader;
mod uploads;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let state = app::build_state().await.context("failed to build application state")?;

    tokio::spawn(monitor::run_monitor(state.clone()));

    let app = router::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .context("failed to bind server to 0.0.0.0:3000")?;
    match listener.local_addr() {
        Ok(addr) => tracing::info!("Server listening on {}", addr),
        Err(e) => tracing::warn!("Server started but local_addr lookup failed: {}", e),
    }
    axum::serve(listener, app).await.context("axum server exited with error")?;
    Ok(())
}
