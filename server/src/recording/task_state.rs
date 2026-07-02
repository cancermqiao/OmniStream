use shared::TaskStatus;

use crate::state::SharedState;

async fn persist_task_status(state: &SharedState, task_id: &str, status: &TaskStatus) {
    if let Err(e) = state.db.update_status(task_id, status).await {
        tracing::error!(
            "Failed to persist task status, task_id={}, status={:?}: {}",
            task_id,
            status,
            e
        );
    }
}

async fn persist_task_filename(state: &SharedState, task_id: &str, filename: &str) {
    if let Err(e) = state.db.update_filename(task_id, filename).await {
        tracing::error!(
            "Failed to persist task filename, task_id={}, filename={}: {}",
            task_id,
            filename,
            e
        );
    }
}

pub(super) async fn set_task_status(state: &SharedState, task_id: &str, status: TaskStatus) {
    if let Some(mut task) = state.tasks.get_mut(task_id) {
        task.status = status.clone();
    }
    persist_task_status(state, task_id, &status).await;
}

pub(super) async fn set_task_filename(state: &SharedState, task_id: &str, filename: &str) {
    if let Some(mut task) = state.tasks.get_mut(task_id) {
        task.filename = filename.to_string();
    }
    persist_task_filename(state, task_id, filename).await;
}

pub(super) async fn mark_related_errors_completed(state: &SharedState, task_id: &str) {
    let Some(url) = state.tasks.get(task_id).map(|task| task.url.clone()) else {
        return;
    };

    let related_error_task_ids = state
        .tasks
        .iter()
        .filter(|entry| {
            entry.key().as_str() != task_id
                && entry.value().url == url
                && matches!(entry.value().status, TaskStatus::Error(_))
        })
        .map(|entry| entry.key().clone())
        .collect::<Vec<_>>();

    for related_task_id in related_error_task_ids {
        tracing::info!(
            "Task {} upload succeeded, marking previous error task {} as completed",
            task_id,
            related_task_id
        );
        set_task_status(state, &related_task_id, TaskStatus::Completed).await;
    }
}

pub(super) fn clear_task_handle(state: &SharedState, task_id: &str) {
    state.handles.remove(task_id);
}

pub(super) fn resolve_task_name(state: &SharedState, task_id: &str) -> String {
    if let Some(task) = state.tasks.get(task_id) { task.name.clone() } else { task_id.to_string() }
}

pub(super) async fn finish_recording_without_files(
    state: &SharedState,
    task_id: &str,
    terminal_error: Option<String>,
) {
    let error_message = terminal_error.unwrap_or_else(|| "No files generated".to_string());
    tracing::error!("Task {} finished without recordable files: {}", task_id, error_message);
    set_task_status(state, task_id, TaskStatus::Error(error_message)).await;
    clear_task_handle(state, task_id);
}
