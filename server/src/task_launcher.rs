use shared::{StreamTask, TaskStatus, UploadConfig};
use uuid::Uuid;

use crate::{recording::spawn_recorder, state::SharedState};

pub struct LaunchTaskParams {
    pub name: String,
    pub url: String,
    pub initial_filename: String,
    pub upload_configs: Vec<UploadConfig>,
    pub custom_recording_settings: Option<shared::RecordingSettings>,
}

fn build_stream_task(task_id: String, params: &LaunchTaskParams) -> StreamTask {
    StreamTask {
        id: task_id,
        name: params.name.clone(),
        url: params.url.clone(),
        status: TaskStatus::Idle,
        filename: params.initial_filename.clone(),
        upload_configs: params.upload_configs.clone(),
    }
}

pub async fn launch_recording_task(
    state: SharedState,
    params: LaunchTaskParams,
) -> StreamTask {
    let task_id = Uuid::new_v4().to_string();
    let task = build_stream_task(task_id.clone(), &params);

    state.tasks.insert(task_id.clone(), task.clone());
    if let Err(e) = state.db.save_task(&task).await {
        tracing::error!("Failed to persist launched task, task_id={}: {}", task_id, e);
    }

    spawn_recorder(
        task_id,
        params.url,
        params.initial_filename,
        state,
        params.custom_recording_settings,
    )
    .await;

    task
}

#[cfg(test)]
mod tests {
    use super::{LaunchTaskParams, build_stream_task};
    use shared::{TaskStatus, UploadConfig};

    #[test]
    fn build_stream_task_sets_expected_defaults_and_fields() {
        let params = LaunchTaskParams {
            name: "主播任务".to_string(),
            url: "https://www.huya.com/211888".to_string(),
            initial_filename: "pending.mp4".to_string(),
            upload_configs: vec![],
            custom_recording_settings: None,
        };

        let task = build_stream_task("task-1".to_string(), &params);

        assert_eq!(task.id, "task-1");
        assert_eq!(task.name, "主播任务");
        assert_eq!(task.url, "https://www.huya.com/211888");
        assert_eq!(task.filename, "pending.mp4");
        assert_eq!(task.status, TaskStatus::Idle);
        assert!(task.upload_configs.is_empty());
    }

    #[test]
    fn build_stream_task_preserves_upload_configs() {
        let params = LaunchTaskParams {
            name: "上传任务".to_string(),
            url: "https://www.douyu.com/9999".to_string(),
            initial_filename: "segment-1.mp4".to_string(),
            upload_configs: vec![
                UploadConfig { title: Some("A".to_string()), ..Default::default() },
                UploadConfig { title: Some("B".to_string()), ..Default::default() },
            ],
            custom_recording_settings: None,
        };

        let task = build_stream_task("task-2".to_string(), &params);

        assert_eq!(task.upload_configs.len(), 2);
        assert_eq!(task.upload_configs[0].title.as_deref(), Some("A"));
        assert_eq!(task.upload_configs[1].title.as_deref(), Some("B"));
    }
}
