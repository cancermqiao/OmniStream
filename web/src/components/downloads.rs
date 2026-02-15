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
) -> Element {
    let mut search = use_signal(String::new);
    let mut sort_asc = use_signal(|| true);
    let mut selected_ids = use_signal::<Vec<String>>(Vec::new);
    let mut bind_upload_ids = use_signal::<Vec<String>>(Vec::new);

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
                        onclick: move |_| {
                            on_batch_bind_uploads.call((selected_ids(), bind_upload_ids()));
                            selected_ids.set(vec![]);
                        },
                        "应用"
                    }
                }
            }

            div { class: "card",
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
                                let checked = selected_ids().contains(&d.id);
                                let status_label =
                                    d.current_status.clone().unwrap_or_else(|| "未知".to_string());
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
                                        td { class: "mono", "{d.url}" }
                                        td {
                                            span { class: "tag", "{status_label}" }
                                        }
                                        td {
                                            {
                                                d.linked_upload_ids.iter().filter_map(|id| {
                                                    uploads.iter().find(|u| &u.id == id).map(|u| rsx! {
                                                        span { class: "tag", "{u.name}" }
                                                    })
                                                })
                                            }
                                        }
                                        td { class: "actions",
                                            button { class: "btn btn-ghost", onclick: move |_| on_edit.call(d_for_edit.clone()), "编辑" }
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
