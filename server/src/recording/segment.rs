use std::time::Duration;

use tokio::process::Command;
use tokio::time::{Instant, MissedTickBehavior, interval, sleep};

use super::{RecorderRuntimeConfig, prepare_segment_file, resolve_task_name, stop_segment_process};
use crate::{checker::STREAMLINK_PATH, state::SharedState};

pub(super) enum SegmentLoopAction {
    Continue,
    Stop,
}

pub(super) struct SegmentRecordResult {
    pub(super) filename: String,
    pub(super) limit_reached: bool,
    pub(super) terminal_error: Option<String>,
}

pub(super) async fn record_segment(
    task_id: &str,
    url: &str,
    state: &SharedState,
    runtime: &RecorderRuntimeConfig,
) -> SegmentRecordResult {
    let current_filename = match prepare_segment_file(state, task_id).await {
        Ok(filename) => filename,
        Err(e) => {
            let task_name = resolve_task_name(state, task_id);
            let task_dir = super::recording_task_dir(&task_name);
            tracing::error!(
                "Task {} failed to create recording directory {}: {}",
                task_id,
                task_dir.display(),
                e
            );
            return SegmentRecordResult {
                filename: String::new(),
                limit_reached: false,
                terminal_error: Some(format!("Failed to prepare recording file: {}", e)),
            };
        }
    };

    tracing::info!("Task {} starting segment: {}", task_id, current_filename);

    let mut command = Command::new(STREAMLINK_PATH);
    command.arg("-o").arg(&current_filename).arg(url).arg(&runtime.quality);
    command.kill_on_drop(true);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            let message = format!("Failed to spawn streamlink: {}", e);
            tracing::error!("Task {} {}", task_id, message);
            return SegmentRecordResult {
                filename: current_filename,
                limit_reached: false,
                terminal_error: Some(message),
            };
        }
    };

    let mut limit_reached = false;
    let segment_started_at = Instant::now();
    let mut check_interval = interval(Duration::from_secs(1));
    check_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            status = child.wait() => {
                match status {
                    Ok(s) => tracing::info!("Task {} segment finished with status: {}", task_id, s),
                    Err(e) => tracing::error!("Task {} segment error: {}", task_id, e),
                }
                break;
            }
            _ = check_interval.tick() => {
                if let Some(limit_sec) = runtime.segment_time_sec
                    && segment_started_at.elapsed() >= Duration::from_secs(limit_sec)
                {
                    tracing::info!(
                        "Task {} segment time limit reached: elapsed={}s >= {}s",
                        task_id,
                        segment_started_at.elapsed().as_secs(),
                        limit_sec
                    );
                    limit_reached = true;
                    stop_segment_process(&mut child, task_id, "time split").await;
                    break;
                }

                if let Some(limit) = runtime.segment_size_bytes
                    && let Ok(meta) = tokio::fs::metadata(&current_filename).await
                    && meta.len() > limit
                {
                    tracing::info!("Task {} segment size limit reached: {} > {}", task_id, meta.len(), limit);
                    limit_reached = true;
                    stop_segment_process(&mut child, task_id, "size split").await;
                    break;
                }
            }
        }
    }

    SegmentRecordResult { filename: current_filename, limit_reached, terminal_error: None }
}

pub(super) fn update_recorded_files(
    task_id: &str,
    current_filename: String,
    recorded_files: &mut Vec<String>,
    consecutive_empty_segments: &mut u8,
) {
    if std::path::Path::new(&current_filename).exists() {
        recorded_files.push(current_filename);
        *consecutive_empty_segments = 0;
    } else {
        *consecutive_empty_segments = consecutive_empty_segments.saturating_add(1);
        tracing::warn!(
            "Task {} segment produced no file (consecutive: {})",
            task_id,
            *consecutive_empty_segments
        );
    }
}

pub(super) async fn decide_next_segment_action(
    state: &SharedState,
    task_id: &str,
    url: &str,
    consecutive_empty_segments: u8,
) -> SegmentLoopAction {
    if consecutive_empty_segments >= 3 {
        tracing::error!(
            "Task {} stopped after {} empty segments: stream appears unavailable",
            task_id,
            consecutive_empty_segments
        );
        return SegmentLoopAction::Stop;
    }

    match state.checker.check_live(url).await {
        Ok(true) => {
            tracing::warn!(
                "Task {} streamlink exited but stream is still live, restarting...",
                task_id
            );
            sleep(Duration::from_secs(5)).await;
            SegmentLoopAction::Continue
        }
        _ => {
            tracing::info!("Task {} stream ended or check failed, stopping recorder", task_id);
            SegmentLoopAction::Stop
        }
    }
}
