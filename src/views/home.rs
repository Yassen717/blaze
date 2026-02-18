use dioxus::prelude::*;

use crate::terminal::WebTerminalDemo;
use crate::components::FeatureCard;
use crate::views::Route;

#[component]
pub fn Home() -> Element {
    rsx! {
        section { class: "hero",
            h1 { class: "hero-title",
                "⚡ "
                span { class: "gradient-text", "Blaze" }
                " Terminal"
            }
            p { class: "hero-subtitle",
                "A blazingly fast, modern terminal emulator built with Rust."
            }
            div { class: "hero-buttons",
                Link { to: Route::DemoPage {}, class: "btn-primary", "Try the Demo" }
                Link { to: Route::DownloadPage {}, class: "btn-primary", "⬇  Download" }
                Link { to: Route::CommandsPage {}, class: "btn-secondary", "View Commands →" }
            }
        }

        section { class: "features-section",
            h2 { class: "section-title", "Why Blaze?" }
            div { class: "features-grid",
                FeatureCard { icon: "⚡", title: "Blazingly Fast", desc: "Built in Rust for native speed. Commands execute instantly." }
                FeatureCard { icon: "🎨", title: "Beautiful UI", desc: "Modern interface with color-coded output and smooth scrolling." }
                FeatureCard { icon: "📝", title: "Command History", desc: "Navigate previous commands with the arrow keys." }
                FeatureCard { icon: "🔧", title: "Built-in Commands", desc: "Handy built-ins plus a curated set of system commands." }
                FeatureCard { icon: "🪶", title: "Lightweight", desc: "Tiny binary size, minimal memory footprint." }
                FeatureCard { icon: "🦀", title: "Open Source", desc: "100% free and open source. Written in Rust." }
            }
        }

        section { class: "preview-section",
            h2 { class: "section-title", "See It In Action" }
            p { class: "section-subtitle", "Try a simulated terminal right here in the browser." }
            WebTerminalDemo {}
            div { class: "center-link",
                Link { to: Route::DemoPage {}, class: "btn-secondary", "Full Interactive Demo →" }
            }
        }
    }
}
