use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::error::Error;
use std::path::Path;

mod downloads;
mod migrations;
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

        migrations::run_migrations(&pool).await?;
        let db = Db { pool };
        Ok(db)
    }
}
