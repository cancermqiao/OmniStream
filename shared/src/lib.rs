use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Idle,          // 空闲/未开始
    Recording,     // 录制中
    Uploading,     // 上传中
    Completed,     // 已完成
    Error(String), // 失败
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamTask {
    pub id: String,
    pub name: String,
    pub url: String,
    pub status: TaskStatus,
    pub filename: String, // 保存的文件名
    #[serde(default)]
    pub upload_configs: Vec<UploadConfig>, // 任务运行时携带的多个上传配置
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub url: String,
}

fn default_tid() -> u16 {
    171
} // 电子竞技
fn default_copyright() -> u8 {
    1
} // 自制
fn default_account_file() -> String {
    "cookies.json".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UploadConfig {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_tid")]
    pub tid: u16,
    #[serde(default = "default_copyright")]
    pub copyright: u8,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub dynamic: String,
    // 账号配置文件路径，默认为 cookies.json
    #[serde(default = "default_account_file")]
    pub account_file: String,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            title: None,
            tags: vec![],
            tid: default_tid(),
            copyright: default_copyright(),
            description: "".to_string(),
            dynamic: "".to_string(),
            account_file: default_account_file(),
        }
    }
}

// 对应前端 Download 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DownloadConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub linked_upload_ids: Vec<String>, // 关联的 UploadTemplate ID 列表
    #[serde(default)]
    pub current_status: Option<String>, // 当前运行状态（实时计算，不落库）
    #[serde(default)]
    pub use_custom_recording_settings: bool, // 是否启用任务级录制设置
    #[serde(default)]
    pub recording_settings: Option<RecordingSettings>, // 任务级录制设置
}

// 对应前端 Upload 配置 (模板)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct UploadTemplate {
    pub id: String,
    pub name: String, // 模板名称
    pub config: UploadConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct UploadAccount {
    pub id: String,
    pub name: String,
    pub mid: Option<u64>,
    pub account_file: String,
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BiliupConfig {
    // 录制分段大小（字节），0 或 None 表示不分段
    #[serde(default)]
    pub segment_size: Option<u64>,
    // 录制分段时长（秒），0 或 None 表示不分段
    #[serde(default)]
    pub segment_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformQualityConfig {
    pub bilibili: String,
    pub douyu: String,
    pub huya: String,
    pub twitch: String,
    pub youtube: String,
    pub default_quality: String,
}

impl Default for PlatformQualityConfig {
    fn default() -> Self {
        Self {
            bilibili: "best".to_string(),
            douyu: "best".to_string(),
            huya: "best".to_string(),
            twitch: "best".to_string(),
            youtube: "best".to_string(),
            default_quality: "best".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RecordingSettings {
    #[serde(default)]
    pub segment_size_mb: Option<u64>,
    #[serde(default)]
    pub segment_time_sec: Option<u64>,
    #[serde(default)]
    pub quality: PlatformQualityConfig,
    #[serde(default)]
    pub auto_cleanup_after_upload: bool,
}

#[async_trait::async_trait]
pub trait StreamChecker: Send + Sync {
    async fn check(&self, url: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::UploadConfig;

    #[test]
    fn upload_config_default_values_are_stable() {
        let config = UploadConfig::default();
        assert_eq!(config.title, None);
        assert!(config.tags.is_empty());
        assert_eq!(config.tid, 171);
        assert_eq!(config.copyright, 1);
        assert_eq!(config.description, "");
        assert_eq!(config.dynamic, "");
        assert_eq!(config.account_file, "cookies.json");
    }

    #[test]
    fn upload_config_deserialize_uses_defaults() {
        let json = "{}";
        let config: UploadConfig = serde_json::from_str(json).expect("valid upload config json");
        assert_eq!(config, UploadConfig::default());
    }
}
