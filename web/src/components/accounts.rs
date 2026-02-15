use dioxus::prelude::*;
use shared::UploadAccount;

use crate::{models::QrStartResponse, styles::qr_image_url};

#[component]
pub fn AccountsPage(
    accounts: Vec<UploadAccount>,
    qr_session: Option<QrStartResponse>,
    qr_message: Option<String>,
    on_start_qr: EventHandler<()>,
    on_confirm_qr: EventHandler<()>,
    on_reset_qr: EventHandler<()>,
    on_rename: EventHandler<(String, String)>,
    on_delete: EventHandler<String>,
) -> Element {
    let mut renaming_file = use_signal::<Option<String>>(|| None);
    let mut rename_input = use_signal(String::new);

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "账号管理" }
                    p { "增删改查 Bilibili 上传账号。" }
                }
                button { class: "btn btn-primary", onclick: move |_| on_start_qr.call(()), "扫码登录" }
            }

            if let Some(msg) = qr_message.clone() {
                div { class: "card",
                    p { class: "status", "{msg}" }
                }
            }

            if let Some(session) = qr_session {
                div { class: "card qr-card",
                    div { class: "qr-box",
                        img { src: "{qr_image_url(&session.qr_url)}", alt: "bilibili-qrcode" }
                    }
                    div { class: "qr-info",
                        p { class: "label", "会话 ID" }
                        p { class: "mono", "{session.session_id}" }
                        p { class: "label", "二维码链接" }
                        textarea { readonly: true, class: "mono", value: "{session.qr_url}" }
                        div { class: "inline-actions",
                            button { class: "btn btn-primary", onclick: move |_| on_confirm_qr.call(()), "确认登录" }
                            button { class: "btn btn-ghost", onclick: move |_| on_reset_qr.call(()), "关闭" }
                        }
                    }
                }
            }

            div { class: "card",
                table { class: "table",
                    thead {
                        tr {
                            th { "显示名称" }
                            th { "MID" }
                            th { "Cookie 文件" }
                            th { class: "actions", "操作" }
                        }
                    }
                    tbody {
                        if accounts.is_empty() {
                            tr { td { colspan: "4", class: "empty", "暂无账号" } }
                        }
                        {
                            accounts.into_iter().map(|acc| {
                                let account_file = acc.account_file.clone();
                                let current_name = acc.name.clone();
                                let file_for_delete = account_file.clone();
                                let is_renaming = renaming_file().as_ref() == Some(&account_file);
                                rsx! {
                                    tr {
                                        td {
                                            if is_renaming {
                                                input {
                                                    class: "input",
                                                    value: "{rename_input}",
                                                    oninput: move |e| rename_input.set(e.value()),
                                                }
                                            } else {
                                                "{acc.name}"
                                            }
                                        }
                                        td { "{acc.mid.unwrap_or_default()}" }
                                        td { class: "mono", "{acc.account_file}" }
                                        td { class: "actions",
                                            if is_renaming {
                                                button {
                                                    class: "btn btn-primary",
                                                    onclick: move |_| {
                                                        on_rename.call((account_file.clone(), rename_input()));
                                                        renaming_file.set(None);
                                                    },
                                                    "保存"
                                                }
                                                button { class: "btn btn-ghost", onclick: move |_| renaming_file.set(None), "取消" }
                                            } else {
                                                button {
                                                    class: "btn btn-ghost",
                                                    onclick: move |_| {
                                                        rename_input.set(current_name.clone());
                                                        renaming_file.set(Some(account_file.clone()));
                                                    },
                                                    "重命名"
                                                }
                                                button { class: "btn btn-danger", onclick: move |_| on_delete.call(file_for_delete.clone()), "删除" }
                                            }
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
