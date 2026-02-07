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
