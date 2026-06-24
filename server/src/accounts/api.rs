use axum::{Json, extract::State, http::StatusCode};
use shared::{AccountDeleteRequest, AccountRenameRequest, QrConfirmRequest, QrStartResponse};
use uuid::Uuid;

use crate::state::SharedState;

use super::{bili, storage};

pub async fn list_accounts() -> Json<Vec<shared::UploadAccount>> {
    Json(list_accounts_service().await)
}

pub async fn list_accounts_service() -> Vec<shared::UploadAccount> {
    storage::scan_saved_accounts().await
}

pub async fn rename_account(Json(payload): Json<AccountRenameRequest>) -> StatusCode {
    match rename_account_service(payload.account_file, payload.display_name).await {
        Ok(()) => StatusCode::OK,
        Err((status, _)) => status,
    }
}

pub async fn delete_account(Json(payload): Json<AccountDeleteRequest>) -> StatusCode {
    match delete_account_service(payload.account_file).await {
        Ok(()) => StatusCode::OK,
        Err((status, _)) => status,
    }
}

pub async fn start_account_qrcode_login(
    State(state): State<SharedState>,
) -> Result<Json<QrStartResponse>, StatusCode> {
    start_account_qrcode_login_service(&state).await.map(Json).map_err(|(status, _)| status)
}

pub async fn start_account_qrcode_login_service(
    state: &SharedState,
) -> Result<QrStartResponse, (StatusCode, String)> {
    let value = bili::get_qrcode().await.map_err(|e| {
        tracing::error!("{e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to create qrcode".to_string())
    })?;

    let qr_url = value["data"]["url"].as_str().map(str::to_string).ok_or_else(|| {
        tracing::error!("qrcode response missing data.url: {}", value);
        (StatusCode::INTERNAL_SERVER_ERROR, "qrcode response missing url".to_string())
    })?;

    let session_id = Uuid::new_v4().to_string();
    state.login_sessions.insert(session_id.clone(), value);
    Ok(QrStartResponse { session_id, qr_url })
}

pub async fn confirm_account_qrcode_login(
    State(state): State<SharedState>,
    Json(payload): Json<QrConfirmRequest>,
) -> StatusCode {
    match confirm_account_qrcode_login_service(&state, payload.session_id).await {
        Ok(()) => StatusCode::OK,
        Err((status, _)) => status,
    }
}

pub async fn confirm_account_qrcode_login_service(
    state: &SharedState,
    session_id: String,
) -> Result<(), (StatusCode, String)> {
    let Some((_, qrcode_value)) = state.login_sessions.remove(&session_id) else {
        return Err((StatusCode::NOT_FOUND, "login session not found".to_string()));
    };

    let login_info = match bili::login_by_qrcode(qrcode_value).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("{e}");
            return Err((StatusCode::BAD_REQUEST, "qrcode login failed".to_string()));
        }
    };

    if let Err(e) = storage::save_login_info(login_info).await {
        tracing::error!("Failed to save login info: {}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to save login info".to_string()));
    }

    Ok(())
}

pub async fn rename_account_service(
    account_file: String,
    display_name: String,
) -> Result<(), (StatusCode, String)> {
    match storage::rename_account(&account_file, &display_name).await {
        Ok(()) => Ok(()),
        Err(err) => {
            if err.contains("invalid") {
                Err((StatusCode::BAD_REQUEST, err))
            } else {
                tracing::error!("{err}");
                Err((StatusCode::INTERNAL_SERVER_ERROR, err))
            }
        }
    }
}

pub async fn delete_account_service(account_file: String) -> Result<(), (StatusCode, String)> {
    match storage::delete_account(&account_file).await {
        Ok(()) => Ok(()),
        Err(err) => {
            if err.contains("invalid") {
                Err((StatusCode::BAD_REQUEST, err))
            } else if err.contains("Failed to delete account file") {
                tracing::error!("{err}");
                Err((StatusCode::NOT_FOUND, err))
            } else {
                tracing::error!("{err}");
                Err((StatusCode::INTERNAL_SERVER_ERROR, err))
            }
        }
    }
}
