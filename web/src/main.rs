#[cfg(any(target_arch = "wasm32", feature = "desktop", feature = "mobile"))]
fn main() {
    #[cfg(any(feature = "desktop", feature = "mobile"))]
    {
        let api_url = app::app::api_url().trim_end_matches('/');
        let server_url = api_url.strip_suffix("/api").unwrap_or(api_url);
        dioxus::fullstack::set_server_url(server_url);
    }
    dioxus::launch(app::App);
}

#[cfg(not(any(target_arch = "wasm32", feature = "desktop", feature = "mobile")))]
fn main() {
    eprintln!(
        "Web renderer requires wasm32 target. Use `dx serve --platform web` for browser development."
    );
    std::process::exit(1);
}
