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
        LivePlatform::Bilibili => resolve_bilibili(url, quality).await,
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

async fn resolve_bilibili(url: &str, quality: &str) -> Result<Option<ResolvedStream>> {
    let room_id = extract_bilibili_room_id(url).await?;
    let client = bilibili_http_client()?;

    let room_init: Value = client
        .get(format!("https://api.live.bilibili.com/room/v1/Room/room_init?id={room_id}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    ensure_bilibili_api_ok(&room_init, "room_init")?;

    let Some(room_data) = room_init.get("data") else {
        return Err(anyhow!("Bilibili room_init API missing data: {room_init}"));
    };
    if room_data.get("live_status").and_then(Value::as_i64) != Some(1) {
        return Ok(None);
    }
    let canonical_room_id = room_data
        .get("room_id")
        .and_then(Value::as_i64)
        .map(|v| v.to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or(room_id);

    let title = fetch_bilibili_title(&client, &canonical_room_id).await;
    let play_info = fetch_bilibili_play_info(&client, &canonical_room_id, quality).await?;
    let input_url = select_bilibili_stream_url(&play_info)
        .ok_or_else(|| anyhow!("Bilibili play info does not contain playable stream URL"))?;

    Ok(Some(ResolvedStream { input_url, title, direct_input: true }))
}

fn bilibili_http_client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36",
        )
        .build()?)
}

async fn extract_bilibili_room_id(url: &str) -> Result<String> {
    if let Some(room_id) = parse_bilibili_room_id_from_url(url) {
        return Ok(room_id);
    }

    let response = bilibili_http_client()?.get(url).send().await?.error_for_status()?;
    if let Some(room_id) = parse_bilibili_room_id_from_url(response.url().as_str()) {
        return Ok(room_id);
    }
    let html = response.text().await?;

    let patterns = [
        r#""room_id"\s*:\s*(\d+)"#,
        r#""roomId"\s*:\s*(\d+)"#,
        r#"room_id=(\d+)"#,
        r#"live\.bilibili\.com/(\d+)"#,
    ];
    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .find_map(|regex| regex.captures(&html))
        .and_then(|captures| captures.get(1).map(|v| v.as_str().to_string()))
        .ok_or_else(|| anyhow!("Bilibili room id not found in URL or page HTML"))
}

fn parse_bilibili_room_id_from_url(raw_url: &str) -> Option<String> {
    let url = Url::parse(raw_url).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    if !(host == "b23.tv" || host.ends_with("bilibili.com")) {
        return None;
    }

    let mut segments = url.path_segments()?;
    if host.starts_with("live.") || url.path().contains("/live/") {
        return segments
            .find(|segment| segment.chars().all(|c| c.is_ascii_digit()))
            .map(str::to_string);
    }
    segments.find(|segment| segment.chars().all(|c| c.is_ascii_digit())).map(str::to_string)
}

async fn fetch_bilibili_title(client: &Client, room_id: &str) -> Option<String> {
    let response: Value = client
        .get(format!("https://api.live.bilibili.com/room/v1/Room/get_info?room_id={room_id}"))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;
    if ensure_bilibili_api_ok(&response, "get_info").is_err() {
        return None;
    }
    non_empty_json_string(&response, &["data", "title"])
}

async fn fetch_bilibili_play_info(client: &Client, room_id: &str, quality: &str) -> Result<Value> {
    let qn = bilibili_quality_qn(quality);
    let response: Value = client
        .get(format!(
            "https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo?room_id={room_id}&protocol=0,1&format=0,1,2&codec=0,1&qn={qn}&platform=web&ptype=8"
        ))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    ensure_bilibili_api_ok(&response, "getRoomPlayInfo")?;
    Ok(response)
}

fn ensure_bilibili_api_ok(value: &Value, api: &str) -> Result<()> {
    if value.get("code").and_then(Value::as_i64) == Some(0) {
        return Ok(());
    }
    Err(anyhow!("Bilibili {api} API failed: {value}"))
}

fn bilibili_quality_qn(quality: &str) -> &'static str {
    match quality.to_ascii_lowercase().as_str() {
        "4k" | "2160p" => "20000",
        "best" | "origin" | "source" | "原画" => "10000",
        "1080p" | "1080p60" | "bluray" | "蓝光" => "400",
        "720p" | "720p60" | "hd" => "250",
        "480p" | "sd" => "150",
        "360p" | "worst" => "80",
        _ => "10000",
    }
}

fn select_bilibili_stream_url(value: &Value) -> Option<String> {
    let streams =
        value.get("data")?.get("playurl_info")?.get("playurl")?.get("stream")?.as_array()?;

    for stream in streams {
        let formats = stream.get("format")?.as_array()?;
        for format in formats {
            let codecs = format.get("codec")?.as_array()?;
            for codec in codecs {
                let base_url = codec.get("base_url").and_then(Value::as_str)?;
                let url_infos = codec.get("url_info")?.as_array()?;
                for url_info in url_infos {
                    let host = url_info.get("host").and_then(Value::as_str)?;
                    let extra = url_info.get("extra").and_then(Value::as_str).unwrap_or_default();
                    let resolved = format!("{host}{base_url}{extra}");
                    if resolved.starts_with("http://") || resolved.starts_with("https://") {
                        return Some(resolved);
                    }
                }
            }
        }
    }
    None
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
        LivePlatform, ResolvedStream, bilibili_quality_qn, detect_platform,
        extract_douyin_hls_streams, non_empty_json_string, parse_bilibili_room_id_from_url,
        resolve_stream, select_bilibili_stream_url, select_douyin_stream,
    };
    use crate::checker::STREAMLINK_PATH;
    use serde_json::json;
    use std::{path::PathBuf, time::Duration};
    use tokio::{process::Command, time::sleep};

    #[test]
    fn detects_requested_platform_urls() {
        assert_eq!(detect_platform("https://live.bilibili.com/6"), LivePlatform::Bilibili);
        assert_eq!(detect_platform("https://b23.tv/abc123"), LivePlatform::Bilibili);
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
    fn parses_bilibili_room_id_from_common_urls() {
        assert_eq!(
            parse_bilibili_room_id_from_url("https://live.bilibili.com/6?broadcast_type=0"),
            Some("6".to_string())
        );
        assert_eq!(
            parse_bilibili_room_id_from_url("https://www.bilibili.com/live/22603245"),
            Some("22603245".to_string())
        );
        assert_eq!(parse_bilibili_room_id_from_url("https://www.example.com/6"), None);
    }

    #[test]
    fn maps_bilibili_quality_to_qn() {
        assert_eq!(bilibili_quality_qn("best"), "10000");
        assert_eq!(bilibili_quality_qn("4k"), "20000");
        assert_eq!(bilibili_quality_qn("1080p"), "400");
        assert_eq!(bilibili_quality_qn("720p"), "250");
        assert_eq!(bilibili_quality_qn("worst"), "80");
    }

    #[test]
    fn selects_bilibili_stream_url_from_play_info() {
        let value = json!({
            "code": 0,
            "data": {
                "playurl_info": {
                    "playurl": {
                        "stream": [{
                            "format": [{
                                "codec": [{
                                    "base_url": "/live-bvc/stream.m4s",
                                    "url_info": [{
                                        "host": "https://example.live/",
                                        "extra": "?token=abc"
                                    }]
                                }]
                            }]
                        }]
                    }
                }
            }
        });

        assert_eq!(
            select_bilibili_stream_url(&value),
            Some("https://example.live//live-bvc/stream.m4s?token=abc".to_string())
        );
    }

    #[test]
    fn extracts_bilibili_title_from_room_info() {
        let value = json!({
            "code": 0,
            "data": { "title": "开播测试" }
        });
        assert_eq!(non_empty_json_string(&value, &["data", "title"]), Some("开播测试".to_string()));
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
            ("bilibili", "https://live.bilibili.com/6", true),
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
