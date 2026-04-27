use anyhow::{Result, anyhow};
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use tokio::process::Command;
use url::Url;

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
            return fetch_huya_live_title(url).await;
        }

        let json: Value = serde_json::from_str(&stdout).ok()?;
        let title = extract_title(&json);
        if title.is_some() {
            return title;
        }

        fetch_huya_live_title(url).await
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

async fn fetch_huya_live_title(url: &str) -> Option<String> {
    if !is_huya_url(url) {
        return None;
    }

    let client = Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36",
        )
        .build()
        .ok()?;

    let html = client.get(url).send().await.ok()?.text().await.ok()?;

    extract_json_assignment(&html, "TT_ROOM_DATA")
        .and_then(|value| extract_non_empty_json_string(&value, &["introduction"]))
        .or_else(|| {
            extract_json_assignment(&html, "hyPlayerConfig").and_then(|value| {
                extract_non_empty_json_string(
                    &value,
                    &["stream", "data", "0", "gameLiveInfo", "introduction"],
                )
            })
        })
        .or_else(|| extract_room_title_attr(&html))
}

fn is_huya_url(url: &str) -> bool {
    Url::parse(url)
        .ok()
        .and_then(|v| v.host_str().map(str::to_string))
        .is_some_and(|host| host == "huya.com" || host == "www.huya.com")
}

fn extract_json_assignment(html: &str, variable: &str) -> Option<Value> {
    let pattern = format!(r#"var\s+{}\s*=\s*(\{{.*?\}});"#, regex::escape(variable));
    let regex = Regex::new(&pattern).ok()?;
    let json = regex.captures(html)?.get(1)?.as_str();
    serde_json::from_str(json).ok()
}

fn extract_non_empty_json_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = if let Ok(index) = segment.parse::<usize>() {
            current.get(index)?
        } else {
            current.get(*segment)?
        };
    }

    current.as_str().map(str::trim).filter(|v| !v.is_empty()).map(str::to_string)
}

fn extract_room_title_attr(html: &str) -> Option<String> {
    let regex = Regex::new(r#"id="J_roomTitle"[^>]*title="([^"]+)""#).ok()?;
    Some(regex.captures(html)?.get(1)?.as_str().trim().to_string())
}
