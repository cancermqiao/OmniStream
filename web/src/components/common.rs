use dioxus::prelude::*;

#[component]
pub fn TabItem(active: bool, label: String, onclick: EventHandler<MouseEvent>) -> Element {
    let class = if active { "tab-item tab-item-active" } else { "tab-item" };

    rsx! {
        button { class: "{class}", onclick, "{label}" }
    }
}
