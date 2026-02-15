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
                let config_json: String = row.get("config");
                let config = serde_json::from_str(&config_json).unwrap_or_default();

                UploadTemplate { id: row.get("id"), name: row.get("name"), config }
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
