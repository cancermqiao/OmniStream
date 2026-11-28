use anyhow::Result;
use async_trait::async_trait;
use shared::UploadConfig;

pub mod bilibili;

pub(crate) fn is_transient_upload_error(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    [
        "所有上传线路测速均失败",
        "all upload lines failed",
        "500",
        "502",
        "503",
        "504",
        "gateway time-out",
        "gateway timeout",
        "timeout",
        "timed out",
        "error sending request",
        "connection reset",
        "connection closed",
        "connection refused",
        "connection aborted",
        "temporarily unavailable",
        "network",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}

#[allow(clippy::double_must_use)]
#[async_trait]
pub trait Uploader: Send + Sync {
    /// 上传文件列表
    async fn upload(
        &self,
        filenames: Vec<String>,
        config: &UploadConfig,
        live_title: Option<&str>,
        task_name: &str,
    ) -> Result<()>;
}

// 简单的工厂方法或枚举来管理多种上传方式
pub enum UploadTarget {
    Bilibili,
}

impl UploadTarget {
    pub fn create_uploader(&self) -> Box<dyn Uploader> {
        match self {
            UploadTarget::Bilibili => Box::new(bilibili::BilibiliUploader::new()),
        }
    }
}
