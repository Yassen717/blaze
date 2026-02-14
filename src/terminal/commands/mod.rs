#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub mod desktop;
#[cfg(not(feature = "desktop"))]
pub mod web;
