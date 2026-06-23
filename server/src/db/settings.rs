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
        match serde_json::from_str::<RecordingSettings>(&raw) {
            Ok(settings) => Ok(Some(settings)),
            Err(e) => {
                tracing::warn!(
                    "Failed to parse app_settings value for key={}: {}",
                    RECORDING_SETTINGS_KEY,
                    e
                );
                Ok(None)
            }
        }
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

#[cfg(test)]
mod tests {
    use super::{Db, RECORDING_SETTINGS_KEY};
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp_db_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("omnistream-db-settings-{name}-{}.db", Uuid::new_v4()))
    }

    #[tokio::test]
    async fn get_recording_settings_returns_none_on_malformed_json() {
        let path = temp_db_path("malformed-settings");
        let db = Db::new(path.to_str().expect("db path")).await.expect("open db");

        sqlx::query("INSERT INTO app_settings (key, value) VALUES (?, ?)")
            .bind(RECORDING_SETTINGS_KEY)
            .bind("{bad-json")
            .execute(&db.pool)
            .await
            .expect("insert malformed settings");

        let settings = db.get_recording_settings().await.expect("read settings");
        assert!(settings.is_none());
    }
}
