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
async fn main() {
    tracing_subscriber::fmt::init();
    let state = app::build_state().await;

    tokio::spawn(monitor::run_monitor(state.clone()));

    let app = router::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
