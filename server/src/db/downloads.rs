use shared::DownloadConfig;
use sqlx::Row;
use std::error::Error;

use super::Db;

impl Db {
    pub async fn get_downloads(&self) -> Result<Vec<DownloadConfig>, Box<dyn Error>> {
        let rows = sqlx::query(
            "SELECT id, name, url, linked_upload_ids, enabled, use_custom_recording_settings, recording_settings FROM downloads",
        )
            .fetch_all(&self.pool)
            .await?;

        let downloads = rows
            .into_iter()
            .map(|row| {
                let id: String = row.get("id");
                let ids_json: Option<String> = row.get("linked_upload_ids");
                let linked_upload_ids: Vec<String> = match ids_json {
                    Some(json) => match serde_json::from_str(&json) {
                        Ok(ids) => ids,
                        Err(e) => {
                            tracing::warn!(
                                "Failed to parse linked_upload_ids for download_id={}: {}",
                                id,
                                e
                            );
                            vec![]
                        }
                    },
                    None => vec![],
                };
                let enabled: i64 = match row.try_get("enabled") {
                    Ok(value) => value,
                    Err(e) => {
                        tracing::warn!("Failed to read enabled for download_id={}: {}", id, e);
                        1
                    }
                };
                let use_custom_recording_settings: i64 = match row
                    .try_get("use_custom_recording_settings")
                {
                    Ok(value) => value,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to read use_custom_recording_settings for download_id={}: {}",
                            id,
                            e
                        );
                        0
                    }
                };
                let recording_settings_json: Option<String> =
                    match row.try_get("recording_settings") {
                        Ok(value) => value,
                        Err(e) => {
                            tracing::warn!(
                                "Failed to read recording_settings column for download_id={}: {}",
                                id,
                                e
                            );
                            None
                        }
                    };
                let recording_settings =
                    recording_settings_json.as_deref().and_then(|json| match serde_json::from_str(
                        json,
                    ) {
                        Ok(settings) => Some(settings),
                        Err(e) => {
                            tracing::warn!(
                                "Failed to parse recording_settings for download_id={}: {}",
                                id,
                                e
                            );
                            None
                        }
                    });

                DownloadConfig {
                    id,
                    name: row.get("name"),
                    url: row.get("url"),
                    linked_upload_ids,
                    current_status: None,
                    enabled: enabled != 0,
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
            INSERT INTO downloads (id, name, url, linked_upload_ids, enabled, use_custom_recording_settings, recording_settings)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                url = excluded.url,
                linked_upload_ids = excluded.linked_upload_ids,
                enabled = excluded.enabled,
                use_custom_recording_settings = excluded.use_custom_recording_settings,
                recording_settings = excluded.recording_settings
            "#,
        )
        .bind(&config.id)
        .bind(&config.name)
        .bind(&config.url)
        .bind(ids_json)
        .bind(if config.enabled { 1 } else { 0 })
        .bind(use_custom_recording_settings)
        .bind(recording_settings_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn set_download_enabled(
        &self,
        id: &str,
        enabled: bool,
    ) -> Result<(), Box<dyn Error>> {
        sqlx::query("UPDATE downloads SET enabled = ? WHERE id = ?")
            .bind(if enabled { 1 } else { 0 })
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_download(&self, id: &str) -> Result<(), Box<dyn Error>> {
        sqlx::query("DELETE FROM downloads WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Db;
    use sqlx::Executor;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp_db_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("omnistream-db-downloads-{name}-{}.db", Uuid::new_v4()))
    }

    #[tokio::test]
    async fn get_downloads_falls_back_on_malformed_json_fields() {
        let path = temp_db_path("malformed-json");
        let db = Db::new(path.to_str().expect("db path")).await.expect("open db");

        db.pool
            .execute(
                r#"
                INSERT INTO downloads (
                    id, name, url, linked_upload_ids, use_custom_recording_settings, recording_settings
                )
                VALUES ('d1', 'demo', 'https://example.com', 'not-json', 1, '{bad-json')
                "#,
            )
            .await
            .expect("insert broken download row");

        let downloads = db.get_downloads().await.expect("read downloads");
        assert_eq!(downloads.len(), 1);
        assert!(downloads[0].linked_upload_ids.is_empty());
        assert!(downloads[0].recording_settings.is_none());
        assert!(downloads[0].use_custom_recording_settings);
    }
}
