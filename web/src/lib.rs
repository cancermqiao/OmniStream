#![allow(non_snake_case)]

pub mod api;
pub mod app;
pub mod components;
pub mod models;
pub mod styles;

pub use app::App;

#[cfg(target_arch = "wasm32")]
pub async fn sleep_ms(ms: u32) {
    gloo_timers::future::TimeoutFuture::new(ms).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep_ms(ms: u32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}
