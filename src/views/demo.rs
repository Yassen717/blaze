use dioxus::prelude::*;

use crate::components::terminal::WebTerminalDemo;

#[component]
pub fn DemoPage() -> Element {
    rsx! {
        section { class: "page-section",
            h1 { class: "page-title", "Interactive Demo" }
            p { class: "page-intro",
                "Try Blaze right in your browser! Type commands below."
            }
            WebTerminalDemo {}
            p { class: "demo-note",
                "This is a simulated demo. Download Blaze for real command execution."
            }
        }
    }
}
