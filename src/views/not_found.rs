use dioxus::prelude::*;

use crate::views::Route;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    };

    rsx! {
        section { class: "page-section",
            h1 { class: "page-title", "404" }
            p { class: "page-intro",
                "This page doesn’t exist: "
                code { "{path}" }
            }

            div { class: "hero-buttons",
                Link { to: Route::Home {}, class: "btn-primary", "Go Home" }
                Link { to: Route::DemoPage {}, class: "btn-secondary", "Try the Demo →" }
            }
        }
    }
}
