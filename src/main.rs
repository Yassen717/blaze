#![allow(non_snake_case)]

use dioxus::prelude::*;

mod components;
mod state;
mod views;

#[cfg(feature = "desktop")]
use components::terminal::DesktopTerminal;

#[cfg(not(feature = "desktop"))]
use views::Route;

// ======================== Assets ========================

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// ======================== Main ========================

#[cfg(feature = "desktop")]
fn main() {
    use dioxus::desktop::{Config, WindowBuilder, LogicalSize};

    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("Blaze Terminal")
                        .with_decorations(false)
                        .with_inner_size(LogicalSize::new(1100.0, 700.0))
                        .with_min_inner_size(LogicalSize::new(600.0, 400.0))
                )
                .with_background_color((5, 6, 7, 255))
                .with_disable_context_menu(true)
        )
        .launch(App);
}

#[cfg(not(feature = "desktop"))]
fn main() {
    dioxus::launch(App);
}

// ======================== Root App ========================

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        AppInner {}
    }
}

/// Desktop: render the real terminal
#[cfg(feature = "desktop")]
#[component]
fn AppInner() -> Element {
    rsx! {
        document::Title { "⚡ Blaze Terminal" }
        DesktopTerminal {}
    }
}

/// Web: render the showcase website
#[cfg(not(feature = "desktop"))]
#[component]
fn AppInner() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
