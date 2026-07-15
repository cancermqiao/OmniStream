use std::{collections::VecDeque, process::Stdio, sync::Arc, time::Duration};

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::{Instant, MissedTickBehavior, interval, sleep};

use super::{RecorderRuntimeConfig, prepare_segment_file, resolve_task_name, stop_segment_process};
use crate::{
    checker::STREAMLINK_PATH, platform::resolve_stream, state::SharedState,
    storage_guard::recording_storage_below_min_free_percent,
};

const FFMPEG_PATH: &str = "ffmpeg";
const RECORDER_OUTPUT_LINES: usize = 20;
const STORAGE_GUARD_CHECK_INTERVAL: Duration = Duration::from_secs(30);

type RecorderOutputBuffer = Arc<Mutex<VecDeque<String>>>;

pub(super) enum SegmentLoopAction {
    Continue,
    Stop,
}

pub(super) struct SegmentRecordResult {
    pub(super) filename: String,
    pub(super) limit_reached: bool,
    pub(super) terminal_error: Option<String>,
    pub(super) disk_full: bool,
    pub(super) storage_guard_triggered: bool,
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
                disk_full: is_disk_full_error(&e),
                storage_guard_triggered: false,
            };
        }
    };

    tracing::info!("Task {} starting segment: {}", task_id, current_filename);

    let recorder = match resolve_stream(url, &runtime.quality).await {
        Ok(Some(stream)) => {
            tracing::info!(
                "Task {} resolved platform stream: original={}, resolved={}",
                task_id,
                url,
                stream.input_url
            );
            if stream.direct_input {
                RecorderCommand::Ffmpeg { input_url: stream.input_url }
            } else {
                RecorderCommand::Streamlink {
                    input_url: stream.input_url,
                    quality: "best".to_string(),
                }
            }
        }
        Ok(None) => RecorderCommand::Streamlink {
            input_url: url.to_string(),
            quality: runtime.quality.clone(),
        },
        Err(e) => {
            tracing::warn!(
                "Task {} failed to resolve platform stream, falling back to original URL: {}",
                task_id,
                e
            );
            RecorderCommand::Streamlink {
                input_url: url.to_string(),
                quality: runtime.quality.clone(),
            }
        }
    };

    let (mut command, recorder_name) = build_recorder_command(&recorder, &current_filename);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            let message = format!("Failed to spawn {recorder_name}: {e}");
            tracing::error!("Task {} {}", task_id, message);
            return SegmentRecordResult {
                filename: current_filename,
                limit_reached: false,
                terminal_error: Some(message),
                disk_full: is_disk_full_error(&e),
                storage_guard_triggered: false,
            };
        }
    };
    let recorder_output = Arc::new(Mutex::new(VecDeque::with_capacity(RECORDER_OUTPUT_LINES)));
    spawn_recorder_output_collector(
        child.stdout.take(),
        task_id.to_string(),
        recorder_name,
        "stdout",
        false,
        recorder_output.clone(),
    );
    spawn_recorder_output_collector(
        child.stderr.take(),
        task_id.to_string(),
        recorder_name,
        "stderr",
        true,
        recorder_output.clone(),
    );

    let mut limit_reached = false;
    let mut storage_guard_triggered = false;
    let mut recorder_error = None;
    let segment_started_at = Instant::now();
    let mut last_storage_guard_check: Option<Instant> = None;
    let mut check_interval = interval(Duration::from_secs(1));
    check_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            status = child.wait() => {
                match status {
                    Ok(s) if s.success() => {
                        tracing::info!("Task {} {} segment finished with status: {}", task_id, recorder_name, s);
                    }
                    Ok(s) => {
                        let recent_output = recent_recorder_output(&recorder_output).await;
                        let message = format!(
                            "{} exited unsuccessfully: status={}, input={}, quality={}, output={}, recent_output={}",
                            recorder_name,
                            s,
                            recorder.input_url(),
                            recorder.quality_label(),
                            current_filename,
                            recent_output.unwrap_or_else(|| "<empty>".to_string())
                        );
                        tracing::error!("Task {} {}", task_id, message);
                        recorder_error = Some(message);
                    }
                    Err(e) => {
                        let recent_output = recent_recorder_output(&recorder_output).await;
                        let message = format!(
                            "{} segment wait failed: {}, input={}, quality={}, output={}, recent_output={}",
                            recorder_name,
                            e,
                            recorder.input_url(),
                            recorder.quality_label(),
                            current_filename,
                            recent_output.unwrap_or_else(|| "<empty>".to_string())
                        );
                        tracing::error!("Task {} {}", task_id, message);
                        recorder_error = Some(message);
                    }
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

                if last_storage_guard_check
                    .map(|last| last.elapsed() >= STORAGE_GUARD_CHECK_INTERVAL)
                    .unwrap_or(true)
                {
                    last_storage_guard_check = Some(Instant::now());
                    match recording_storage_below_min_free_percent().await {
                        Ok(Some(snapshot)) => {
                            tracing::warn!(
                                "Task {} stopping current segment because recording storage is below 2% free: path={}, available_kb={}, total_kb={}, free_percent={:.2}",
                                task_id,
                                snapshot.path.display(),
                                snapshot.available_kb,
                                snapshot.total_kb,
                                snapshot.free_percent
                            );
                            storage_guard_triggered = true;
                            stop_segment_process(&mut child, task_id, "storage guard").await;
                            break;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            tracing::warn!(
                                "Task {} failed to check recording storage during segment: {}",
                                task_id,
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    let disk_full = recorder_error.as_deref().is_some_and(is_disk_full_message);
    if disk_full {
        tracing::error!(
            "Task {} detected disk full while recording {}, stopping recorder loop and starting upload for completed files",
            task_id,
            current_filename
        );
    }

    let terminal_error = if !limit_reached
        && !storage_guard_triggered
        && recorder_error.is_some()
        && !recorded_file_has_content(&current_filename).await
    {
        recorder_error
    } else {
        None
    };

    SegmentRecordResult {
        filename: current_filename,
        limit_reached,
        terminal_error,
        disk_full,
        storage_guard_triggered,
    }
}

fn is_disk_full_error(error: &std::io::Error) -> bool {
    error.raw_os_error() == Some(28) || is_disk_full_message(&error.to_string())
}

fn is_disk_full_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("no space left on device")
        || lower.contains("errno 28")
        || lower.contains("enospc")
        || lower.contains("disk full")
}

enum RecorderCommand {
    Streamlink { input_url: String, quality: String },
    Ffmpeg { input_url: String },
}

impl RecorderCommand {
    fn input_url(&self) -> &str {
        match self {
            RecorderCommand::Streamlink { input_url, .. }
            | RecorderCommand::Ffmpeg { input_url } => input_url,
        }
    }

    fn quality_label(&self) -> &str {
        match self {
            RecorderCommand::Streamlink { quality, .. } => quality,
            RecorderCommand::Ffmpeg { .. } => "copy",
        }
    }
}

fn build_recorder_command(recorder: &RecorderCommand, output: &str) -> (Command, &'static str) {
    match recorder {
        RecorderCommand::Streamlink { input_url, quality } => {
            let mut command = Command::new(STREAMLINK_PATH);
            command.arg("-o").arg(output).arg(input_url).arg(quality);
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
            command.kill_on_drop(true);
            (command, STREAMLINK_PATH)
        }
        RecorderCommand::Ffmpeg { input_url } => {
            let mut command = Command::new(FFMPEG_PATH);
            command
                .arg("-hide_banner")
                .arg("-loglevel")
                .arg("warning")
                .arg("-y")
                .arg("-reconnect")
                .arg("1")
                .arg("-reconnect_streamed")
                .arg("1")
                .arg("-reconnect_delay_max")
                .arg("5")
                .arg("-headers")
                .arg(ffmpeg_headers_for_input(input_url))
                .arg("-i")
                .arg(input_url)
                .arg("-c")
                .arg("copy")
                .arg("-bsf:a")
                .arg("aac_adtstoasc")
                .arg("-movflags")
                .arg("frag_keyframe+empty_moov")
                .arg("-f")
                .arg("mp4")
                .arg(output);
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
            command.kill_on_drop(true);
            (command, FFMPEG_PATH)
        }
    }
}

fn ffmpeg_headers_for_input(input_url: &str) -> &'static str {
    if is_bilibili_cdn_url(input_url) {
        "Referer: https://live.bilibili.com/\r\nUser-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36\r\n"
    } else {
        "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36\r\n"
    }
}

fn is_bilibili_cdn_url(input_url: &str) -> bool {
    input_url.contains("bilivideo.com")
        || input_url.contains("bilivideo.cn")
        || input_url.contains("bilibili.com")
        || input_url.contains("biliapi.net")
}

fn spawn_recorder_output_collector<R>(
    reader: Option<R>,
    task_id: String,
    recorder_name: &'static str,
    stream_name: &'static str,
    log_lines: bool,
    output: RecorderOutputBuffer,
) where
    R: AsyncRead + Unpin + Send + 'static,
{
    let Some(reader) = reader else {
        return;
    };

    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }
                    {
                        let mut output = output.lock().await;
                        if output.len() == RECORDER_OUTPUT_LINES {
                            output.pop_front();
                        }
                        output.push_back(format!("{stream_name}: {trimmed}"));
                    }
                    if log_lines {
                        tracing::warn!(
                            "Task {} {} {}: {}",
                            task_id,
                            recorder_name,
                            stream_name,
                            trimmed
                        );
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    tracing::warn!(
                        "Task {} failed to read {} {}: {}",
                        task_id,
                        recorder_name,
                        stream_name,
                        e
                    );
                    break;
                }
            }
        }
    });
}

async fn recent_recorder_output(output: &RecorderOutputBuffer) -> Option<String> {
    let output = output.lock().await;
    if output.is_empty() {
        None
    } else {
        Some(output.iter().cloned().collect::<Vec<_>>().join(" | "))
    }
}

async fn recorded_file_has_content(path: &str) -> bool {
    tokio::fs::metadata(path).await.map(|meta| meta.len() > 0).unwrap_or(false)
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
        Ok(false) => {
            tracing::info!("Task {} stream ended, stopping recorder", task_id);
            SegmentLoopAction::Stop
        }
        Err(e) => {
            tracing::error!(
                "Task {} failed to check live status after recorder exit, url={}: {}",
                task_id,
                url,
                e
            );
            SegmentLoopAction::Stop
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ffmpeg_headers_for_input, is_bilibili_cdn_url, is_disk_full_message};

    #[test]
    fn ffmpeg_headers_include_bilibili_referer_for_bilibili_cdn() {
        let headers = ffmpeg_headers_for_input("https://d1--ov-gotcha05.bilivideo.com/live.flv");

        assert!(is_bilibili_cdn_url("https://d1--ov-gotcha05.bilivideo.com/live.flv"));
        assert!(headers.contains("Referer: https://live.bilibili.com/"));
        assert!(headers.contains("User-Agent: Mozilla/5.0"));
    }

    #[test]
    fn ffmpeg_headers_keep_generic_user_agent_for_other_inputs() {
        let headers = ffmpeg_headers_for_input("https://example.com/live.m3u8");

        assert!(!headers.contains("Referer: https://live.bilibili.com/"));
        assert!(headers.contains("User-Agent: Mozilla/5.0"));
    }

    #[test]
    fn disk_full_detector_recognizes_common_recorder_messages() {
        assert!(is_disk_full_message("OSError: [Errno 28] No space left on device"));
        assert!(is_disk_full_message("write failed: ENOSPC"));
        assert!(is_disk_full_message("fatal error: disk full"));
        assert!(!is_disk_full_message("network timeout"));
    }
}
