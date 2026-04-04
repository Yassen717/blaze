pub mod components;

#[cfg(not(feature = "desktop"))]
pub mod commands;
#[cfg(not(feature = "desktop"))]
pub mod state;
#[cfg(not(feature = "desktop"))]
pub mod utils;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub mod screen;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub mod pty;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub use components::DesktopTerminal;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub use pty::renderer::PtyScreen;
#[cfg(not(feature = "desktop"))]
pub use components::WebTerminalDemo;
