use dioxus::prelude::*;
use shared::RecordingSettings;

const QUALITY_OPTIONS: &[&str] =
    &["best", "worst", "1080p60", "1080p", "720p60", "720p", "480p", "360p"];

#[component]
pub fn SettingsPage(
    settings: RecordingSettings,
    save_message: Option<String>,
    save_error: bool,
    on_save: EventHandler<RecordingSettings>,
) -> Element {
    let mut segment_size_mb =
        use_signal(|| settings.segment_size_mb.map(|v| v.to_string()).unwrap_or_default());
    let mut segment_time_sec =
        use_signal(|| settings.segment_time_sec.map(|v| v.to_string()).unwrap_or_default());

    let mut bilibili = use_signal(|| settings.quality.bilibili.clone());
    let mut douyu = use_signal(|| settings.quality.douyu.clone());
    let mut huya = use_signal(|| settings.quality.huya.clone());
    let mut twitch = use_signal(|| settings.quality.twitch.clone());
    let mut youtube = use_signal(|| settings.quality.youtube.clone());
    let mut default_quality = use_signal(|| settings.quality.default_quality.clone());
    let mut auto_cleanup_after_upload = use_signal(|| settings.auto_cleanup_after_upload);
    let mut form_error = use_signal::<Option<String>>(|| None);
    let segment_size_label =
        settings.segment_size_mb.map(|v| format!("{v} MB")).unwrap_or_else(|| "未限制".to_string());
    let segment_time_label = settings
        .segment_time_sec
        .map(|v| format!("{v} 秒"))
        .unwrap_or_else(|| "未限制".to_string());
    let cleanup_label = if settings.auto_cleanup_after_upload { "已开启" } else { "未开启" };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "录制设置" }
                    p { "设置各平台录播画质与单文件分片大小（按体积或时长）。" }
                }
            }

            div { class: "stat-grid",
                div { class: "stat-card",
                    p { class: "stat-label", "体积分片" }
                    p { class: "stat-value", "{segment_size_label}" }
                    p { class: "stat-hint", "留空表示不按体积切分" }
                }
                div { class: "stat-card",
                    p { class: "stat-label", "时长分片" }
                    p { class: "stat-value", "{segment_time_label}" }
                    p { class: "stat-hint", "留空表示不按时长切分" }
                }
                div { class: "stat-card",
                    p { class: "stat-label", "默认画质" }
                    p { class: "stat-value", "{settings.quality.default_quality}" }
                    p { class: "stat-hint", "平台未单独配置时使用" }
                }
                div { class: "stat-card",
                    p { class: "stat-label", "上传后清理" }
                    p { class: "stat-value", "{cleanup_label}" }
                    p { class: "stat-hint", "节省本地录制空间" }
                }
            }

            div { class: "card",
                p { class: "section-title", "分片策略" }
                div { class: "grid-2",
                    div { class: "field",
                        label { "单文件分片大小（MB，可留空）" }
                        input {
                            class: "input",
                            value: "{segment_size_mb}",
                            placeholder: "例如 1024",
                            oninput: move |e| segment_size_mb.set(e.value()),
                        }
                    }
                    div { class: "field",
                        label { "单文件分片时长（秒，可留空）" }
                        input {
                            class: "input",
                            value: "{segment_time_sec}",
                            placeholder: "例如 3600",
                            oninput: move |e| segment_time_sec.set(e.value()),
                        }
                    }
                }

                p { class: "section-title", "平台画质（streamlink quality）" }
                div { class: "grid-2",
                    QualitySelect { label: "Bilibili 画质".to_string(), value: bilibili, on_change: move |v| bilibili.set(v) }
                    QualitySelect { label: "斗鱼 画质".to_string(), value: douyu, on_change: move |v| douyu.set(v) }
                    QualitySelect { label: "虎牙 画质".to_string(), value: huya, on_change: move |v| huya.set(v) }
                    QualitySelect { label: "Twitch 画质".to_string(), value: twitch, on_change: move |v| twitch.set(v) }
                    QualitySelect { label: "YouTube 画质".to_string(), value: youtube, on_change: move |v| youtube.set(v) }
                    QualitySelect { label: "默认画质".to_string(), value: default_quality, on_change: move |v| default_quality.set(v) }
                }

                p { class: "label", "可选值：best、worst、1080p60、1080p、720p60、720p、480p、360p。" }

                p { class: "section-title", "上传后处理" }
                label { class: "mini-check",
                    input {
                        r#type: "checkbox",
                        checked: auto_cleanup_after_upload(),
                        onchange: move |_| auto_cleanup_after_upload.set(!auto_cleanup_after_upload()),
                    }
                    span { "上传全部成功后自动删除本地录制文件（释放空间）" }
                }

                if let Some(err) = form_error() {
                    p { class: "status-banner status-error", "{err}" }
                } else if let Some(msg) = save_message.clone() {
                    p {
                        class: if save_error { "status-banner status-error" } else { "status-banner" },
                        "{msg}"
                    }
                }

                div { class: "inline-actions",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            let parse_opt_u64 = |label: &str, v: String| -> Result<Option<u64>, String> {
                                let t = v.trim();
                                if t.is_empty() {
                                    Ok(None)
                                } else {
                                    t.parse::<u64>()
                                        .map(Some)
                                        .map_err(|_| format!("{label}只能填写非负整数或留空"))
                                }
                            };
                            let segment_size = match parse_opt_u64("单文件分片大小", segment_size_mb()) {
                                Ok(v) => v,
                                Err(message) => {
                                    form_error.set(Some(message));
                                    return;
                                }
                            };
                            let segment_time = match parse_opt_u64("单文件分片时长", segment_time_sec()) {
                                Ok(v) => v,
                                Err(message) => {
                                    form_error.set(Some(message));
                                    return;
                                }
                            };

                            form_error.set(None);
                            on_save.call(RecordingSettings {
                                segment_size_mb: segment_size,
                                segment_time_sec: segment_time,
                                quality: shared::PlatformQualityConfig {
                                    bilibili: bilibili(),
                                    douyu: douyu(),
                                    huya: huya(),
                                    twitch: twitch(),
                                    youtube: youtube(),
                                    default_quality: default_quality(),
                                },
                                auto_cleanup_after_upload: auto_cleanup_after_upload(),
                            });
                        },
                        "保存设置"
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
