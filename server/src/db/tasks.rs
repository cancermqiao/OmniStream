use shared::{StreamTask, TaskStatus};
use sqlx::Row;
use std::error::Error;

use super::Db;

impl Db {
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
                    Some(json) => serde_json::from_str(&json).unwrap_or_default(),
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

        sqlx::query("UPDATE tasks SET status = ? WHERE id = ?")
            .bind(status_str)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn parse_status(raw: &str) -> TaskStatus {
    match raw {
        "Idle" => TaskStatus::Idle,
        "Recording" => TaskStatus::Recording,
        "Uploading" => TaskStatus::Uploading,
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
        TaskStatus::Completed => "Completed".to_string(),
        TaskStatus::Error(e) => format!("Error:{e}"),
    }
}
