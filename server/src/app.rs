use anyhow::{Result, anyhow};
use dashmap::DashMap;
use shared::RecordingSettings;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    checker::StreamlinkChecker,
    db::Db,
    state::{AppState, SharedState},
};

pub async fn build_state() -> Result<SharedState> {
    let db_path = resolve_db_path();
    let db = Db::new(&db_path).await.map_err(|e| anyhow!(e.to_string()))?;
    tracing::info!("Using database file: {}", db_path);

    let checker = StreamlinkChecker::new();
    let recording_settings = match db.get_recording_settings().await {
        Ok(Some(settings)) => settings,
        Ok(None) => RecordingSettings {
            segment_size_mb: None,
            segment_time_sec: Some(3600),
            ..Default::default()
        },
        Err(e) => {
            tracing::error!("Failed to load recording settings from DB, using defaults: {}", e);
            RecordingSettings {
                segment_size_mb: None,
                segment_time_sec: Some(3600),
                ..Default::default()
            }
        }
    };
    let recording_settings = Arc::new(RwLock::new(recording_settings));

    Ok(Arc::new(AppState {
        tasks: DashMap::new(),
        handles: DashMap::new(),
        checking_urls: DashMap::new(),
        db,
        checker,
        recording_settings,
        login_sessions: DashMap::new(),
    }))
}

fn resolve_db_path() -> String {
    std::env::var("BILIUP_DB_PATH").unwrap_or_else(|_| "data/omnistream.db".to_string())
}
