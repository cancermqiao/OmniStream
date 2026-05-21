use shared::UploadTemplate;
use sqlx::Row;
use std::error::Error;

use super::Db;

impl Db {
    pub async fn get_uploads(&self) -> Result<Vec<UploadTemplate>, Box<dyn Error>> {
        let rows =
            sqlx::query("SELECT id, name, config FROM uploads").fetch_all(&self.pool).await?;

        let uploads = rows
            .into_iter()
            .map(|row| {
                let id: String = row.get("id");
                let config_json: String = row.get("config");
                let config = match serde_json::from_str(&config_json) {
                    Ok(config) => config,
                    Err(e) => {
                        tracing::warn!("Failed to parse upload config for upload_id={}: {}", id, e);
                        Default::default()
                    }
                };

                UploadTemplate { id, name: row.get("name"), config }
            })
            .collect();

        Ok(uploads)
    }

    pub async fn save_upload(&self, template: &UploadTemplate) -> Result<(), Box<dyn Error>> {
        let config_json = serde_json::to_string(&template.config)?;

        sqlx::query(
            r#"
            INSERT INTO uploads (id, name, config)
            VALUES (?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                config = excluded.config
            "#,
        )
        .bind(&template.id)
        .bind(&template.name)
        .bind(config_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_upload(&self, id: &str) -> Result<(), Box<dyn Error>> {
        sqlx::query("DELETE FROM uploads WHERE id = ?").bind(id).execute(&self.pool).await?;
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
        std::env::temp_dir().join(format!("omnistream-db-uploads-{name}-{}.db", Uuid::new_v4()))
    }

    #[tokio::test]
    async fn get_uploads_falls_back_on_malformed_config_json() {
        let path = temp_db_path("malformed-config");
        let db = Db::new(path.to_str().expect("db path")).await.expect("open db");

        db.pool
            .execute(
                r#"
                INSERT INTO uploads (id, name, config)
                VALUES ('u1', 'broken', '{bad-json')
                "#,
            )
            .await
            .expect("insert broken upload row");

        let uploads = db.get_uploads().await.expect("read uploads");
        assert_eq!(uploads.len(), 1);
        assert_eq!(uploads[0].id, "u1");
        assert_eq!(uploads[0].name, "broken");
        assert_eq!(uploads[0].config, Default::default());
    }
}
