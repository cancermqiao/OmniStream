use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_http::cors::CorsLayer;

use crate::{accounts, downloads, settings, state::SharedState, tasks, uploads};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/api/tasks", get(tasks::list_tasks).post(tasks::add_task))
        .route("/api/tasks/{id}/stop", post(tasks::stop_task))
        .route("/api/downloads", get(downloads::list_downloads).post(downloads::add_download))
        .route("/api/downloads/{id}", delete(downloads::delete_download))
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
        .layer(CorsLayer::permissive())
        .with_state(state)
}
