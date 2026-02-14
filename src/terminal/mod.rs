pub mod commands;
pub mod components;
pub mod state;
pub mod utils;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub use components::DesktopTerminal;
#[cfg(not(feature = "desktop"))]
pub use components::WebTerminalDemo;
