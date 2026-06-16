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

    pub async fn delete_upload_and_unlink_downloads(&self, id: &str) -> Result<(), Box<dyn Error>> {
        let mut tx = self.pool.begin().await?;
        let rows =
            sqlx::query("SELECT id, linked_upload_ids FROM downloads").fetch_all(&mut *tx).await?;

        for row in rows {
            let download_id: String = row.get("id");
            let ids_json: Option<String> = row.get("linked_upload_ids");
            let Some(ids_json) = ids_json else {
                continue;
            };
            let mut linked_upload_ids: Vec<String> = match serde_json::from_str(&ids_json) {
                Ok(ids) => ids,
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse linked_upload_ids while deleting upload_id={}, download_id={}: {}",
                        id,
                        download_id,
                        e
                    );
                    continue;
                }
            };

            let original_len = linked_upload_ids.len();
            linked_upload_ids.retain(|linked_id| linked_id != id);
            if linked_upload_ids.len() == original_len {
                continue;
            }

            let updated_json = serde_json::to_string(&linked_upload_ids)?;
            sqlx::query("UPDATE downloads SET linked_upload_ids = ? WHERE id = ?")
                .bind(updated_json)
                .bind(&download_id)
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query("DELETE FROM uploads WHERE id = ?").bind(id).execute(&mut *tx).await?;
        tx.commit().await?;
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

    #[tokio::test]
    async fn delete_upload_and_unlink_downloads_removes_references_atomically() {
        let path = temp_db_path("unlink-downloads");
        let db = Db::new(path.to_str().expect("db path")).await.expect("open db");

        db.pool
            .execute(
                r#"
                INSERT INTO uploads (id, name, config)
                VALUES ('u1', 'one', '{}'), ('u2', 'two', '{}')
                "#,
            )
            .await
            .expect("insert uploads");
        db.pool
            .execute(
                r#"
                INSERT INTO downloads (id, name, url, linked_upload_ids)
                VALUES ('d1', 'demo', 'https://example.com', '["u1","u2"]')
                "#,
            )
            .await
            .expect("insert download");

        db.delete_upload_and_unlink_downloads("u1").await.expect("delete upload");

        let upload_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM uploads WHERE id = 'u1'")
            .fetch_one(&db.pool)
            .await
            .expect("count uploads");
        let linked_ids: String =
            sqlx::query_scalar("SELECT linked_upload_ids FROM downloads WHERE id = 'd1'")
                .fetch_one(&db.pool)
                .await
                .expect("read linked ids");

        assert_eq!(upload_count, 0);
        assert_eq!(linked_ids, "[\"u2\"]");
    }
}
