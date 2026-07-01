use anyhow::{Result, anyhow};
use md5::{Digest, Md5};
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LivePlatform {
    Bilibili,
    Douyu,
    Huya,
    Tiktok,
    Douyin,
    Twitch,
    Youtube,
    Kick,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ResolvedStream {
    pub input_url: String,
    pub title: Option<String>,
    pub direct_input: bool,
}

pub async fn resolve_stream(url: &str, quality: &str) -> Result<Option<ResolvedStream>> {
    match detect_platform(url) {
        LivePlatform::Douyu => resolve_douyu(url).await.map(Some),
        LivePlatform::Douyin => resolve_douyin(url, quality).await.map(Some),
        _ => Ok(None),
    }
}

pub fn detect_platform(url: &str) -> LivePlatform {
    let Some(host) =
        Url::parse(url).ok().and_then(|url| url.host_str().map(|host| host.to_ascii_lowercase()))
    else {
        return LivePlatform::Unknown;
    };

    if host == "b23.tv" || host.ends_with("bilibili.com") {
        return LivePlatform::Bilibili;
    }
    if host.ends_with("douyu.com") {
        return LivePlatform::Douyu;
    }
    if host.ends_with("huya.com") {
        return LivePlatform::Huya;
    }
    if host.ends_with("tiktok.com") {
        return LivePlatform::Tiktok;
    }
    if host.ends_with("douyin.com") {
        return LivePlatform::Douyin;
    }
    if host.ends_with("twitch.tv") {
        return LivePlatform::Twitch;
    }
    if host.ends_with("youtube.com") || host == "youtu.be" {
        return LivePlatform::Youtube;
    }
    if host.ends_with("kick.com") {
        return LivePlatform::Kick;
    }

    LivePlatform::Unknown
}

async fn resolve_douyu(url: &str) -> Result<ResolvedStream> {
    let room_id = extract_douyu_room_id(url).await?;
    let client = Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36",
        )
        .build()?;

    let room_info: Value = client
        .get(format!("https://www.douyu.com/betard/{room_id}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros();
    let mut hasher = Md5::new();
    hasher.update(format!("{room_id}{timestamp}"));
    let auth = hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let preview: Value = client
        .post(format!("https://playweb.douyucdn.cn/lapi/live/hlsH5Preview/{room_id}"))
        .header("rid", &room_id)
        .header("time", timestamp.to_string())
        .header("auth", auth)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(format!("did={DOUYU_DID}&rid={room_id}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if preview.get("error").and_then(Value::as_i64) != Some(0) {
        return Err(anyhow!("Douyu preview API failed: {preview}"));
    }

    let rtmp_url = non_empty_json_string(&preview, &["data", "rtmp_url"])
        .ok_or_else(|| anyhow!("Douyu preview API missing rtmp_url"))?;
    let rtmp_live = non_empty_json_string(&preview, &["data", "rtmp_live"])
        .ok_or_else(|| anyhow!("Douyu preview API missing rtmp_live"))?;
    let input_url =
        format!("{}/{}", rtmp_url.trim_end_matches('/'), rtmp_live.trim_start_matches('/'));

    Ok(ResolvedStream {
        input_url,
        title: non_empty_json_string(&room_info, &["room", "room_name"]),
        direct_input: true,
    })
}

const DOUYU_DID: &str = "10000000000000000000000000001501";

async fn extract_douyu_room_id(url: &str) -> Result<String> {
    if let Some(room_id) = Url::parse(url)
        .ok()
        .and_then(|url| url.path_segments()?.next_back().map(str::to_string))
        .filter(|v| v.chars().all(|c| c.is_ascii_digit()))
    {
        return Ok(room_id);
    }

    let html = Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36",
        )
        .build()?
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let patterns = [
        r"\$ROOM\.room_id\s*=\s*(\d+)",
        r"room_id\s*=\s*(\d+)",
        r#""room_id.?":(\d+)"#,
        r"data-onlineid=(\d+)",
    ];
    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .find_map(|regex| regex.captures(&html))
        .and_then(|captures| captures.get(1).map(|v| v.as_str().to_string()))
        .ok_or_else(|| anyhow!("Douyu room id not found in URL or page HTML"))
}

async fn resolve_douyin(url: &str, quality: &str) -> Result<ResolvedStream> {
    let html = Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36",
        )
        .build()?
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let streams = extract_douyin_hls_streams(&html);
    if streams.is_empty() {
        return Err(anyhow!("Douyin page does not contain playable HLS streams"));
    }

    let input_url = select_douyin_stream(&streams, quality)
        .or_else(|| streams.first().map(|(_, stream)| stream.clone()))
        .ok_or_else(|| anyhow!("Douyin page does not contain playable HLS streams"))?;

    Ok(ResolvedStream { input_url, title: extract_douyin_title(&html), direct_input: true })
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

fn non_empty_json_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor.as_str().and_then(|v| non_empty(v.to_string()))
}

fn extract_douyin_hls_streams(html: &str) -> Vec<(String, String)> {
    let regex = Regex::new(
        r#"\\"?(FULL_HD1|HD1|SD1|SD2|LD|ORIGIN|origin)\\"?\s*:\s*\\"?(https?://pull-hls[^"\\]+(?:\\u0026[^"\\]+)*)"#,
    )
    .expect("valid douyin hls regex");

    let mut streams = Vec::new();
    for captures in regex.captures_iter(html) {
        let label = captures.get(1).map(|v| v.as_str().to_ascii_uppercase()).unwrap_or_default();
        let url = captures.get(2).map(|v| unescape_json_url(v.as_str())).unwrap_or_default();
        if !url.is_empty() && !streams.iter().any(|(_, existing)| existing == &url) {
            streams.push((label, url));
        }
    }
    streams
}

fn select_douyin_stream(streams: &[(String, String)], quality: &str) -> Option<String> {
    let labels = match quality.to_ascii_lowercase().as_str() {
        "best" | "1080p60" | "1080p" => ["FULL_HD1", "ORIGIN", "HD1", "SD2", "SD1", "LD"],
        "720p60" | "720p" | "480p" => ["SD2", "SD1", "HD1", "FULL_HD1", "ORIGIN", "LD"],
        "360p" | "worst" => ["LD", "SD1", "SD2", "HD1", "FULL_HD1", "ORIGIN"],
        _ => ["FULL_HD1", "ORIGIN", "HD1", "SD2", "SD1", "LD"],
    };

    labels.iter().find_map(|label| {
        streams
            .iter()
            .find(|(stream_label, _)| stream_label == label)
            .map(|(_, stream)| stream.clone())
    })
}

fn extract_douyin_title(html: &str) -> Option<String> {
    let regexes = [r#"\\"title\\"?\s*:\s*\\"([^"\\]+)\\""#, r#"<title>([^<]+)</title>"#];
    regexes.iter().find_map(|pattern| {
        Regex::new(pattern)
            .ok()?
            .captures(html)?
            .get(1)
            .map(|v| v.as_str().trim().to_string())
            .filter(|v| !v.is_empty())
    })
}

fn unescape_json_url(raw: &str) -> String {
    raw.replace("\\u0026", "&")
        .replace("\\/", "/")
        .replace("\\\\", "\\")
        .trim_matches('"')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        LivePlatform, ResolvedStream, detect_platform, extract_douyin_hls_streams, resolve_stream,
        select_douyin_stream,
    };
    use crate::checker::STREAMLINK_PATH;
    use std::{path::PathBuf, time::Duration};
    use tokio::{process::Command, time::sleep};

    #[test]
    fn detects_requested_platform_urls() {
        assert_eq!(detect_platform("https://www.douyu.com/74960"), LivePlatform::Douyu);
        assert_eq!(
            detect_platform("https://www.tiktok.com/@diemhuynh_2003/live"),
            LivePlatform::Tiktok
        );
        assert_eq!(detect_platform("https://live.douyin.com/393646574978"), LivePlatform::Douyin);
        assert_eq!(detect_platform("https://www.twitch.tv/seucreysonreborn"), LivePlatform::Twitch);
        assert_eq!(detect_platform("https://kick.com/topson"), LivePlatform::Kick);
    }

    #[test]
    fn extracts_and_selects_douyin_hls_streams() {
        let html = r#"\"hls_pull_url_map\":{\"FULL_HD1\":\"http://pull-hls-l11.douyincdn.com/stage/origin.m3u8?expire=1\u0026sign=a\",\"SD1\":\"http://pull-hls-l11.douyincdn.com/stage/sd.m3u8?expire=1\u0026sign=b\",\"LD\":\"http://pull-hls-l11.douyincdn.com/stage/ld.m3u8?expire=1\u0026sign=c\"}"#;
        let streams = extract_douyin_hls_streams(html);

        assert_eq!(streams.len(), 3);
        assert_eq!(
            select_douyin_stream(&streams, "best").as_deref(),
            Some("http://pull-hls-l11.douyincdn.com/stage/origin.m3u8?expire=1&sign=a")
        );
        assert_eq!(
            select_douyin_stream(&streams, "worst").as_deref(),
            Some("http://pull-hls-l11.douyincdn.com/stage/ld.m3u8?expire=1&sign=c")
        );
    }

    #[tokio::test]
    #[ignore]
    async fn live_examples_can_record_short_segments() {
        let cases = [
            ("douyu", "https://www.douyu.com/74960", true),
            ("tiktok", "https://www.tiktok.com/@diemhuynh_2003/live", false),
            ("douyin", "https://live.douyin.com/393646574978", true),
            ("twitch", "https://www.twitch.tv/seucreysonreborn", false),
            ("kick", "https://kick.com/topson", false),
        ];

        for (name, original_url, needs_resolve) in cases {
            let resolved = if needs_resolve {
                resolve_stream(original_url, "best")
                    .await
                    .expect("platform resolver should not fail")
                    .expect("platform resolver should return a stream")
            } else {
                ResolvedStream {
                    input_url: original_url.to_string(),
                    title: None,
                    direct_input: false,
                }
            };

            record_short_segment(name, &resolved).await;
        }
    }

    #[tokio::test]
    #[ignore]
    async fn kick_live_example_can_record_short_segment() {
        let resolved = ResolvedStream {
            input_url: "https://kick.com/topson".to_string(),
            title: None,
            direct_input: false,
        };

        record_short_segment("kick-topson", &resolved).await;
    }

    async fn record_short_segment(name: &str, stream: &ResolvedStream) {
        if stream.direct_input {
            record_short_segment_with_ffmpeg(name, &stream.input_url).await;
        } else {
            record_short_segment_with_streamlink(name, &stream.input_url).await;
        }
    }

    async fn record_short_segment_with_streamlink(name: &str, input_url: &str) {
        let output = live_output_path(name);
        let _ = tokio::fs::remove_file(&output).await;

        let mut child = Command::new(STREAMLINK_PATH)
            .arg("--force")
            .arg("-o")
            .arg(&output)
            .arg(input_url)
            .arg("best")
            .kill_on_drop(true)
            .spawn()
            .expect("streamlink should spawn");

        sleep(Duration::from_secs(15)).await;
        let _ = child.kill().await;
        let _ = child.wait().await;

        let len = tokio::fs::metadata(&output)
            .await
            .unwrap_or_else(|_| panic!("{name} should create a recording file"))
            .len();
        assert!(len > 0, "{name} recording file should not be empty");
    }

    async fn record_short_segment_with_ffmpeg(name: &str, input_url: &str) {
        let output = live_output_path(name);
        let _ = tokio::fs::remove_file(&output).await;

        let mut child = Command::new("ffmpeg")
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
            .arg(&output)
            .kill_on_drop(true)
            .spawn()
            .expect("ffmpeg should spawn");

        sleep(Duration::from_secs(15)).await;
        let _ = child.kill().await;
        let _ = child.wait().await;

        let len = tokio::fs::metadata(&output)
            .await
            .unwrap_or_else(|_| panic!("{name} should create a recording file"))
            .len();
        assert!(len > 0, "{name} recording file should not be empty");
    }

    fn live_output_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("omnistream-live-smoke-{name}.mp4"))
    }
}
