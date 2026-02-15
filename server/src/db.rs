use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::error::Error;
use std::path::Path;

mod downloads;
mod settings;
mod tasks;
mod uploads;

#[derive(Clone)]
pub struct Db {
    pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(database_path: &str) -> Result<Self, Box<dyn Error>> {
        let db_path = Path::new(database_path);
        if let Some(parent) = db_path.parent()
            && !parent.as_os_str().is_empty()
        {
            tokio::fs::create_dir_all(parent).await?;
        }

        let connect_options = SqliteConnectOptions::new().filename(db_path).create_if_missing(true);

        let pool =
            SqlitePoolOptions::new().max_connections(5).connect_with(connect_options).await?;

        let db = Db { pool };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                status TEXT NOT NULL,
                filename TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                upload_configs TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        let _ = sqlx::query("ALTER TABLE tasks ADD COLUMN upload_configs TEXT")
            .execute(&self.pool)
            .await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS downloads (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                linked_upload_ids TEXT,
                use_custom_recording_settings INTEGER DEFAULT 0,
                recording_settings TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        let _ = sqlx::query(
            "ALTER TABLE downloads ADD COLUMN use_custom_recording_settings INTEGER DEFAULT 0",
        )
        .execute(&self.pool)
        .await;
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN recording_settings TEXT")
            .execute(&self.pool)
            .await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS uploads (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                config TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
