use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use shared::{CreateTaskRequest, StreamTask, TaskStatus};

use crate::{
    state::SharedState,
    task_launcher::{LaunchTaskParams, launch_recording_task},
};

pub async fn list_tasks(State(state): State<SharedState>) -> Json<Vec<StreamTask>> {
    if state.tasks.is_empty() {
        match state.db.get_all_tasks().await {
            Ok(mut tasks) => {
                for task in &mut tasks {
                    archive_orphan_active_task(&state, task);
                    state.tasks.insert(task.id.clone(), task.clone());
                }
                return Json(tasks);
            }
            Err(e) => {
                tracing::error!("Failed to load tasks from DB during initial list: {}", e);
            }
        }
    }
    let tasks: Vec<StreamTask> = state.tasks.iter().map(|r| r.value().clone()).collect();
    Json(tasks)
}

fn archive_orphan_active_task(state: &SharedState, task: &mut StreamTask) {
    if !should_archive_orphan_active_task(&task.status, state.handles.contains_key(&task.id)) {
        return;
    }

    tracing::warn!(
        "Archiving orphan active task loaded from DB: task_id={}, name={}, previous_status={:?}",
        task.id,
        task.name,
        task.status
    );
    task.status = TaskStatus::Stopped;
}

fn should_archive_orphan_active_task(status: &TaskStatus, has_handle: bool) -> bool {
    !has_handle && matches!(status, TaskStatus::Recording | TaskStatus::Uploading)
}

pub async fn add_task(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Json<StreamTask> {
    let task = launch_recording_task(
        state.clone(),
        LaunchTaskParams {
            initial_filename: "pending.mp4".to_string(),
            name: payload.name,
            url: payload.url,
            upload_configs: vec![],
            custom_recording_settings: None,
        },
    )
    .await;
    Json(task)
}

pub async fn stop_task(Path(id): Path<String>, State(state): State<SharedState>) -> StatusCode {
    if let Some((_, handle)) = state.handles.remove(&id) {
        handle.abort_handle.abort();
        if let Some(mut task) = state.tasks.get_mut(&id) {
            task.status = TaskStatus::Stopped;
        }
        if let Err(e) = state.db.update_status(&id, &TaskStatus::Stopped).await {
            tracing::error!("Failed to persist stopped task status, task_id={}: {}", id, e);
        }
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

#[cfg(test)]
mod tests {
    use super::should_archive_orphan_active_task;
    use shared::TaskStatus;

    #[test]
    fn archives_active_task_without_runtime_handle() {
        assert!(should_archive_orphan_active_task(&TaskStatus::Recording, false));
        assert!(should_archive_orphan_active_task(&TaskStatus::Uploading, false));
    }

    #[test]
    fn keeps_running_or_terminal_tasks() {
        assert!(!should_archive_orphan_active_task(&TaskStatus::Recording, true));
        assert!(!should_archive_orphan_active_task(&TaskStatus::Completed, false));
        assert!(!should_archive_orphan_active_task(&TaskStatus::Stopped, false));
        assert!(!should_archive_orphan_active_task(
            &TaskStatus::Error("failed".to_string()),
            false
        ));
    }
}
