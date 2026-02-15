use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::state::SharedState;

use super::{
    bili,
    models::{AccountDeleteRequest, AccountRenameRequest, QrConfirmRequest, QrStartResponse},
    storage,
};

pub async fn list_accounts() -> Json<Vec<shared::UploadAccount>> {
    Json(storage::scan_saved_accounts().await)
}

pub async fn rename_account(Json(payload): Json<AccountRenameRequest>) -> StatusCode {
    match storage::rename_account(&payload.account_file, &payload.display_name).await {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            if err.contains("invalid") {
                StatusCode::BAD_REQUEST
            } else {
                tracing::error!("{err}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

pub async fn delete_account(Json(payload): Json<AccountDeleteRequest>) -> StatusCode {
    match storage::delete_account(&payload.account_file).await {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            if err.contains("invalid") {
                StatusCode::BAD_REQUEST
            } else if err.contains("Failed to delete account file") {
                tracing::error!("{err}");
                StatusCode::NOT_FOUND
            } else {
                tracing::error!("{err}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

pub async fn start_account_qrcode_login(
    State(state): State<SharedState>,
) -> Result<Json<QrStartResponse>, StatusCode> {
    let value = bili::get_qrcode().await.map_err(|e| {
        tracing::error!("{e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let qr_url = value["data"]["url"].as_str().map(str::to_string).ok_or_else(|| {
        tracing::error!("qrcode response missing data.url: {}", value);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let session_id = Uuid::new_v4().to_string();
    state.login_sessions.insert(session_id.clone(), value);
    Ok(Json(QrStartResponse { session_id, qr_url }))
}

pub async fn confirm_account_qrcode_login(
    State(state): State<SharedState>,
    Json(payload): Json<QrConfirmRequest>,
) -> StatusCode {
    let Some((_, qrcode_value)) = state.login_sessions.remove(&payload.session_id) else {
        return StatusCode::NOT_FOUND;
    };

    let login_info = match bili::login_by_qrcode(qrcode_value).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("{e}");
            return StatusCode::BAD_REQUEST;
        }
    };

    if let Err(e) = storage::save_login_info(login_info).await {
        tracing::error!("Failed to save login info: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
