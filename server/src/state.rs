use dashmap::DashMap;
use shared::{RecordingSettings, StreamTask};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{checker::StreamlinkChecker, db::Db};

pub struct RecorderHandle {
    pub abort_handle: tokio::task::AbortHandle,
}

pub struct AppState {
    pub tasks: DashMap<String, StreamTask>,
    pub handles: DashMap<String, RecorderHandle>,
    pub checking_urls: DashMap<String, ()>,
    pub db: Db,
    pub checker: StreamlinkChecker,
    pub recording_settings: Arc<RwLock<RecordingSettings>>,
    pub login_sessions: DashMap<String, serde_json::Value>,
}

pub type SharedState = Arc<AppState>;
