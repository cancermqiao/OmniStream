use shared::DownloadConfig;
use sqlx::Row;
use std::error::Error;

use super::Db;

impl Db {
    pub async fn get_downloads(&self) -> Result<Vec<DownloadConfig>, Box<dyn Error>> {
        let rows = sqlx::query(
            "SELECT id, name, url, linked_upload_ids, use_custom_recording_settings, recording_settings FROM downloads",
        )
            .fetch_all(&self.pool)
            .await?;

        let downloads = rows
            .into_iter()
            .map(|row| {
                let ids_json: Option<String> = row.get("linked_upload_ids");
                let linked_upload_ids: Vec<String> = match ids_json {
                    Some(json) => serde_json::from_str(&json).unwrap_or_default(),
                    None => vec![],
                };
                let use_custom_recording_settings: i64 =
                    row.try_get("use_custom_recording_settings").unwrap_or(0);
                let recording_settings_json: Option<String> =
                    row.try_get("recording_settings").ok();
                let recording_settings = recording_settings_json
                    .as_deref()
                    .and_then(|json| serde_json::from_str(json).ok());

                DownloadConfig {
                    id: row.get("id"),
                    name: row.get("name"),
                    url: row.get("url"),
                    linked_upload_ids,
                    current_status: None,
                    use_custom_recording_settings: use_custom_recording_settings != 0,
                    recording_settings,
                }
            })
            .collect();

        Ok(downloads)
    }

    pub async fn save_download(&self, config: &DownloadConfig) -> Result<(), Box<dyn Error>> {
        let ids_json = serde_json::to_string(&config.linked_upload_ids)?;
        let recording_settings_json =
            config.recording_settings.as_ref().map(serde_json::to_string).transpose()?;
        let use_custom_recording_settings =
            if config.use_custom_recording_settings { 1 } else { 0 };

        sqlx::query(
            r#"
            INSERT INTO downloads (id, name, url, linked_upload_ids, use_custom_recording_settings, recording_settings)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                url = excluded.url,
                linked_upload_ids = excluded.linked_upload_ids,
                use_custom_recording_settings = excluded.use_custom_recording_settings,
                recording_settings = excluded.recording_settings
            "#,
        )
        .bind(&config.id)
        .bind(&config.name)
        .bind(&config.url)
        .bind(ids_json)
        .bind(use_custom_recording_settings)
        .bind(recording_settings_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_download(&self, id: &str) -> Result<(), Box<dyn Error>> {
        sqlx::query("DELETE FROM downloads WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }
}
