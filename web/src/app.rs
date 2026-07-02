use dioxus::prelude::*;
use shared::{DownloadConfig, UploadTemplate};

use crate::api;
use crate::components::{
    AccountsPage, DownloadModal, DownloadsPage, SettingsPage, TabItem, UploadModal, UploadsPage,
};
use crate::models::{AppData, QrStartResponse, Tab};
#[cfg(target_arch = "wasm32")]
use crate::sleep_ms;
use crate::styles::theme_css;

pub fn App() -> Element {
    let api_url = api_url();

    let mut active_tab = use_signal(|| Tab::Downloads);
    let mut sidebar_collapsed = use_signal(|| false);
    let mut data = use_signal(AppData::default);

    let mut editing_download = use_signal::<Option<DownloadConfig>>(|| None);
    let mut editing_upload = use_signal::<Option<UploadTemplate>>(|| None);
    let mut download_modal_error = use_signal::<Option<String>>(|| None);
    let mut upload_modal_error = use_signal::<Option<String>>(|| None);
    let mut settings_message = use_signal::<Option<String>>(|| None);
    let mut settings_error = use_signal(|| false);
    let mut operation_message = use_signal::<Option<String>>(|| None);
    let mut operation_error = use_signal(|| false);

    let mut qr_session = use_signal::<Option<QrStartResponse>>(|| None);
    let mut qr_message = use_signal::<Option<String>>(|| None);
    let mut manual_upload_message = use_signal::<Option<String>>(|| None);
    let mut manual_upload_error = use_signal(|| false);

    #[cfg(target_arch = "wasm32")]
    {
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
                sleep_ms(2000).await;
            }
        });

        use_effect(move || {
            let Some(current_message) = operation_message() else {
                return;
            };
            spawn(async move {
                sleep_ms(5000).await;
                if operation_message() == Some(current_message) {
                    operation_message.set(None);
                }
            });
        });
    }

    let snapshot = data();

    rsx! {
        document::Link { rel: "icon", href: "/assets/favicon.svg", r#type: "image/svg+xml" }
        div { class: "app-shell",
            style { "{theme_css()}" }
            if let Some(msg) = operation_message() {
                div { class: "toast-layer",
                    p {
                        class: if operation_error() { "status-banner toast status-error" } else { "status-banner toast" },
                        "{msg}"
                    }
                }
            }

            div { class: if sidebar_collapsed() { "layout layout-collapsed" } else { "layout" },
                aside { class: if sidebar_collapsed() { "sidebar sidebar-collapsed" } else { "sidebar" },
                    div { class: "sidebar-top",
                        div {
                            div { class: "brand", "OmniStream" }
                            if !sidebar_collapsed() {
                                p { class: "subtitle", "直播录制与上传工作流" }
                            }
                        }
                        button {
                            class: "sidebar-toggle",
                            title: if sidebar_collapsed() { "展开侧边栏" } else { "收起侧边栏" },
                            onclick: move |_| sidebar_collapsed.set(!sidebar_collapsed()),
                            if sidebar_collapsed() { ">" } else { "<" }
                        }
                    }

                    TabItem {
                        active: active_tab() == Tab::Downloads,
                        label: "下载任务",
                        icon: "↓",
                        compact: sidebar_collapsed(),
                        onclick: move |_| active_tab.set(Tab::Downloads),
                    }
                    TabItem {
                        active: active_tab() == Tab::Accounts,
                        label: "账号管理",
                        icon: "◎",
                        compact: sidebar_collapsed(),
                        onclick: move |_| active_tab.set(Tab::Accounts),
                    }
                    TabItem {
                        active: active_tab() == Tab::Uploads,
                        label: "上传任务",
                        icon: "↑",
                        compact: sidebar_collapsed(),
                        onclick: move |_| active_tab.set(Tab::Uploads),
                    }
                    TabItem {
                        active: active_tab() == Tab::Settings,
                        label: "录制设置",
                        icon: "⚙",
                        compact: sidebar_collapsed(),
                        onclick: move |_| active_tab.set(Tab::Settings),
                    }
                }

                main { class: "content",
                    match active_tab() {
                        Tab::Downloads => rsx! {
                            DownloadsPage {
                                downloads: snapshot.downloads.clone(),
                                uploads: snapshot.uploads.clone(),
                                on_create: move |_| {
                                    download_modal_error.set(None);
                                    editing_download.set(Some(DownloadConfig::default()));
                                },
                                on_edit: move |d| {
                                    download_modal_error.set(None);
                                    editing_download.set(Some(d));
                                },
                                on_delete: move |id: String| async move {
                                    match api::delete_download(api_url, &id).await {
                                        Ok(()) => {
                                            operation_message.set(Some("下载任务已删除。".to_string()));
                                            operation_error.set(false);
                                            if let Some(v) = api::fetch_downloads(api_url).await {
                                                let mut next = data();
                                                next.downloads = v;
                                                data.set(next);
                                            }
                                        }
                                        Err(e) => {
                                            operation_message.set(Some(format!("删除下载任务失败：{e}")));
                                            operation_error.set(true);
                                        }
                                    }
                                },
                                on_batch_delete: move |ids: Vec<String>| async move {
                                    let mut failed = Vec::new();
                                    for id in ids {
                                        if let Err(e) = api::delete_download(api_url, &id).await {
                                            failed.push(e);
                                        }
                                    }
                                    if failed.is_empty() {
                                        operation_message.set(Some("批量删除下载任务完成。".to_string()));
                                        operation_error.set(false);
                                    } else {
                                        operation_message.set(Some(format!("部分下载任务删除失败：{}", failed.join("；"))));
                                        operation_error.set(true);
                                    }
                                    if let Some(v) = api::fetch_downloads(api_url).await {
                                        let mut next = data();
                                        next.downloads = v;
                                        data.set(next);
                                    }
                                },
                                on_manual_upload: move |id: String| async move {
                                    manual_upload_message.set(Some("正在触发手动上传...".to_string()));
                                    manual_upload_error.set(false);

                                    match api::trigger_manual_upload(api_url, &id).await {
                                        Ok(msg) => {
                                            manual_upload_message.set(Some(format!("手动上传已触发：{msg}")));
                                            manual_upload_error.set(false);
                                        }
                                        Err(e) => {
                                            manual_upload_message.set(Some(format!("手动上传触发失败：{e}")));
                                            manual_upload_error.set(true);
                                            #[cfg(target_arch = "wasm32")]
                                            web_sys::console::error_1(&format!("manual upload failed: {e}").into());
                                        }
                                    }

                                    if let Some(v) = api::fetch_downloads(api_url).await {
                                        let mut next = data();
                                        next.downloads = v;
                                        data.set(next);
                                    }
                                },
                                on_stop: move |(id, name): (String, String)| async move {
                                    operation_message.set(Some(format!("正在停止下载任务「{name}」...")));
                                    operation_error.set(false);

                                    match api::stop_download(api_url, &id).await {
                                        Ok(_) => {
                                            operation_message.set(Some(format!("下载任务「{name}」已停止，并已暂停自动监听。")));
                                            operation_error.set(false);
                                        }
                                        Err(e) => {
                                            operation_message.set(Some(format!("停止下载任务「{name}」失败：{e}")));
                                            operation_error.set(true);
                                        }
                                    }

                                    if let Some(v) = api::fetch_downloads(api_url).await {
                                        let mut next = data();
                                        next.downloads = v;
                                        data.set(next);
                                    }
                                },
                                on_resume: move |(id, name): (String, String)| async move {
                                    operation_message.set(Some(format!("正在恢复下载任务「{name}」监听...")));
                                    operation_error.set(false);

                                    match api::resume_download(api_url, &id).await {
                                        Ok(_) => {
                                            operation_message.set(Some(format!("下载任务「{name}」已恢复监听。")));
                                            operation_error.set(false);
                                        }
                                        Err(e) => {
                                            operation_message.set(Some(format!("恢复下载任务「{name}」失败：{e}")));
                                            operation_error.set(true);
                                        }
                                    }

                                    if let Some(v) = api::fetch_downloads(api_url).await {
                                        let mut next = data();
                                        next.downloads = v;
                                        data.set(next);
                                    }
                                },
                                on_clear_files: move |(id, name): (String, String)| async move {
                                    operation_message.set(Some(format!("正在清空下载任务「{name}」的本地文件...")));
                                    operation_error.set(false);

                                    match api::clear_download_files(api_url, &id).await {
                                        Ok(_) => {
                                            operation_message.set(Some(format!("下载任务「{name}」的本地文件已清空。")));
                                            operation_error.set(false);
                                        }
                                        Err(e) => {
                                            operation_message.set(Some(format!("清空下载任务「{name}」文件失败：{e}")));
                                            operation_error.set(true);
                                        }
                                    }
                                },
                                manual_upload_message: manual_upload_message(),
                                manual_upload_error: manual_upload_error(),
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
                                on_create: move |_| {
                                    upload_modal_error.set(None);
                                    editing_upload.set(Some(UploadTemplate::default()));
                                },
                                on_edit: move |u| {
                                    upload_modal_error.set(None);
                                    editing_upload.set(Some(u));
                                },
                                on_delete: move |id: String| async move {
                                    match api::delete_upload(api_url, &id).await {
                                        Ok(()) => {
                                            operation_message.set(Some("上传任务已删除，并已清理下载任务关联。".to_string()));
                                            operation_error.set(false);
                                            if let Some(v) = api::fetch_uploads(api_url).await {
                                                let mut next = data();
                                                next.uploads = v;
                                                data.set(next);
                                            }
                                            if let Some(v) = api::fetch_downloads(api_url).await {
                                                let mut next = data();
                                                next.downloads = v;
                                                data.set(next);
                                            }
                                        }
                                        Err(e) => {
                                            operation_message.set(Some(format!("删除上传任务失败：{e}")));
                                            operation_error.set(true);
                                        }
                                    }
                                },
                                on_batch_delete: move |ids: Vec<String>| async move {
                                    let mut failed = Vec::new();
                                    for id in ids {
                                        if let Err(e) = api::delete_upload(api_url, &id).await {
                                            failed.push(e);
                                        }
                                    }
                                    if failed.is_empty() {
                                        operation_message.set(Some("批量删除上传任务完成，并已清理下载任务关联。".to_string()));
                                        operation_error.set(false);
                                    } else {
                                        operation_message.set(Some(format!("部分上传任务删除失败：{}", failed.join("；"))));
                                        operation_error.set(true);
                                    }
                                    if let Some(v) = api::fetch_uploads(api_url).await {
                                        let mut next = data();
                                        next.uploads = v;
                                        data.set(next);
                                    }
                                    if let Some(v) = api::fetch_downloads(api_url).await {
                                        let mut next = data();
                                        next.downloads = v;
                                        data.set(next);
                                    }
                                },
                            }
                        },
                        Tab::Settings => rsx! {
                            SettingsPage {
                                settings: snapshot.recording_settings.clone(),
                                save_message: settings_message(),
                                save_error: settings_error(),
                                on_save: move |settings| async move {
                                    match api::save_recording_settings(api_url, &settings).await {
                                        Ok(()) => {
                                            settings_message.set(Some("录制设置已保存。".to_string()));
                                            settings_error.set(false);
                                            if let Some(v) = api::fetch_recording_settings(api_url).await {
                                                let mut next = data();
                                                next.recording_settings = v;
                                                data.set(next);
                                            }
                                        }
                                        Err(e) => {
                                            settings_message.set(Some(format!("录制设置保存失败：{e}")));
                                            settings_error.set(true);
                                        }
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
                    save_error: download_modal_error(),
                    on_close: move |_| editing_download.set(None),
                    on_save: move |payload| async move {
                        match api::save_download(api_url, &payload).await {
                            Ok(()) => {
                                download_modal_error.set(None);
                                editing_download.set(None);
                                operation_message.set(Some("下载任务已保存。".to_string()));
                                operation_error.set(false);
                                if let Some(v) = api::fetch_downloads(api_url).await {
                                    let mut next = data();
                                    next.downloads = v;
                                    data.set(next);
                                }
                            }
                            Err(e) => download_modal_error.set(Some(format!("保存下载任务失败：{e}"))),
                        }
                    },
                }
            }

            if let Some(template) = editing_upload() {
                UploadModal {
                    template: template.clone(),
                    accounts: snapshot.accounts.clone(),
                    save_error: upload_modal_error(),
                    on_close: move |_| editing_upload.set(None),
                    on_save: move |payload| async move {
                        match api::save_upload(api_url, &payload).await {
                            Ok(()) => {
                                upload_modal_error.set(None);
                                editing_upload.set(None);
                                operation_message.set(Some("上传任务已保存。".to_string()));
                                operation_error.set(false);
                                if let Some(v) = api::fetch_uploads(api_url).await {
                                    let mut next = data();
                                    next.uploads = v;
                                    data.set(next);
                                }
                            }
                            Err(e) => upload_modal_error.set(Some(format!("保存上传任务失败：{e}"))),
                        }
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
