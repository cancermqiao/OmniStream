use dioxus::prelude::*;
use shared::{
    DownloadConfig, QrStartResponse, RecordingSettings, StorageStats, UploadAccount, UploadTemplate,
};

#[cfg(feature = "server")]
use std::sync::{Arc, OnceLock};

#[cfg(feature = "server")]
#[async_trait::async_trait]
pub trait BackendApi: Send + Sync {
    async fn fetch_downloads(&self) -> Result<Vec<DownloadConfig>, String>;
    async fn fetch_uploads(&self) -> Result<Vec<UploadTemplate>, String>;
    async fn fetch_accounts(&self) -> Result<Vec<UploadAccount>, String>;
    async fn save_download(&self, payload: DownloadConfig) -> Result<(), String>;
    async fn delete_download(&self, id: String) -> Result<(), String>;
    async fn clear_download_files(&self, id: String) -> Result<String, String>;
    async fn stop_download(&self, id: String) -> Result<String, String>;
    async fn resume_download(&self, id: String) -> Result<String, String>;
    async fn save_upload(&self, payload: UploadTemplate) -> Result<(), String>;
    async fn delete_upload(&self, id: String) -> Result<(), String>;
    async fn start_qr_login(&self) -> Result<QrStartResponse, String>;
    async fn confirm_qr_login(&self, session_id: String) -> Result<(), String>;
    async fn rename_account(
        &self,
        account_file: String,
        display_name: String,
    ) -> Result<(), String>;
    async fn delete_account(&self, account_file: String) -> Result<(), String>;
    async fn fetch_recording_settings(&self) -> Result<RecordingSettings, String>;
    async fn fetch_storage_stats(&self) -> Result<StorageStats, String>;
    async fn save_recording_settings(&self, settings: RecordingSettings) -> Result<(), String>;
    async fn trigger_manual_upload(&self, id: String) -> Result<String, String>;
}

#[cfg(feature = "server")]
static BACKEND: OnceLock<Arc<dyn BackendApi>> = OnceLock::new();

#[cfg(feature = "server")]
pub fn install_backend(backend: Arc<dyn BackendApi>) {
    let _ = BACKEND.set(backend);
}

#[cfg(feature = "server")]
fn backend() -> dioxus::prelude::ServerFnResult<&'static Arc<dyn BackendApi>> {
    BACKEND.get().ok_or_else(|| dioxus::prelude::ServerFnError::new("backend is not installed"))
}

#[cfg(feature = "server")]
fn server_error(err: impl ToString) -> dioxus::prelude::ServerFnError {
    dioxus::prelude::ServerFnError::new(err.to_string())
}

#[server]
async fn server_fetch_downloads() -> ServerFnResult<Vec<DownloadConfig>> {
    backend().cloned()?.fetch_downloads().await.map_err(server_error)
}

#[server]
async fn server_fetch_uploads() -> ServerFnResult<Vec<UploadTemplate>> {
    backend().cloned()?.fetch_uploads().await.map_err(server_error)
}

#[server]
async fn server_fetch_accounts() -> ServerFnResult<Vec<UploadAccount>> {
    backend().cloned()?.fetch_accounts().await.map_err(server_error)
}

#[server]
async fn server_save_download(payload: DownloadConfig) -> ServerFnResult<()> {
    backend().cloned()?.save_download(payload).await.map_err(server_error)
}

#[server]
async fn server_delete_download(id: String) -> ServerFnResult<()> {
    backend().cloned()?.delete_download(id).await.map_err(server_error)
}

#[server]
async fn server_clear_download_files(id: String) -> ServerFnResult<String> {
    backend().cloned()?.clear_download_files(id).await.map_err(server_error)
}

#[server]
async fn server_stop_download(id: String) -> ServerFnResult<String> {
    backend().cloned()?.stop_download(id).await.map_err(server_error)
}

#[server]
async fn server_resume_download(id: String) -> ServerFnResult<String> {
    backend().cloned()?.resume_download(id).await.map_err(server_error)
}

#[server]
async fn server_save_upload(payload: UploadTemplate) -> ServerFnResult<()> {
    backend().cloned()?.save_upload(payload).await.map_err(server_error)
}

#[server]
async fn server_delete_upload(id: String) -> ServerFnResult<()> {
    backend().cloned()?.delete_upload(id).await.map_err(server_error)
}

#[server]
async fn server_start_qr_login() -> ServerFnResult<QrStartResponse> {
    backend().cloned()?.start_qr_login().await.map_err(server_error)
}

#[server]
async fn server_confirm_qr_login(session_id: String) -> ServerFnResult<()> {
    backend().cloned()?.confirm_qr_login(session_id).await.map_err(server_error)
}

#[server]
async fn server_rename_account(account_file: String, display_name: String) -> ServerFnResult<()> {
    backend().cloned()?.rename_account(account_file, display_name).await.map_err(server_error)
}

#[server]
async fn server_delete_account(account_file: String) -> ServerFnResult<()> {
    backend().cloned()?.delete_account(account_file).await.map_err(server_error)
}

#[server]
async fn server_fetch_recording_settings() -> ServerFnResult<RecordingSettings> {
    backend().cloned()?.fetch_recording_settings().await.map_err(server_error)
}

#[server]
async fn server_fetch_storage_stats() -> ServerFnResult<StorageStats> {
    backend().cloned()?.fetch_storage_stats().await.map_err(server_error)
}

#[server]
async fn server_save_recording_settings(settings: RecordingSettings) -> ServerFnResult<()> {
    backend().cloned()?.save_recording_settings(settings).await.map_err(server_error)
}

#[server]
async fn server_trigger_manual_upload(id: String) -> ServerFnResult<String> {
    backend().cloned()?.trigger_manual_upload(id).await.map_err(server_error)
}

pub async fn fetch_downloads(_api_url: &str) -> Option<Vec<DownloadConfig>> {
    server_fetch_downloads().await.ok()
}

pub async fn fetch_uploads(_api_url: &str) -> Option<Vec<UploadTemplate>> {
    server_fetch_uploads().await.ok()
}

pub async fn fetch_accounts(_api_url: &str) -> Option<Vec<UploadAccount>> {
    server_fetch_accounts().await.ok()
}

pub async fn save_download(_api_url: &str, payload: &DownloadConfig) -> Result<(), String> {
    server_save_download(payload.clone()).await.map_err(|e| e.to_string())
}

pub async fn delete_download(_api_url: &str, id: &str) -> Result<(), String> {
    server_delete_download(id.to_string()).await.map_err(|e| e.to_string())
}

pub async fn clear_download_files(_api_url: &str, id: &str) -> Result<String, String> {
    server_clear_download_files(id.to_string()).await.map_err(|e| e.to_string())
}

pub async fn stop_download(_api_url: &str, id: &str) -> Result<String, String> {
    server_stop_download(id.to_string()).await.map_err(|e| e.to_string())
}

pub async fn resume_download(_api_url: &str, id: &str) -> Result<String, String> {
    server_resume_download(id.to_string()).await.map_err(|e| e.to_string())
}

pub async fn save_upload(_api_url: &str, payload: &UploadTemplate) -> Result<(), String> {
    server_save_upload(payload.clone()).await.map_err(|e| e.to_string())
}

pub async fn delete_upload(_api_url: &str, id: &str) -> Result<(), String> {
    server_delete_upload(id.to_string()).await.map_err(|e| e.to_string())
}

pub async fn start_qr_login(_api_url: &str) -> Result<QrStartResponse, String> {
    server_start_qr_login().await.map_err(|e| e.to_string())
}

pub async fn confirm_qr_login(_api_url: &str, session_id: String) -> Result<(), String> {
    server_confirm_qr_login(session_id).await.map_err(|e| e.to_string())
}

pub async fn rename_account(_api_url: &str, account_file: String, display_name: String) {
    let _ = server_rename_account(account_file, display_name).await;
}

pub async fn delete_account(_api_url: &str, account_file: String) {
    let _ = server_delete_account(account_file).await;
}

pub async fn fetch_recording_settings(_api_url: &str) -> Option<RecordingSettings> {
    server_fetch_recording_settings().await.ok()
}

pub async fn fetch_storage_stats(_api_url: &str) -> Option<StorageStats> {
    server_fetch_storage_stats().await.ok()
}

pub async fn save_recording_settings(
    _api_url: &str,
    settings: &RecordingSettings,
) -> Result<(), String> {
    server_save_recording_settings(settings.clone()).await.map_err(|e| e.to_string())
}

pub async fn trigger_manual_upload(_api_url: &str, id: &str) -> Result<String, String> {
    server_trigger_manual_upload(id.to_string()).await.map_err(|e| e.to_string())
}
