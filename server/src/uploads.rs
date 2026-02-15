use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use shared::UploadTemplate;
use uuid::Uuid;

use crate::state::SharedState;

pub async fn list_uploads(State(state): State<SharedState>) -> Json<Vec<UploadTemplate>> {
    match state.db.get_uploads().await {
        Ok(uploads) => Json(uploads),
        Err(e) => {
            tracing::error!("Failed to list uploads: {}", e);
            Json(vec![])
        }
    }
}

pub async fn add_upload(
    State(state): State<SharedState>,
    Json(payload): Json<UploadTemplate>,
) -> StatusCode {
    let mut template = payload;
    if template.id.is_empty() {
        template.id = Uuid::new_v4().to_string();
    }
    if let Err(e) = state.db.save_upload(&template).await {
        tracing::error!("Failed to save upload: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

pub async fn delete_upload(Path(id): Path<String>, State(state): State<SharedState>) -> StatusCode {
    if let Err(e) = state.db.delete_upload(&id).await {
        tracing::error!("Failed to delete upload: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}
