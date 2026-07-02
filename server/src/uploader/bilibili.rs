use super::Uploader;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use biliup::client::StatelessClient;
use biliup::uploader::VideoFile;
use biliup::uploader::bilibili::Studio;
use biliup::uploader::credential::login_by_cookies;
use biliup::uploader::line::Probe;
use chrono::Local;
use futures::TryStreamExt;
use shared::UploadConfig;
use std::path::Path;

pub struct BilibiliUploader;

impl BilibiliUploader {
    const MAX_TITLE_CHARS: usize = 80;

    pub fn new() -> Self {
        Self
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
            let line = Probe::probe(&client.client)
                .await
                .map_err(|e| anyhow!("Failed to probe upload line: {}", e))?;

            // 预上传
            let parcel = line
                .pre_upload(&bili, video_file)
                .await
                .map_err(|e| anyhow!("Failed to pre_upload: {}", e))?;

            // 上传
            // upload(client, limit, progress_callback, retry)
            // progress callback needs to return a Stream<Item = Result<(B, usize), Kind>>
            let video = parcel
                .upload(
                    client.clone(),
                    3,
                    |stream| {
                        stream.map_err(biliup::error::Kind::from).map_ok(|b| {
                            let len = b.len();
                            (b, len)
                        })
                    },
                    3,
                )
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

        // Tag 必须非空，如果 config 中没有，使用 "omnistream"
        let tag =
            if config.tags.is_empty() { "omnistream".to_string() } else { config.tags.join(",") };

        let studio = Studio::builder()
            .title(title)
            .videos(videos)
            .tid(config.tid)
            .copyright(config.copyright)
            .desc(config.description.clone())
            .dynamic(config.dynamic.clone())
            .tag(tag)
            .desc_v2(None) // required field
            .build();

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
