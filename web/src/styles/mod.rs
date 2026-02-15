mod forms;
mod layout;
mod modal;
mod pages;
mod table;

pub fn qr_image_url(raw: &str) -> String {
    format!(
        "https://api.qrserver.com/v1/create-qr-code/?size=260x260&data={}",
        urlencoding::encode(raw)
    )
}

pub fn theme_css() -> String {
    [layout::CSS, pages::CSS, table::CSS, forms::CSS, modal::CSS].join("\n")
}
