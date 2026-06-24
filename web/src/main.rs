#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(app::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!(
        "Web renderer requires wasm32 target. Use `dx serve --platform web` for browser development."
    );
    std::process::exit(1);
}
