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
        let title = config
            .title
            .as_ref()
            .map(|t| Self::render_title(t, live_title, task_name).trim().to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| {
                videos[0].title.clone().unwrap_or_else(|| "Uploaded by OmniStream".to_string())
            });

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
