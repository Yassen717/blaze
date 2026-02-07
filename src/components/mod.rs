pub mod terminal;

#[cfg(not(feature = "desktop"))]
use dioxus::prelude::*;

#[cfg(not(feature = "desktop"))]
#[component]
pub fn FeatureCard(icon: &'static str, title: &'static str, desc: &'static str) -> Element {
    rsx! {
        div { class: "feature-card",
            div { class: "feature-icon", "{icon}" }
            h3 { "{title}" }
            p { "{desc}" }
        }
    }
}

#[cfg(not(feature = "desktop"))]
#[component]
pub fn CmdCard(cmd: &'static str, desc: &'static str, example: &'static str) -> Element {
    rsx! {
        div { class: "command-card",
            code { class: "cmd-name", "{cmd}" }
            p { "{desc}" }
            pre { class: "cmd-example", "> {example}" }
        }
    }
}
