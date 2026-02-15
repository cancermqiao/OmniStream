use anyhow::Result;
use async_trait::async_trait;
use shared::UploadConfig;

pub mod bilibili;

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
