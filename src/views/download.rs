use dioxus::prelude::*;

const RELEASE_URL: &str = "https://github.com/Yassen717/blaze/releases/tag/v0.1.1";
const WINDOWS_ASSET_URL: &str = "https://github.com/Yassen717/blaze/releases/download/v0.1.1/blaze_0.1.1_x64_en-US.msi";

#[component]
pub fn DownloadPage() -> Element {
    rsx! {
        section { class: "page-section",
            // ── header ──────────────────────────────────────────────────────────
            h1 { class: "page-title", "Download Blaze" }
            p { class: "page-intro",
                "Get the native desktop terminal for your platform. Each release ships a \
                 ready-to-run installer — no Rust toolchain required."
            }

            // ── primary download card ────────────────────────────────────────────
            div { class: "dl-cards",

                // Windows
                div { class: "dl-card",
                    div { class: "dl-card-header",
                        span { class: "dl-icon", "🪟" }
                        div {
                            h3 { "Windows" }
                            span { class: "dl-badge", "x64 · MSI installer" }
                        }
                    }
                    p { class: "dl-card-desc",
                        "Compatible with Windows 10 / 11 (64-bit). The installer sets up \
                         Blaze and adds it to your Start Menu."
                    }
                    a {
                        class: "btn-primary dl-btn",
                        href: WINDOWS_ASSET_URL,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "⬇  Download v0.1.1 (.msi)"
                    }
                }

                // Linux / macOS — build from source
                div { class: "dl-card dl-card-alt",
                    div { class: "dl-card-header",
                        span { class: "dl-icon", "🐧" }
                        div {
                            h3 { "Linux / macOS" }
                            span { class: "dl-badge dl-badge-muted", "Build from source" }
                        }
                    }
                    p { class: "dl-card-desc",
                        "Pre-built binaries for Linux and macOS are coming soon. In the \
                         meantime you can compile Blaze directly from source in a few steps."
                    }
                    pre { class: "dl-code",
                        "git clone https://github.com/Yassen717/blaze\n"
                        "cd blaze/app\n"
                        "dx serve --platform desktop"
                    }
                }
            }

            // ── release notes link ───────────────────────────────────────────────
            div { class: "dl-release-row",
                a {
                    class: "btn-secondary",
                    href: RELEASE_URL,
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "View Release Notes on GitHub →"
                }
            }

            // ── requirements ────────────────────────────────────────────────────
            h2 { "System Requirements" }
            div { class: "dl-reqs",
                div { class: "dl-req-item",
                    span { class: "dl-req-icon", "💻" }
                    div {
                        strong { "OS" }
                        p { "Windows 10 / 11 (x64)  ·  Linux (coming soon)  ·  macOS (coming soon)" }
                    }
                }
                div { class: "dl-req-item",
                    span { class: "dl-req-icon", "⚙️" }
                    div {
                        strong { "Architecture" }
                        p { "x86-64 (AMD64)" }
                    }
                }
                div { class: "dl-req-item",
                    span { class: "dl-req-icon", "📦" }
                    div {
                        strong { "Runtime" }
                        p { "No extra runtime needed — Blaze ships as a self-contained binary." }
                    }
                }
            }

            // ── version badge ────────────────────────────────────────────────────
            div { class: "dl-version-row",
                span { class: "dl-version-tag", "v0.1.1" }
                span { class: "dl-version-meta", "· Latest release" }
            }
        }
    }
}
