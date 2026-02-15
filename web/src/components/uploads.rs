use dioxus::prelude::*;
use shared::{UploadAccount, UploadTemplate};

use super::upload_taxonomy::tid_name;

#[component]
pub fn UploadsPage(
    uploads: Vec<UploadTemplate>,
    accounts: Vec<UploadAccount>,
    on_create: EventHandler<()>,
    on_edit: EventHandler<UploadTemplate>,
    on_delete: EventHandler<String>,
    on_batch_delete: EventHandler<Vec<String>>,
) -> Element {
    let mut search = use_signal(String::new);
    let mut sort_asc = use_signal(|| true);
    let mut selected_ids = use_signal::<Vec<String>>(Vec::new);

    let keyword = search().to_lowercase();
    let mut rows: Vec<UploadTemplate> = uploads
        .into_iter()
        .filter(|u| {
            keyword.is_empty()
                || u.name.to_lowercase().contains(&keyword)
                || u.config.title.clone().unwrap_or_default().to_lowercase().contains(&keyword)
        })
        .collect();
    rows.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    if !sort_asc() {
        rows.reverse();
    }
    let filtered_ids = rows.iter().map(|u| u.id.clone()).collect::<Vec<_>>();
    let rows_view = rows.clone();

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "上传任务" }
                    p { "管理投稿模板并关联账号与 B 站投稿参数。" }
                }
                button { class: "btn btn-primary", onclick: move |_| on_create.call(()), "新建上传任务" }
            }

            div { class: "card",
                p { class: "label", "上传接口核心信息：上传账号、视频标题（支持占位符）、分区、标签、简介。" }
                div { class: "toolbar",
                    input {
                        class: "input",
                        placeholder: "搜索任务名称或投稿标题",
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
                        onclick: move |_| {
                            on_batch_delete.call(selected_ids());
                            selected_ids.set(vec![]);
                        },
                        "批量删除"
                    }
                }
            }

            div { class: "card",
                table { class: "table",
                    thead {
                        tr {
                            th { "选择" }
                            th { "任务名称" }
                            th { "上传账号" }
                            th { "视频标题" }
                            th { "分区" }
                            th { "简介" }
                            th { "标签" }
                            th { class: "actions", "操作" }
                        }
                    }
                    tbody {
                        if rows_view.is_empty() {
                            tr { td { colspan: "8", class: "empty", "暂无上传任务" } }
                        }
                        {
                            rows_view.into_iter().map(|u| {
                                let account_name = accounts
                                    .iter()
                                    .find(|a| a.account_file == u.config.account_file)
                                    .map(|a| a.name.clone())
                                    .unwrap_or_else(|| u.config.account_file.clone());
                                let u_for_edit = u.clone();
                                let u_id = u.id.clone();
                                let u_id_for_check = u_id.clone();
                                let u_id_for_delete = u_id.clone();
                                let checked = selected_ids().contains(&u.id);
                                rsx! {
                                    tr {
                                        td {
                                            input {
                                                r#type: "checkbox",
                                                checked,
                                                onchange: move |_| {
                                                    let mut ids = selected_ids();
                                                    if ids.contains(&u_id_for_check) {
                                                        ids.retain(|v| v != &u_id_for_check);
                                                    } else {
                                                        ids.push(u_id_for_check.clone());
                                                    }
                                                    selected_ids.set(ids);
                                                },
                                            }
                                        }
                                        td { "{u.name}" }
                                        td { "{account_name}" }
                                        td { "{u.config.title.clone().unwrap_or_default()}" }
                                        td { "{tid_name(u.config.tid)}" }
                                        td { "{u.config.description}" }
                                        td {
                                            for t in &u.config.tags {
                                                span { class: "tag", "#{t}" }
                                            }
                                        }
                                        td { class: "actions",
                                            button { class: "btn btn-ghost", onclick: move |_| on_edit.call(u_for_edit.clone()), "编辑" }
                                            button { class: "btn btn-danger", onclick: move |_| on_delete.call(u_id_for_delete.clone()), "删除" }
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
