use dioxus::prelude::*;
use shared::{DownloadConfig, UploadTemplate};

#[component]
pub fn DownloadsPage(
    downloads: Vec<DownloadConfig>,
    uploads: Vec<UploadTemplate>,
    on_create: EventHandler<()>,
    on_edit: EventHandler<DownloadConfig>,
    on_delete: EventHandler<String>,
    on_batch_delete: EventHandler<Vec<String>>,
    on_batch_bind_uploads: EventHandler<(Vec<String>, Vec<String>)>,
    on_manual_upload: EventHandler<String>,
    on_stop: EventHandler<String>,
    on_resume: EventHandler<String>,
    manual_upload_message: Option<String>,
    manual_upload_error: bool,
) -> Element {
    let mut search = use_signal(String::new);
    let mut sort_asc = use_signal(|| true);
    let mut selected_ids = use_signal::<Vec<String>>(Vec::new);
    let mut bind_upload_ids = use_signal::<Vec<String>>(Vec::new);

    let total_count = downloads.len();
    let linked_count = downloads.iter().filter(|d| !d.linked_upload_ids.is_empty()).count();
    let active_count = downloads
        .iter()
        .filter(|d| {
            matches!(d.current_status.as_deref(), Some("下载中") | Some("上传中") | Some("检测中"))
        })
        .count();
    let stopped_count = downloads.iter().filter(|d| !d.enabled).count();
    let keyword = search().to_lowercase();
    let mut rows: Vec<DownloadConfig> = downloads
        .into_iter()
        .filter(|d| {
            keyword.is_empty()
                || d.name.to_lowercase().contains(&keyword)
                || d.url.to_lowercase().contains(&keyword)
        })
        .collect();
    rows.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    if !sort_asc() {
        rows.reverse();
    }
    let filtered_ids = rows.iter().map(|d| d.id.clone()).collect::<Vec<_>>();
    let rows_view = rows.clone();

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "下载任务" }
                    p { "管理直播间地址及关联上传任务。" }
                }
                button { class: "btn btn-primary", onclick: move |_| on_create.call(()), "新建下载任务" }
            }

            div { class: "stat-grid",
                div { class: "stat-card",
                    p { class: "stat-label", "下载任务" }
                    p { class: "stat-value", "{total_count}" }
                    p { class: "stat-hint", "全部录制入口" }
                }
                div { class: "stat-card",
                    p { class: "stat-label", "运行中" }
                    p { class: "stat-value", "{active_count}" }
                    p { class: "stat-hint", "下载 / 上传 / 检测" }
                }
                div { class: "stat-card",
                    p { class: "stat-label", "已关联上传" }
                    p { class: "stat-value", "{linked_count}" }
                    p { class: "stat-hint", "可一键转入上传流程" }
                }
                div { class: "stat-card",
                    p { class: "stat-label", "已停止" }
                    p { class: "stat-value", "{stopped_count}" }
                    p { class: "stat-hint", "暂停自动监听" }
                }
            }

            div { class: "card",
                div { class: "toolbar",
                    input {
                        class: "input",
                        placeholder: "搜索任务名或直播地址",
                        value: "{search}",
                        oninput: move |e| search.set(e.value()),
                    }
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| sort_asc.set(!sort_asc()),
                        if sort_asc() { "排序：名称正序" } else { "排序：名称倒序" }
                    }
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| selected_ids.set(filtered_ids.clone()),
                        "选择筛选结果"
                    }
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| selected_ids.set(vec![]),
                        "清空选择"
                    }
                    button {
                        class: "btn btn-danger",
                        disabled: selected_ids().is_empty(),
                        onclick: move |_| on_batch_delete.call(selected_ids()),
                        "批量删除"
                    }
                }
                div { class: "toolbar",
                    span { class: "toolbar-label", "批量绑定上传任务（已选 {selected_ids().len()} 项）" }
                    {
                        uploads.iter().map(|u| {
                            let uid = u.id.clone();
                            let checked = bind_upload_ids().contains(&uid);
                            rsx! {
                                label { class: "mini-check",
                                    input {
                                        r#type: "checkbox",
                                        checked,
                                        onchange: move |_| {
                                            let mut ids = bind_upload_ids();
                                            if ids.contains(&uid) {
                                                ids.retain(|v| v != &uid);
                                            } else {
                                                ids.push(uid.clone());
                                            }
                                            bind_upload_ids.set(ids);
                                        },
                                    }
                                    span { "{u.name}" }
                                }
                            }
                        })
                    }
                    button {
                        class: "btn btn-primary",
                        disabled: selected_ids().is_empty(),
                        onclick: move |_| {
                            on_batch_bind_uploads.call((selected_ids(), bind_upload_ids()));
                            selected_ids.set(vec![]);
                        },
                        "应用"
                    }
                }
                if let Some(msg) = manual_upload_message.clone() {
                    p {
                        class: if manual_upload_error { "status-banner status-error" } else { "status-banner" },
                        "{msg}"
                    }
                }
            }

            div { class: "card",
                div { class: "table-wrap",
                    table { class: "table",
                        thead {
                            tr {
                                th { "选择" }
                                th { "任务名称" }
                                th { "直播地址" }
                                th { "当前状态" }
                                th { "关联上传任务" }
                                th { class: "actions", "操作" }
                            }
                        }
                        tbody {
                            if rows_view.is_empty() {
                                tr { td { colspan: "6", class: "empty", "暂无下载任务" } }
                            }
                            {
                                rows_view.into_iter().map(|d| {
                                    let d_for_edit = d.clone();
                                    let d_id = d.id.clone();
                                    let d_id_for_check = d_id.clone();
                                    let d_id_for_delete = d_id.clone();
                                    let d_id_for_manual_upload = d_id.clone();
                                    let d_id_for_stop = d_id.clone();
                                    let d_id_for_resume = d_id.clone();
                                    let checked = selected_ids().contains(&d.id);
                                    let status_label =
                                        d.current_status.clone().unwrap_or_else(|| "未知".to_string());
                                    let status_class = status_class(&status_label);
                                    let can_stop = matches!(status_label.as_str(), "下载中" | "上传中" | "检测中");
                                    rsx! {
                                        tr {
                                            td {
                                                input {
                                                    r#type: "checkbox",
                                                    checked,
                                                    onchange: move |_| {
                                                        let mut ids = selected_ids();
                                                        if ids.contains(&d_id_for_check) {
                                                            ids.retain(|v| v != &d_id_for_check);
                                                        } else {
                                                            ids.push(d_id_for_check.clone());
                                                        }
                                                        selected_ids.set(ids);
                                                    },
                                                }
                                            }
                                            td { "{d.name}" }
                                            td { class: "mono text-ellipsis", title: "{d.url}", "{d.url}" }
                                            td {
                                                span { class: "{status_class}", "{status_label}" }
                                            }
                                            td {
                                                if d.linked_upload_ids.is_empty() {
                                                    span { class: "muted", "未关联" }
                                                } else {
                                                    {
                                                        d.linked_upload_ids.iter().filter_map(|id| {
                                                            uploads.iter().find(|u| &u.id == id).map(|u| rsx! {
                                                                span { class: "tag tag-info", "{u.name}" }
                                                            })
                                                        })
                                                    }
                                                }
                                            }
                                            td { class: "actions",
                                                button { class: "btn btn-ghost", onclick: move |_| on_edit.call(d_for_edit.clone()), "编辑" }
                                                if !d.enabled {
                                                    button { class: "btn btn-primary", onclick: move |_| on_resume.call(d_id_for_resume.clone()), "恢复监听" }
                                                } else {
                                                    button {
                                                        class: "btn btn-warning",
                                                        disabled: !can_stop,
                                                        onclick: move |_| on_stop.call(d_id_for_stop.clone()),
                                                        "停止"
                                                    }
                                                }
                                                button { class: "btn btn-primary", onclick: move |_| on_manual_upload.call(d_id_for_manual_upload.clone()), "手动上传" }
                                                button { class: "btn btn-danger", onclick: move |_| on_delete.call(d_id_for_delete.clone()), "删除" }
                                            }
                                        }
                                    }
                                })
                            }
                        }
                    }
                }
            }
        }
    }
}

fn status_class(label: &str) -> &'static str {
    match label {
        "下载中" | "上传中" => "tag tag-success",
        "检测中" => "tag tag-info",
        "失败" => "tag tag-danger",
        "已完成" => "tag tag-success",
        "已停止" => "tag tag-warning",
        _ => "tag",
    }
}
