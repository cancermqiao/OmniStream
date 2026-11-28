use super::{Uploader, is_transient_upload_error};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use biliup::client::StatelessClient;
use biliup::uploader::VideoFile;
use biliup::uploader::bilibili::{Studio, Video};
use biliup::uploader::credential::login_by_cookies;
use biliup::uploader::line::Probe;
use chrono::Local;
use futures::TryStreamExt;
use shared::UploadConfig;
use std::{future::Future, path::Path, time::Duration};
use tokio::time::sleep;

pub struct BilibiliUploader;

impl BilibiliUploader {
    const MAX_TITLE_CHARS: usize = 80;
    const MAX_LINE_PROBE_ATTEMPTS: usize = 3;
    const LINE_PROBE_RETRY_DELAYS_SECS: [u64; 2] = [5, 15];

    pub fn new() -> Self {
        Self
    }

    async fn retry_line_probe<F, Fut, T>(filename: &str, mut probe: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut attempt = 1;
        loop {
            match probe().await {
                Ok(line) => return Ok(line),
                Err(error) => {
                    let transient = is_transient_upload_error(&error.to_string());
                    if !transient || attempt == Self::MAX_LINE_PROBE_ATTEMPTS {
                        tracing::warn!(
                            "Upload line probe stopped: file={}, attempt={}/{}, transient={}, error={}",
                            filename,
                            attempt,
                            Self::MAX_LINE_PROBE_ATTEMPTS,
                            transient,
                            error
                        );
                        return Err(error);
                    }
                    let delay = Self::LINE_PROBE_RETRY_DELAYS_SECS[attempt - 1];
                    tracing::warn!(
                        "Upload line probe failed, retrying current file after {}s: file={}, attempt={}/{}, error={}",
                        delay,
                        filename,
                        attempt,
                        Self::MAX_LINE_PROBE_ATTEMPTS,
                        error
                    );
                    sleep(Duration::from_secs(delay)).await;
                    attempt += 1;
                }
            }
        }
    }

    fn render_title(template: &str, live_title: Option<&str>, task_name: &str) -> String {
        let placeholder_title =
            live_title.map(str::trim).filter(|v| !v.is_empty()).unwrap_or(task_name);
        let merged = template.replace("{title}", placeholder_title);
        // Supports chrono strftime placeholders, e.g. %Y-%m-%d / %H:%M.
        Local::now().format(&merged).to_string()
    }

    fn normalize_title(raw: String) -> String {
        let trimmed = raw.trim();
        let char_count = trimmed.chars().count();
        if char_count <= Self::MAX_TITLE_CHARS {
            return trimmed.to_string();
        }

        let truncated = trimmed.chars().take(Self::MAX_TITLE_CHARS).collect::<String>();
        tracing::warn!(
            "Bilibili upload title exceeded {} chars and was truncated: original_chars={}, truncated_title={:?}",
            Self::MAX_TITLE_CHARS,
            char_count,
            truncated
        );
        truncated
    }

    fn resolve_title(
        config: &UploadConfig,
        live_title: Option<&str>,
        task_name: &str,
        fallback_title: Option<&str>,
    ) -> String {
        let raw_title = config
            .title
            .as_ref()
            .map(|t| Self::render_title(t, live_title, task_name))
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| fallback_title.unwrap_or("Uploaded by OmniStream").to_string());

        Self::normalize_title(raw_title)
    }

    fn validate_config(config: &UploadConfig) -> Result<()> {
        if config.account_file.trim().is_empty() {
            return Err(anyhow!("account_file is required"));
        }
        if config.tid == 0 {
            return Err(anyhow!("tid is required"));
        }
        if config.copyright != 1 && config.copyright != 2 {
            return Err(anyhow!("copyright must be 1 (Original) or 2 (Reprint)"));
        }
        Ok(())
    }

    fn build_studio(config: &UploadConfig, title: String, videos: Vec<Video>) -> Studio {
        let tag =
            if config.tags.is_empty() { "omnistream".to_string() } else { config.tags.join(",") };
        Studio::builder()
            .title(title)
            .videos(videos)
            .tid(config.tid)
            .copyright(config.copyright)
            .source(String::new())
            .cover(String::new())
            .desc(config.description.clone())
            .dynamic(config.dynamic.clone())
            .tag(tag)
            .is_only_self(u8::from(config.only_self))
            .dolby(0)
            .no_reprint(0)
            .charging_pay(0)
            .up_selection_reply(false)
            .up_close_reply(false)
            .up_close_danmu(false)
            .build()
    }
}

#[async_trait]
impl Uploader for BilibiliUploader {
    async fn upload(
        &self,
        filenames: Vec<String>,
        config: &UploadConfig,
        live_title: Option<&str>,
        task_name: &str,
    ) -> Result<()> {
        if filenames.is_empty() {
            return Ok(());
        }
        Self::validate_config(config)?;

        // 1. 登录
        // login_by_cookies returns Result<BiliBili>
        let bili = login_by_cookies(&config.account_file, None)
            .await
            .map_err(|e| anyhow!("Failed to login by {}: {}", config.account_file, e))?;

        let client = StatelessClient::default();
        let mut videos = Vec::new();

        // 2. 上传每个文件
        for filename in filenames {
            let path = Path::new(&filename);
            if !path.exists() {
                return Err(anyhow!("File not found: {}", filename));
            }

            tracing::info!("Starting upload for: {}", filename);

            let video_file = VideoFile::new(path)?;

            // 探测线路
            let line = Self::retry_line_probe(&filename, || async {
                Probe::probe(&client.client)
                    .await
                    .map_err(|e| anyhow!("Failed to probe upload line: {}", e))
            })
            .await?;

            // 预上传
            let parcel = line
                .pre_upload(&bili, video_file)
                .await
                .map_err(|e| anyhow!("Failed to pre_upload: {}", e))?;

            // 上传
            // Chunk retries are handled by biliup internally.
            // progress callback needs to return a Stream<Item = Result<(B, usize), Kind>>
            let video = parcel
                .upload(client.clone(), 3, |stream| {
                    stream.map_err(biliup::error::Kind::from).map_ok(|b| {
                        let len = b.len();
                        (b, len)
                    })
                })
                .await
                .map_err(|e| anyhow!("Failed to upload file {}: {}", filename, e))?;

            videos.push(video);
            tracing::info!("Uploaded: {}", filename);
        }

        if videos.is_empty() {
            return Err(anyhow!("No videos uploaded"));
        }

        // 3. 提交投稿
        let title = Self::resolve_title(config, live_title, task_name, videos[0].title.as_deref());

        tracing::info!(
            "Resolved upload title: template={:?}, live_title={:?}, task_name={:?}, final_title={:?}",
            config.title,
            live_title,
            task_name,
            title
        );

        let studio = Self::build_studio(config, title, videos);

        tracing::info!("Submitting archive: {:?}", studio.title);

        let ret = bili
            .submit_by_app(&studio, None)
            .await
            .map_err(|e| anyhow!("Failed to submit archive: {}", e))?;

        tracing::info!("Submission result: {:?}", ret);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BilibiliUploader;

    #[tokio::test(start_paused = true)]
    async fn line_probe_recovers_after_transient_failures() {
        let mut attempts = 0;
        let started = tokio::time::Instant::now();
        let line = BilibiliUploader::retry_line_probe("current.mp4", || {
            attempts += 1;
            std::future::ready(if attempts < 3 {
                Err(anyhow::anyhow!("Failed to probe upload line: 所有上传线路测速均失败"))
            } else {
                Ok("usable-line")
            })
        })
        .await
        .expect("transient probe failures should recover");

        assert_eq!(line, "usable-line");
        assert_eq!(attempts, 3);
        assert_eq!(started.elapsed(), std::time::Duration::from_secs(20));
    }

    #[tokio::test(start_paused = true)]
    async fn line_probe_stops_after_bounded_attempts_and_preserves_error() {
        let mut attempts = 0;
        let started = tokio::time::Instant::now();
        let error = BilibiliUploader::retry_line_probe("current.mp4", || {
            attempts += 1;
            std::future::ready(Err::<(), _>(anyhow::anyhow!(
                "Failed to probe upload line: 所有上传线路测速均失败"
            )))
        })
        .await
        .expect_err("persistent failures must not loop forever");

        assert_eq!(attempts, 3);
        assert_eq!(started.elapsed(), std::time::Duration::from_secs(20));
        assert!(super::is_transient_upload_error(&error.to_string()));
    }

    #[tokio::test(start_paused = true)]
    async fn line_probe_does_not_retry_validation_errors() {
        let mut attempts = 0;
        let started = tokio::time::Instant::now();
        let error = BilibiliUploader::retry_line_probe("current.mp4", || {
            attempts += 1;
            std::future::ready(Err::<(), _>(anyhow::anyhow!("account_file is required")))
        })
        .await
        .expect_err("validation errors should fail immediately");

        assert_eq!(attempts, 1);
        assert_eq!(started.elapsed(), std::time::Duration::ZERO);
        assert_eq!(error.to_string(), "account_file is required");
    }

    #[tokio::test(start_paused = true)]
    async fn successful_line_probe_does_not_wait_or_retry() {
        let mut attempts = 0;
        let started = tokio::time::Instant::now();
        let line = BilibiliUploader::retry_line_probe("current.mp4", || {
            attempts += 1;
            std::future::ready(Ok("usable-line"))
        })
        .await
        .expect("first successful probe should return immediately");

        assert_eq!(line, "usable-line");
        assert_eq!(attempts, 1);
        assert_eq!(started.elapsed(), std::time::Duration::ZERO);
    }

    #[test]
    fn submission_payload_preserves_metadata_and_visibility() {
        for only_self in [false, true] {
            let config = shared::UploadConfig {
                only_self,
                tid: 21,
                copyright: 2,
                tags: vec!["录播".into(), "游戏".into()],
                description: "视频简介".into(),
                dynamic: "投稿动态".into(),
                ..Default::default()
            };
            let studio = BilibiliUploader::build_studio(&config, "测试标题".into(), vec![]);
            let payload = serde_json::to_value(studio).expect("serialize submission payload");
            assert_eq!(payload["is_only_self"], u8::from(only_self));
            assert_eq!(payload["title"], "测试标题");
            assert_eq!(payload["tid"], 21);
            assert_eq!(payload["copyright"], 2);
            assert_eq!(payload["tag"], "录播,游戏");
            assert_eq!(payload["desc"], "视频简介");
            assert_eq!(payload["dynamic"], "投稿动态");
            assert_eq!(payload["charging_pay"], 0);
        }
    }

    #[test]
    fn render_title_uses_live_title_placeholder() {
        let rendered = BilibiliUploader::render_title("{title} 录播", Some("开整"), "任务名");
        assert!(rendered.starts_with("开整 录播"));
    }

    #[test]
    fn render_title_falls_back_to_task_name_when_live_title_is_blank() {
        let rendered = BilibiliUploader::render_title("{title} 录播", Some("   "), "任务名");
        assert!(rendered.starts_with("任务名 录播"));
    }

    #[test]
    fn render_title_applies_time_placeholders() {
        let rendered = BilibiliUploader::render_title("{title}-%Y", Some("开整"), "任务名");
        assert!(rendered.starts_with("开整-"));
        assert_eq!(rendered.len(), "开整-2026".len());
    }

    #[test]
    fn normalize_title_truncates_to_bilibili_limit() {
        let long_title = "a".repeat(100);
        let rendered = BilibiliUploader::normalize_title(long_title);

        assert_eq!(rendered.chars().count(), BilibiliUploader::MAX_TITLE_CHARS);
    }

    #[test]
    fn resolve_title_truncates_rendered_live_title() {
        let config = shared::UploadConfig {
            title: Some("【Arteezy直播录像%Y-%m-%d】{title}".to_string()),
            ..Default::default()
        };
        let live_title = "x".repeat(120);
        let rendered = BilibiliUploader::resolve_title(&config, Some(&live_title), "Arteezy", None);

        assert_eq!(rendered.chars().count(), BilibiliUploader::MAX_TITLE_CHARS);
    }
}
