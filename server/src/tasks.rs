use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use shared::{CreateTaskRequest, StreamTask, TaskStatus};
use uuid::Uuid;

use crate::{recording::spawn_recorder, state::SharedState};

pub async fn list_tasks(State(state): State<SharedState>) -> Json<Vec<StreamTask>> {
    if state.tasks.is_empty() {
        match state.db.get_all_tasks().await {
            Ok(tasks) => {
                for task in tasks.iter() {
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

pub async fn add_task(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Json<StreamTask> {
    let task_id = Uuid::new_v4().to_string();
    let filename = format!("{}.mp4", task_id);

    let task = StreamTask {
        id: task_id.clone(),
        name: payload.name,
        url: payload.url.clone(),
        status: TaskStatus::Idle,
        filename: filename.clone(),
        upload_configs: vec![],
    };

    state.tasks.insert(task_id.clone(), task.clone());
    if let Err(e) = state.db.save_task(&task).await {
        tracing::error!("Failed to persist newly created task, task_id={}: {}", task_id, e);
    }
    spawn_recorder(task_id, payload.url, filename, state.clone(), None).await;
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
