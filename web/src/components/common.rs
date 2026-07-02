use dioxus::prelude::*;

#[component]
pub fn TabItem(
    active: bool,
    label: String,
    icon: String,
    compact: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let class = if active { "tab-item tab-item-active" } else { "tab-item" };

    rsx! {
        button { class: "{class}", onclick, title: "{label}",
            span { class: "tab-icon", "{icon}" }
            if !compact {
                span { class: "tab-label", "{label}" }
            }
        }
    }
}
