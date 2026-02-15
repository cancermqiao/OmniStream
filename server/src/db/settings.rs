use shared::RecordingSettings;
use sqlx::Row;
use std::error::Error;

use super::Db;

const RECORDING_SETTINGS_KEY: &str = "recording_settings";

impl Db {
    pub async fn get_recording_settings(
        &self,
    ) -> Result<Option<RecordingSettings>, Box<dyn Error>> {
        let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?")
            .bind(RECORDING_SETTINGS_KEY)
            .fetch_optional(&self.pool)
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let raw: String = row.get("value");
        let settings = serde_json::from_str::<RecordingSettings>(&raw)?;
        Ok(Some(settings))
    }

    pub async fn save_recording_settings(
        &self,
        settings: &RecordingSettings,
    ) -> Result<(), Box<dyn Error>> {
        let raw = serde_json::to_string(settings)?;
        sqlx::query(
            r#"
            INSERT INTO app_settings (key, value)
            VALUES (?, ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
        )
        .bind(RECORDING_SETTINGS_KEY)
        .bind(raw)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
