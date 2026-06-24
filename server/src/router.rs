use axum::{
    Router,
    http::StatusCode,
    routing::{any, delete, get, post},
};
use dioxus_server::{DioxusRouterExt, ServeConfig};
use shared::{DownloadConfig, QrStartResponse, RecordingSettings, UploadAccount, UploadTemplate};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::{accounts, downloads, settings, state::SharedState, tasks, uploads};

#[derive(Clone)]
struct FrontendBackend {
    state: SharedState,
}

fn message(err: (StatusCode, String)) -> String {
    err.1
}

#[async_trait::async_trait]
impl app::api::BackendApi for FrontendBackend {
    async fn fetch_downloads(&self) -> Result<Vec<DownloadConfig>, String> {
        downloads::list_downloads_service(&self.state).await.map_err(message)
    }

    async fn fetch_uploads(&self) -> Result<Vec<UploadTemplate>, String> {
        uploads::list_uploads_service(&self.state).await.map_err(message)
    }

    async fn fetch_accounts(&self) -> Result<Vec<UploadAccount>, String> {
        Ok(accounts::list_accounts_service().await)
    }

    async fn save_download(&self, payload: DownloadConfig) -> Result<(), String> {
        downloads::save_download_service(&self.state, payload).await.map_err(message)
    }

    async fn delete_download(&self, id: String) -> Result<(), String> {
        downloads::delete_download_service(&self.state, &id).await.map_err(message)
    }

    async fn clear_download_files(&self, id: String) -> Result<String, String> {
        downloads::clear_download_files_service(&self.state, &id).await.map_err(message)
    }

    async fn stop_download(&self, id: String) -> Result<String, String> {
        downloads::stop_download_service(&self.state, &id).await.map_err(message)
    }

    async fn resume_download(&self, id: String) -> Result<String, String> {
        downloads::resume_download_service(&self.state, &id).await.map_err(message)
    }

    async fn save_upload(&self, payload: UploadTemplate) -> Result<(), String> {
        uploads::save_upload_service(&self.state, payload).await.map_err(message)
    }

    async fn delete_upload(&self, id: String) -> Result<(), String> {
        uploads::delete_upload_service(&self.state, &id).await.map_err(message)
    }

    async fn start_qr_login(&self) -> Result<QrStartResponse, String> {
        accounts::start_account_qrcode_login_service(&self.state).await.map_err(message)
    }

    async fn confirm_qr_login(&self, session_id: String) -> Result<(), String> {
        accounts::confirm_account_qrcode_login_service(&self.state, session_id)
            .await
            .map_err(message)
    }

    async fn rename_account(
        &self,
        account_file: String,
        display_name: String,
    ) -> Result<(), String> {
        accounts::rename_account_service(account_file, display_name).await.map_err(message)
    }

    async fn delete_account(&self, account_file: String) -> Result<(), String> {
        accounts::delete_account_service(account_file).await.map_err(message)
    }

    async fn fetch_recording_settings(&self) -> Result<RecordingSettings, String> {
        Ok(settings::get_recording_settings_service(&self.state).await)
    }

    async fn save_recording_settings(&self, settings: RecordingSettings) -> Result<(), String> {
        settings::set_recording_settings_service(&self.state, settings).await.map_err(message)
    }

    async fn trigger_manual_upload(&self, id: String) -> Result<String, String> {
        downloads::trigger_manual_upload_service(&self.state, &id)
            .await
            .map(|(_, message)| message)
            .map_err(message)
    }
}

pub fn build_router(state: SharedState) -> Router {
    let backend: Arc<dyn app::api::BackendApi> = Arc::new(FrontendBackend { state: state.clone() });
    app::api::install_backend(backend.clone());

    let api_router = Router::new()
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
        .layer(CorsLayer::permissive())
        .with_state(state);

    let config = ServeConfig::new().context(backend);

    api_router.merge(Router::new().serve_dioxus_application(config, app::App))
}

async fn api_not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}
