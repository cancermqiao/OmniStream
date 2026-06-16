use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use shared::UploadTemplate;
use uuid::Uuid;

use crate::{accounts::storage, state::SharedState};

const MAX_UPLOAD_TITLE_TEMPLATE_CHARS: usize = 80;
const MAX_UPLOAD_DESCRIPTION_CHARS: usize = 2_000;
const MAX_UPLOAD_DYNAMIC_CHARS: usize = 233;
const MAX_UPLOAD_TAGS: usize = 12;
const MAX_UPLOAD_TAG_CHARS: usize = 20;

pub async fn list_uploads(
    State(state): State<SharedState>,
) -> (StatusCode, Json<Vec<UploadTemplate>>) {
    match state.db.get_uploads().await {
        Ok(uploads) => (StatusCode::OK, Json(uploads)),
        Err(e) => {
            tracing::error!("Failed to list uploads: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

pub async fn add_upload(
    State(state): State<SharedState>,
    Json(payload): Json<UploadTemplate>,
) -> (StatusCode, String) {
    let mut template = payload;
    if template.id.is_empty() {
        template.id = Uuid::new_v4().to_string();
    }
    normalize_upload_template(&mut template);

    if let Err(message) = validate_upload_template(&template).await {
        tracing::warn!("Rejected upload template update: {}", message);
        return (StatusCode::BAD_REQUEST, message);
    }

    if let Err(e) = state.db.save_upload(&template).await {
        tracing::error!("Failed to save upload: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "failed to save upload template".to_string());
    }
    (StatusCode::OK, String::new())
}

fn normalize_upload_template(template: &mut UploadTemplate) {
    template.name = template.name.trim().to_string();
    template.config.account_file = template.config.account_file.trim().to_string();
    template.config.description = template.config.description.trim().to_string();
    template.config.dynamic = template.config.dynamic.trim().to_string();
    template.config.title = template
        .config
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_string);
    template.config.tags = template
        .config
        .tags
        .iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();
}

async fn validate_upload_template(template: &UploadTemplate) -> Result<(), String> {
    validate_upload_template_shape(template)?;
    if !storage::account_file_exists(&template.config.account_file).await {
        return Err(format!("account_file does not exist: {}", template.config.account_file));
    }
    Ok(())
}

fn validate_upload_template_shape(template: &UploadTemplate) -> Result<(), String> {
    if template.name.is_empty() {
        return Err("upload template name is required".to_string());
    }
    if template.config.account_file.is_empty() {
        return Err("account_file is required".to_string());
    }
    if template.config.tid == 0 {
        return Err("tid is required".to_string());
    }
    if template.config.copyright != 1 && template.config.copyright != 2 {
        return Err("copyright must be 1 (Original) or 2 (Reprint)".to_string());
    }
    if let Some(title) = &template.config.title
        && title.chars().count() > MAX_UPLOAD_TITLE_TEMPLATE_CHARS
    {
        return Err(format!(
            "title template exceeds {} characters",
            MAX_UPLOAD_TITLE_TEMPLATE_CHARS
        ));
    }
    if template.config.description.chars().count() > MAX_UPLOAD_DESCRIPTION_CHARS {
        return Err(format!("description exceeds {} characters", MAX_UPLOAD_DESCRIPTION_CHARS));
    }
    if template.config.dynamic.chars().count() > MAX_UPLOAD_DYNAMIC_CHARS {
        return Err(format!("dynamic exceeds {} characters", MAX_UPLOAD_DYNAMIC_CHARS));
    }
    if template.config.tags.len() > MAX_UPLOAD_TAGS {
        return Err(format!("tags exceed maximum count {}", MAX_UPLOAD_TAGS));
    }
    if let Some(tag) =
        template.config.tags.iter().find(|tag| tag.chars().count() > MAX_UPLOAD_TAG_CHARS)
    {
        return Err(format!("tag exceeds {} characters: {}", MAX_UPLOAD_TAG_CHARS, tag));
    }
    Ok(())
}

pub async fn delete_upload(Path(id): Path<String>, State(state): State<SharedState>) -> StatusCode {
    if let Err(e) = state.db.delete_upload_and_unlink_downloads(&id).await {
        tracing::error!("Failed to delete upload: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::{normalize_upload_template, validate_upload_template_shape};
    use shared::{UploadConfig, UploadTemplate};

    fn valid_template() -> UploadTemplate {
        UploadTemplate {
            id: "u1".to_string(),
            name: "default".to_string(),
            config: UploadConfig { account_file: "cookies.json".to_string(), ..Default::default() },
        }
    }

    #[test]
    fn normalize_upload_template_trims_optional_fields() {
        let mut template = valid_template();
        template.name = "  template  ".to_string();
        template.config.account_file = "  cookies.json  ".to_string();
        template.config.title = Some("  {title}  ".to_string());
        template.config.tags = vec!["  game  ".to_string(), " ".to_string()];

        normalize_upload_template(&mut template);

        assert_eq!(template.name, "template");
        assert_eq!(template.config.account_file, "cookies.json");
        assert_eq!(template.config.title.as_deref(), Some("{title}"));
        assert_eq!(template.config.tags, vec!["game"]);
    }

    #[test]
    fn validate_upload_template_shape_rejects_missing_required_fields() {
        let mut template = valid_template();
        template.name.clear();
        assert!(validate_upload_template_shape(&template).is_err());

        let mut template = valid_template();
        template.config.account_file.clear();
        assert!(validate_upload_template_shape(&template).is_err());
    }

    #[test]
    fn validate_upload_template_shape_rejects_invalid_limits() {
        let mut template = valid_template();
        template.config.copyright = 3;
        assert!(validate_upload_template_shape(&template).is_err());

        let mut template = valid_template();
        template.config.tags = vec!["x".repeat(21)];
        assert!(validate_upload_template_shape(&template).is_err());
    }
}
