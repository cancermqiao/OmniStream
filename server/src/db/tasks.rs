use shared::{StreamTask, TaskStatus};
use sqlx::Row;
use std::error::Error;
use std::io;

use super::Db;

impl Db {
    pub async fn save_task(&self, task: &StreamTask) -> Result<(), Box<dyn Error>> {
        let upload_configs_json = serde_json::to_string(&task.upload_configs)?;
        let status_str = stringify_status(&task.status);

        sqlx::query(
            r#"
            INSERT INTO tasks (id, name, url, status, filename, upload_configs)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                url = excluded.url,
                status = excluded.status,
                filename = excluded.filename,
                upload_configs = excluded.upload_configs
            "#,
        )
        .bind(&task.id)
        .bind(&task.name)
        .bind(&task.url)
        .bind(status_str)
        .bind(&task.filename)
        .bind(upload_configs_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<StreamTask>, Box<dyn Error>> {
        let rows = sqlx::query("SELECT id, name, url, status, filename, upload_configs FROM tasks")
            .fetch_all(&self.pool)
            .await?;

        let tasks = rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("status");
                let status = parse_status(&status_str);

                let upload_configs_json: Option<String> = row.get("upload_configs");
                let upload_configs = match upload_configs_json {
                    Some(json) => match serde_json::from_str(&json) {
                        Ok(configs) => configs,
                        Err(e) => {
                            let task_id: String = row.get("id");
                            tracing::warn!(
                                "Failed to parse upload_configs for task_id={}: {}",
                                task_id,
                                e
                            );
                            vec![]
                        }
                    },
                    None => vec![],
                };

                StreamTask {
                    id: row.get("id"),
                    name: row.get("name"),
                    url: row.get("url"),
                    status,
                    filename: row.get("filename"),
                    upload_configs,
                }
            })
            .collect();

        Ok(tasks)
    }

    pub async fn update_status(&self, id: &str, status: &TaskStatus) -> Result<(), Box<dyn Error>> {
        let status_str = stringify_status(status);

        let result = sqlx::query("UPDATE tasks SET status = ? WHERE id = ?")
            .bind(status_str)
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("task not found for status update: {id}"),
            )
            .into());
        }
        Ok(())
    }

    pub async fn update_filename(&self, id: &str, filename: &str) -> Result<(), Box<dyn Error>> {
        let result = sqlx::query("UPDATE tasks SET filename = ? WHERE id = ?")
            .bind(filename)
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("task not found for filename update: {id}"),
            )
            .into());
        }
        Ok(())
    }
}

fn parse_status(raw: &str) -> TaskStatus {
    match raw {
        "Idle" => TaskStatus::Idle,
        "Recording" => TaskStatus::Recording,
        "Uploading" => TaskStatus::Uploading,
        "Stopped" => TaskStatus::Stopped,
        "Completed" => TaskStatus::Completed,
        s if s.starts_with("Error:") => TaskStatus::Error(s[6..].to_string()),
        _ => TaskStatus::Idle,
    }
}

fn stringify_status(status: &TaskStatus) -> String {
    match status {
        TaskStatus::Idle => "Idle".to_string(),
        TaskStatus::Recording => "Recording".to_string(),
        TaskStatus::Uploading => "Uploading".to_string(),
        TaskStatus::Stopped => "Stopped".to_string(),
        TaskStatus::Completed => "Completed".to_string(),
        TaskStatus::Error(e) => format!("Error:{e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_status, stringify_status};
    use shared::TaskStatus;

    #[test]
    fn stopped_status_round_trips() {
        assert_eq!(parse_status("Stopped"), TaskStatus::Stopped);
        assert_eq!(stringify_status(&TaskStatus::Stopped), "Stopped");
    }

    #[test]
    fn error_status_round_trips() {
        assert_eq!(parse_status("Error:boom"), TaskStatus::Error("boom".to_string()));
        assert_eq!(stringify_status(&TaskStatus::Error("boom".to_string())), "Error:boom");
    }

    #[test]
    fn unknown_status_falls_back_to_idle() {
        assert_eq!(parse_status("mystery"), TaskStatus::Idle);
    }
}
