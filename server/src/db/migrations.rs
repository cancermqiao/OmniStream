use sqlx::{Pool, Row, Sqlite};
use std::error::Error;

#[derive(Clone, Copy)]
struct Migration {
    version: i64,
    name: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration { version: 1, name: "create_core_tables" },
    Migration { version: 2, name: "add_tasks_upload_configs" },
    Migration { version: 3, name: "add_download_recording_settings" },
];

pub async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await?;

    for migration in MIGRATIONS {
        let applied: Option<i64> =
            sqlx::query_scalar("SELECT version FROM schema_migrations WHERE version = ?")
                .bind(migration.version)
                .fetch_optional(pool)
                .await?;

        if applied.is_some() {
            continue;
        }

        tracing::info!("Applying database migration {} - {}", migration.version, migration.name);
        apply_migration(pool, *migration).await?;
    }

    Ok(())
}

async fn apply_migration(pool: &Pool<Sqlite>, migration: Migration) -> Result<(), Box<dyn Error>> {
    let mut tx = pool.begin().await?;

    match migration.version {
        1 => {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS tasks (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    url TEXT NOT NULL,
                    status TEXT NOT NULL,
                    filename TEXT NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
                "#,
            )
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS downloads (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    url TEXT NOT NULL,
                    linked_upload_ids TEXT
                );
                "#,
            )
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS uploads (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    config TEXT NOT NULL
                );
                "#,
            )
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS app_settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );
                "#,
            )
            .execute(&mut *tx)
            .await?;
        }
        2 => {
            if !column_exists(&mut tx, "tasks", "upload_configs").await? {
                sqlx::query("ALTER TABLE tasks ADD COLUMN upload_configs TEXT")
                    .execute(&mut *tx)
                    .await?;
            }
        }
        3 => {
            if !column_exists(&mut tx, "downloads", "use_custom_recording_settings").await? {
                sqlx::query(
                    "ALTER TABLE downloads ADD COLUMN use_custom_recording_settings INTEGER DEFAULT 0",
                )
                .execute(&mut *tx)
                .await?;
            }
            if !column_exists(&mut tx, "downloads", "recording_settings").await? {
                sqlx::query("ALTER TABLE downloads ADD COLUMN recording_settings TEXT")
                    .execute(&mut *tx)
                    .await?;
            }
        }
        _ => return Err(format!("unknown migration version: {}", migration.version).into()),
    }

    sqlx::query("INSERT INTO schema_migrations (version, name) VALUES (?, ?)")
        .bind(migration.version)
        .bind(migration.name)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

async fn column_exists(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    table: &str,
    column: &str,
) -> Result<bool, Box<dyn Error>> {
    let pragma = format!("PRAGMA table_info({table})");
    let rows = sqlx::query(&pragma).fetch_all(&mut **tx).await?;
    Ok(rows.into_iter().any(|row| row.get::<String, _>("name") == column))
}

#[cfg(test)]
mod tests {
    use super::run_migrations;
    use sqlx::{
        Executor, Pool, Row, Sqlite,
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    };
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp_db_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("omnistream-{name}-{}.db", Uuid::new_v4()))
    }

    async fn open_pool(path: &PathBuf) -> Pool<Sqlite> {
        let opts = SqliteConnectOptions::new().filename(path).create_if_missing(true);
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .expect("open sqlite pool")
    }

    async fn column_names(pool: &Pool<Sqlite>, table: &str) -> Vec<String> {
        let pragma = format!("PRAGMA table_info({table})");
        let rows = sqlx::query(&pragma).fetch_all(pool).await.expect("fetch table info");
        rows.into_iter().map(|row| row.get("name")).collect()
    }

    #[tokio::test]
    async fn empty_db_initializes_latest_schema() {
        let path = temp_db_path("empty-init");
        let pool = open_pool(&path).await;

        run_migrations(&pool).await.expect("run migrations");

        let versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM schema_migrations ORDER BY version")
                .fetch_all(&pool)
                .await
                .expect("fetch versions");
        assert_eq!(versions, vec![1, 2, 3]);

        let task_columns = column_names(&pool, "tasks").await;
        assert!(task_columns.contains(&"upload_configs".to_string()));

        let download_columns = column_names(&pool, "downloads").await;
        assert!(download_columns.contains(&"use_custom_recording_settings".to_string()));
        assert!(download_columns.contains(&"recording_settings".to_string()));
    }

    #[tokio::test]
    async fn legacy_db_upgrades_missing_columns() {
        let path = temp_db_path("legacy-upgrade");
        let pool = open_pool(&path).await;

        pool.execute(
            r#"
            CREATE TABLE tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                status TEXT NOT NULL,
                filename TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .await
        .expect("create legacy tasks");
        pool.execute(
            r#"
            CREATE TABLE downloads (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                linked_upload_ids TEXT
            );
            "#,
        )
        .await
        .expect("create legacy downloads");
        pool.execute(
            r#"
            CREATE TABLE uploads (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                config TEXT NOT NULL
            );
            "#,
        )
        .await
        .expect("create legacy uploads");
        pool.execute(
            r#"
            CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )
        .await
        .expect("create legacy settings");
        pool.execute(
            r#"
            CREATE TABLE schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .await
        .expect("create migrations table");
        pool.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (1, 'create_core_tables')",
        )
        .await
        .expect("seed migration v1");

        run_migrations(&pool).await.expect("upgrade legacy db");

        let versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM schema_migrations ORDER BY version")
                .fetch_all(&pool)
                .await
                .expect("fetch versions");
        assert_eq!(versions, vec![1, 2, 3]);

        let task_columns = column_names(&pool, "tasks").await;
        assert!(task_columns.contains(&"upload_configs".to_string()));

        let download_columns = column_names(&pool, "downloads").await;
        assert!(download_columns.contains(&"use_custom_recording_settings".to_string()));
        assert!(download_columns.contains(&"recording_settings".to_string()));
    }
}
