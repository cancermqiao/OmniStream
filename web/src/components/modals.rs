use dioxus::prelude::*;
use shared::{
    DownloadConfig, PlatformQualityConfig, RecordingSettings, UploadAccount, UploadConfig,
    UploadTemplate,
};

use super::upload_taxonomy::tid_options;
const QUALITY_OPTIONS: &[&str] =
    &["best", "worst", "1080p60", "1080p", "720p60", "720p", "480p", "360p"];

#[component]
pub fn DownloadModal(
    config: DownloadConfig,
    uploads: Vec<UploadTemplate>,
    on_close: EventHandler<()>,
    on_save: EventHandler<DownloadConfig>,
) -> Element {
    let mut name = use_signal(|| config.name.clone());
    let mut url = use_signal(|| config.url.clone());
    let mut linked = use_signal(|| config.linked_upload_ids.clone());
    let base_settings = config.recording_settings.clone().unwrap_or_default();
    let mut use_custom_recording_settings = use_signal(|| config.use_custom_recording_settings);
    let mut segment_size_mb =
        use_signal(|| base_settings.segment_size_mb.map(|v| v.to_string()).unwrap_or_default());
    let mut segment_time_sec =
        use_signal(|| base_settings.segment_time_sec.map(|v| v.to_string()).unwrap_or_default());
    let mut auto_cleanup_after_upload = use_signal(|| base_settings.auto_cleanup_after_upload);
    let mut q_bilibili = use_signal(|| base_settings.quality.bilibili.clone());
    let mut q_douyu = use_signal(|| base_settings.quality.douyu.clone());
    let mut q_huya = use_signal(|| base_settings.quality.huya.clone());
    let mut q_twitch = use_signal(|| base_settings.quality.twitch.clone());
    let mut q_youtube = use_signal(|| base_settings.quality.youtube.clone());
    let mut q_default = use_signal(|| base_settings.quality.default_quality.clone());

    rsx! {
        div { class: "modal-wrap",
            div { class: "modal-mask", onclick: move |_| on_close.call(()) }
            div { class: "modal",
                h3 { "下载任务" }
                div { class: "field",
                    label { "任务名称" }
                    input { class: "input", value: "{name}", oninput: move |e| name.set(e.value()) }
                }
                div { class: "field",
                    label { "直播地址" }
                    input { class: "input mono", value: "{url}", oninput: move |e| url.set(e.value()) }
                }
                div { class: "field",
                    label { "关联上传任务" }
                    div { class: "check-grid",
                        {
                            uploads.into_iter().map(|u| {
                                let uid = u.id.clone();
                                let checked = linked().contains(&uid);
                                rsx! {
                                    label { class: "check-item",
                                        input {
                                            r#type: "checkbox",
                                            checked,
                                            onchange: move |_| {
                                                let mut ids = linked();
                                                if ids.contains(&uid) {
                                                    ids.retain(|v| v != &uid);
                                                } else {
                                                    ids.push(uid.clone());
                                                }
                                                linked.set(ids);
                                            },
                                        }
                                        span { "{u.name}" }
                                    }
                                }
                            })
                        }
                    }
                }

                p { class: "section-title", "任务级录制设置" }
                label { class: "mini-check",
                    input {
                        r#type: "checkbox",
                        checked: use_custom_recording_settings(),
                        onchange: move |_| use_custom_recording_settings.set(!use_custom_recording_settings()),
                    }
                    span { "启用该下载任务的独立录制设置（未启用时使用全局录制设置）" }
                }

                if use_custom_recording_settings() {
                    div { class: "grid-2",
                        div { class: "field",
                            label { "单文件分片大小（MB，可留空）" }
                            input { class: "input", value: "{segment_size_mb}", oninput: move |e| segment_size_mb.set(e.value()) }
                        }
                        div { class: "field",
                            label { "单文件分片时长（秒，可留空）" }
                            input { class: "input", value: "{segment_time_sec}", oninput: move |e| segment_time_sec.set(e.value()) }
                        }
                    }
                    div { class: "grid-2",
                        QualitySelect { label: "Bilibili 画质".to_string(), value: q_bilibili, on_change: move |v| q_bilibili.set(v) }
                        QualitySelect { label: "斗鱼 画质".to_string(), value: q_douyu, on_change: move |v| q_douyu.set(v) }
                        QualitySelect { label: "虎牙 画质".to_string(), value: q_huya, on_change: move |v| q_huya.set(v) }
                        QualitySelect { label: "Twitch 画质".to_string(), value: q_twitch, on_change: move |v| q_twitch.set(v) }
                        QualitySelect { label: "YouTube 画质".to_string(), value: q_youtube, on_change: move |v| q_youtube.set(v) }
                        QualitySelect { label: "默认画质".to_string(), value: q_default, on_change: move |v| q_default.set(v) }
                    }
                    label { class: "mini-check",
                        input {
                            r#type: "checkbox",
                            checked: auto_cleanup_after_upload(),
                            onchange: move |_| auto_cleanup_after_upload.set(!auto_cleanup_after_upload()),
                        }
                        span { "上传全部成功后自动删除本地录制文件" }
                    }
                }

                div { class: "inline-actions",
                    button { class: "btn btn-ghost", onclick: move |_| on_close.call(()), "取消" }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            let parse_opt_u64 = |v: String| {
                                let t = v.trim();
                                if t.is_empty() { None } else { t.parse::<u64>().ok() }
                            };
                            let task_settings = RecordingSettings {
                                segment_size_mb: parse_opt_u64(segment_size_mb()),
                                segment_time_sec: parse_opt_u64(segment_time_sec()),
                                quality: PlatformQualityConfig {
                                    bilibili: q_bilibili(),
                                    douyu: q_douyu(),
                                    huya: q_huya(),
                                    twitch: q_twitch(),
                                    youtube: q_youtube(),
                                    default_quality: q_default(),
                                },
                                auto_cleanup_after_upload: auto_cleanup_after_upload(),
                            };
                            on_save.call(DownloadConfig {
                                id: config.id.clone(),
                                name: name(),
                                url: url(),
                                linked_upload_ids: linked(),
                                current_status: None,
                                use_custom_recording_settings: use_custom_recording_settings(),
                                recording_settings: if use_custom_recording_settings() {
                                    Some(task_settings)
                                } else {
                                    None
                                },
                            });
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

#[component]
fn QualitySelect(label: String, value: Signal<String>, on_change: EventHandler<String>) -> Element {
    let current = value();
    let has_current = QUALITY_OPTIONS.iter().any(|q| *q == current);
    rsx! {
        div { class: "field",
            label { "{label}" }
            select {
                class: "input",
                value: "{current}",
                onchange: move |e| on_change.call(e.value()),
                for q in QUALITY_OPTIONS {
                    option { value: "{q}", "{q}" }
                }
                if !has_current {
                    option { value: "{current}", "{current}" }
                }
            }
        }
    }
}

#[component]
pub fn UploadModal(
    template: UploadTemplate,
    accounts: Vec<UploadAccount>,
    on_close: EventHandler<()>,
    on_save: EventHandler<UploadTemplate>,
) -> Element {
    let mut name = use_signal(|| template.name.clone());
    let mut account_file = use_signal(|| template.config.account_file.clone());
    let mut title = use_signal(|| template.config.title.clone().unwrap_or_default());
    let mut tid = use_signal(|| template.config.tid);
    let mut copyright = use_signal(|| template.config.copyright);
    let mut tags = use_signal(|| template.config.tags.join(","));
    let mut description = use_signal(|| template.config.description.clone());
    let mut dynamic = use_signal(|| template.config.dynamic.clone());
    let mut form_error = use_signal::<Option<String>>(|| None);

    rsx! {
        div { class: "modal-wrap",
            div { class: "modal-mask", onclick: move |_| on_close.call(()) }
            div { class: "modal wide",
                h3 { "上传任务" }

                p { class: "section-title", "基础信息" }
                div { class: "field",
                    label { "任务名称（必填）" }
                    input { class: "input", value: "{name}", oninput: move |e| name.set(e.value()) }
                }

                p { class: "section-title", "账号信息" }
                div { class: "field",
                    label { "Bilibili 账号（必填）" }
                    select {
                        class: "input",
                        value: "{account_file}",
                        onchange: move |e| account_file.set(e.value()),
                        if accounts.is_empty() {
                            option { value: "", "暂无可用账号" }
                        }
                        for a in accounts.iter().filter(|a| a.valid) {
                            option { value: "{a.account_file}", "{a.name} ({a.mid.unwrap_or_default()})" }
                        }
                    }
                }

                p { class: "section-title", "投稿参数" }
                div { class: "grid-2",
                    div { class: "field",
                        label { "视频标题模板（必填）" }
                        input { class: "input", value: "{title}", oninput: move |e| title.set(e.value()) }
                        p { class: "label", "支持占位符：{title}（直播间标题）、%Y-%m-%d、%H:%M。示例：{title} 录播 %Y-%m-%d" }
                    }
                    div { class: "field",
                        label { "分区（必填）" }
                        select {
                            class: "input",
                            value: "{tid}",
                            onchange: move |e| {
                                if let Ok(v) = e.value().parse::<u16>() {
                                    tid.set(v);
                                }
                            },
                            for (id, label) in tid_options() {
                                option { value: "{id}", "{label}" }
                            }
                        }
                    }
                }
                div { class: "field",
                    label { "版权" }
                    select {
                        class: "input",
                        value: "{copyright}",
                        onchange: move |e| {
                            if let Ok(v) = e.value().parse::<u8>() {
                                copyright.set(v);
                            }
                        },
                        option { value: "1", "自制" }
                        option { value: "2", "转载" }
                    }
                }

                div { class: "field",
                    label { "标签（逗号分隔）" }
                    input { class: "input", value: "{tags}", oninput: move |e| tags.set(e.value()) }
                }

                div { class: "field",
                    label { "简介" }
                    textarea { class: "input", rows: "3", value: "{description}", oninput: move |e| description.set(e.value()) }
                }

                div { class: "field",
                    label { "动态" }
                    input { class: "input", value: "{dynamic}", oninput: move |e| dynamic.set(e.value()) }
                }

                if let Some(err) = form_error() {
                    p { class: "status status-error", "{err}" }
                }

                div { class: "inline-actions",
                    button { class: "btn btn-ghost", onclick: move |_| on_close.call(()), "取消" }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            let task_name = name().trim().to_string();
                            if task_name.is_empty() {
                                form_error.set(Some("请填写任务名称".to_string()));
                                return;
                            }

                            let title_template = title().trim().to_string();
                            if title_template.is_empty() {
                                form_error.set(Some("请填写视频标题模板".to_string()));
                                return;
                            }

                            if tid() == 0 {
                                form_error.set(Some("请选择有效分区".to_string()));
                                return;
                            }

                            let selected_account = if account_file().is_empty() {
                                accounts
                                    .iter()
                                    .find(|a| a.valid)
                                    .map(|a| a.account_file.clone())
                                    .unwrap_or_default()
                            } else {
                                account_file()
                            };
                            if selected_account.trim().is_empty() {
                                form_error.set(Some("请先在账号管理页添加并选择可用账号".to_string()));
                                return;
                            }

                            let tag_list = tags()
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect::<Vec<_>>();

                            form_error.set(None);
                            on_save.call(UploadTemplate {
                                id: template.id.clone(),
                                name: task_name,
                                config: UploadConfig {
                                    title: Some(title_template),
                                    tags: tag_list,
                                    tid: tid(),
                                    copyright: copyright(),
                                    description: description(),
                                    dynamic: dynamic(),
                                    account_file: selected_account,
                                    ..Default::default()
                                },
                            });
                        },
                        "保存"
                    }
                }
            }
        }
    }
}
