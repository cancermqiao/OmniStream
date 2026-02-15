use dashmap::DashMap;
use shared::RecordingSettings;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    checker::StreamlinkChecker,
    db::Db,
    state::{AppState, SharedState},
};

pub async fn build_state() -> SharedState {
    let db_path = resolve_db_path();
    let db = Db::new(&db_path).await.expect("Failed to initialize database");
    tracing::info!("Using database file: {}", db_path);

    let checker = StreamlinkChecker::new();
    let recording_settings = Arc::new(RwLock::new(
        db.get_recording_settings().await.ok().flatten().unwrap_or(RecordingSettings {
            segment_size_mb: None,
            segment_time_sec: Some(3600),
            ..Default::default()
        }),
    ));

    Arc::new(AppState {
        tasks: DashMap::new(),
        handles: DashMap::new(),
        checking_urls: DashMap::new(),
        db,
        checker,
        recording_settings,
        login_sessions: DashMap::new(),
    })
}

fn resolve_db_path() -> String {
    std::env::var("BILIUP_DB_PATH").unwrap_or_else(|_| "data/omnistream.db".to_string())
}
