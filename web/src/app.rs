use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use shared::{DownloadConfig, UploadTemplate};
use std::collections::HashSet;

use crate::api;
use crate::components::{
    AccountsPage, DownloadModal, DownloadsPage, SettingsPage, TabItem, UploadModal, UploadsPage,
};
use crate::models::{AppData, QrStartResponse, Tab};
use crate::styles::theme_css;

pub fn App() -> Element {
    let api_url = api_url();

    let mut active_tab = use_signal(|| Tab::Downloads);
    let mut data = use_signal(AppData::default);

    let mut editing_download = use_signal::<Option<DownloadConfig>>(|| None);
    let mut editing_upload = use_signal::<Option<UploadTemplate>>(|| None);

    let mut qr_session = use_signal::<Option<QrStartResponse>>(|| None);
    let mut qr_message = use_signal::<Option<String>>(|| None);

    use_future(move || async move {
        let mut account_tick = 0u8;
        let mut settings_tick = 0u8;
        loop {
            let mut next = data();

            if let Some(v) = api::fetch_downloads(api_url).await {
                next.downloads = v;
            }
            if let Some(v) = api::fetch_uploads(api_url).await {
                next.uploads = v;
            }
            if account_tick == 0
                && let Some(v) = api::fetch_accounts(api_url).await
            {
                next.accounts = v;
            }
            account_tick = (account_tick + 1) % 10;

            if settings_tick == 0
                && let Some(v) = api::fetch_recording_settings(api_url).await
            {
                next.recording_settings = v;
            }
            settings_tick = (settings_tick + 1) % 10;

            data.set(next);
            TimeoutFuture::new(2000).await;
        }
    });

    let snapshot = data();

    rsx! {
        document::Link { rel: "icon", href: "/assets/favicon.svg", r#type: "image/svg+xml" }
        div { class: "app-shell",
            style { "{theme_css()}" }

            div { class: "layout",
                aside { class: "sidebar",
                    div { class: "brand", "OmniStream" }
                    p { class: "subtitle", "直播录制与上传工作流" }

                    TabItem {
                        active: active_tab() == Tab::Downloads,
                        label: "下载任务",
                        onclick: move |_| active_tab.set(Tab::Downloads),
                    }
                    TabItem {
                        active: active_tab() == Tab::Accounts,
                        label: "账号管理",
                        onclick: move |_| active_tab.set(Tab::Accounts),
                    }
                    TabItem {
                        active: active_tab() == Tab::Uploads,
                        label: "上传任务",
                        onclick: move |_| active_tab.set(Tab::Uploads),
                    }
                    TabItem {
                        active: active_tab() == Tab::Settings,
                        label: "录制设置",
                        onclick: move |_| active_tab.set(Tab::Settings),
                    }
                }

                main { class: "content",
                    match active_tab() {
                        Tab::Downloads => rsx! {
                            DownloadsPage {
                                downloads: snapshot.downloads.clone(),
                                uploads: snapshot.uploads.clone(),
                                on_create: move |_| editing_download.set(Some(DownloadConfig::default())),
                                on_edit: move |d| editing_download.set(Some(d)),
                                on_delete: move |id: String| async move {
                                    api::delete_download(api_url, &id).await;
                                },
                                on_batch_delete: move |ids: Vec<String>| async move {
                                    for id in ids {
                                        api::delete_download(api_url, &id).await;
                                    }
                                    if let Some(v) = api::fetch_downloads(api_url).await {
                                        let mut next = data();
                                        next.downloads = v;
                                        data.set(next);
                                    }
                                },
                                on_batch_bind_uploads: move |(download_ids, upload_ids): (Vec<String>, Vec<String>)| async move {
                                    if download_ids.is_empty() {
                                        return;
                                    }
                                    let target_ids = download_ids.into_iter().collect::<HashSet<_>>();
                                    let current = data().downloads;
                                    for mut d in current {
                                        if target_ids.contains(&d.id) {
                                            d.linked_upload_ids = upload_ids.clone();
                                            api::save_download(api_url, &d).await;
                                        }
                                    }
                                    if let Some(v) = api::fetch_downloads(api_url).await {
                                        let mut next = data();
                                        next.downloads = v;
                                        data.set(next);
                                    }
                                },
                                on_manual_upload: move |id: String| async move {
                                    if let Err(e) = api::trigger_manual_upload(api_url, &id).await {
                                        #[cfg(target_arch = "wasm32")]
                                        web_sys::console::error_1(&format!("manual upload failed: {e}").into());
                                    }
                                    if let Some(v) = api::fetch_downloads(api_url).await {
                                        let mut next = data();
                                        next.downloads = v;
                                        data.set(next);
                                    }
                                },
                            }
                        },
                        Tab::Accounts => rsx! {
                            AccountsPage {
                                accounts: snapshot.accounts.clone(),
                                qr_session: qr_session(),
                                qr_message: qr_message(),
                                on_start_qr: move |_| async move {
                                    qr_message.set(Some("正在创建扫码会话...".to_string()));
                                    match api::start_qr_login(api_url).await {
                                        Ok(v) => {
                                            qr_session.set(Some(v));
                                            qr_message.set(Some(
                                                "请使用哔哩哔哩 App 扫码后，点击“确认登录”。".to_string(),
                                            ));
                                        }
                                        Err(e) => {
                                            qr_message.set(Some(format!("创建扫码会话失败：{e}")))
                                        }
                                    }
                                },
                                on_confirm_qr: move |_| async move {
                                    let Some(session) = qr_session() else {
                                        qr_message.set(Some("当前没有可用的扫码会话。".to_string()));
                                        return;
                                    };
                                    qr_message.set(Some("正在确认登录...".to_string()));

                                    match api::confirm_qr_login(api_url, session.session_id.clone()).await {
                                        Ok(()) => {
                                            qr_message.set(Some("登录成功，账号已保存。".to_string()));
                                            qr_session.set(None);
                                            if let Some(accounts) = api::fetch_accounts(api_url).await {
                                                let mut next = data();
                                                next.accounts = accounts;
                                                data.set(next);
                                            }
                                        }
                                        Err(e) => qr_message.set(Some(format!("确认登录失败：{e}"))),
                                    }
                                },
                                on_reset_qr: move |_| {
                                    qr_session.set(None);
                                    qr_message.set(None);
                                },
                                on_rename: move |(account_file, name)| async move {
                                    api::rename_account(api_url, account_file, name).await;
                                    if let Some(accounts) = api::fetch_accounts(api_url).await {
                                        let mut next = data();
                                        next.accounts = accounts;
                                        data.set(next);
                                    }
                                },
                                on_delete: move |account_file| async move {
                                    api::delete_account(api_url, account_file).await;
                                    if let Some(accounts) = api::fetch_accounts(api_url).await {
                                        let mut next = data();
                                        next.accounts = accounts;
                                        data.set(next);
                                    }
                                },
                            }
                        },
                        Tab::Uploads => rsx! {
                            UploadsPage {
                                uploads: snapshot.uploads.clone(),
                                accounts: snapshot.accounts.clone(),
                                on_create: move |_| editing_upload.set(Some(UploadTemplate::default())),
                                on_edit: move |u| editing_upload.set(Some(u)),
                                on_delete: move |id: String| async move {
                                    api::delete_upload(api_url, &id).await;
                                },
                                on_batch_delete: move |ids: Vec<String>| async move {
                                    for id in ids {
                                        api::delete_upload(api_url, &id).await;
                                    }
                                    if let Some(v) = api::fetch_uploads(api_url).await {
                                        let mut next = data();
                                        next.uploads = v;
                                        data.set(next);
                                    }
                                },
                            }
                        },
                        Tab::Settings => rsx! {
                            SettingsPage {
                                settings: snapshot.recording_settings.clone(),
                                on_save: move |settings| async move {
                                    api::save_recording_settings(api_url, &settings).await;
                                    if let Some(v) = api::fetch_recording_settings(api_url).await {
                                        let mut next = data();
                                        next.recording_settings = v;
                                        data.set(next);
                                    }
                                },
                            }
                        },
                    }
                }
            }

            if let Some(config) = editing_download() {
                DownloadModal {
                    config: config.clone(),
                    uploads: snapshot.uploads.clone(),
                    on_close: move |_| editing_download.set(None),
                    on_save: move |payload| async move {
                        api::save_download(api_url, &payload).await;
                        editing_download.set(None);
                    },
                }
            }

            if let Some(template) = editing_upload() {
                UploadModal {
                    template: template.clone(),
                    accounts: snapshot.accounts.clone(),
                    on_close: move |_| editing_upload.set(None),
                    on_save: move |payload| async move {
                        api::save_upload(api_url, &payload).await;
                        editing_upload.set(None);
                    },
                }
            }
        }
    }
}

pub fn api_url() -> &'static str {
    option_env!("BILIUP_API_URL").unwrap_or(default_api_url())
}

const fn default_api_url() -> &'static str {
    #[cfg(target_arch = "wasm32")]
    {
        "/api"
    }
    #[cfg(target_os = "android")]
    {
        "http://10.0.2.2:3000/api"
    }
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    {
        "http://127.0.0.1:3000/api"
    }
}
