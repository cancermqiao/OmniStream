use anyhow::{Result, anyhow};
use serde_json::Value;
use tokio::process::Command;

pub const STREAMLINK_PATH: &str = "streamlink";

#[derive(Clone)]
pub struct StreamlinkChecker;

impl StreamlinkChecker {
    pub fn new() -> Self {
        Self
    }

    pub async fn check_live(&self, url: &str) -> Result<bool> {
        // streamlink --json <url>
        let output = Command::new(STREAMLINK_PATH).arg("--json").arg(url).output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if stdout.trim().is_empty() {
            if output.status.success() {
                return Ok(false);
            }
            return classify_streamlink_error(stderr.trim());
        }

        // Parse JSON
        let json: Value = serde_json::from_str(&stdout)
            .map_err(|e| anyhow!("Failed to parse streamlink output: {}", e))?;

        // Check if "streams" is present and not empty
        // Structure is usually: { "streams": { "best": ... }, ... } or { "error": ... }
        if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
            return classify_streamlink_error(error);
        }

        if let Some(streams) = json.get("streams")
            && streams.as_object().map(|stream_map| !stream_map.is_empty()).unwrap_or(false)
        {
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn fetch_live_title(&self, url: &str) -> Option<String> {
        let output = Command::new(STREAMLINK_PATH).arg("--json").arg(url).output().await.ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return None;
        }

        let json: Value = serde_json::from_str(&stdout).ok()?;
        extract_title(&json)
    }
}

fn extract_title(json: &Value) -> Option<String> {
    let metadata_title = json
        .get("metadata")
        .and_then(|v| v.get("title"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    if metadata_title.is_some() {
        return metadata_title;
    }

    json.get("title")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn classify_streamlink_error(message: &str) -> Result<bool> {
    let msg = message.to_lowercase();
    // 常见“主播未开播/无可用流”场景，按离线处理
    if msg.contains("no playable streams found")
        || msg.contains("no streams found")
        || msg.contains("is offline")
    {
        return Ok(false);
    }
    // 其他情况视为异常，交给上层记录告警，便于排查网络/解析问题
    Err(anyhow!("Streamlink check failed: {}", message))
}
