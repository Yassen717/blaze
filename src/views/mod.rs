#[cfg(not(feature = "desktop"))]
use dioxus::prelude::*;

#[cfg(not(feature = "desktop"))]
pub mod home;
#[cfg(not(feature = "desktop"))]
pub mod commands;
#[cfg(not(feature = "desktop"))]
pub mod demo;
#[cfg(not(feature = "desktop"))]
pub mod not_found;

#[cfg(not(feature = "desktop"))]
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(WebLayout)]
        #[route("/")]
        Home {},
        #[route("/commands")]
        CommandsPage {},
        #[route("/demo")]
        DemoPage {},

    // Catch-all route for 404s (must be last)
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[cfg(not(feature = "desktop"))]
#[component]
pub fn WebLayout() -> Element {
    rsx! {
        document::Title { "Blaze Terminal" }
        nav { class: "web-nav",
            div { class: "nav-inner",
                Link { to: Route::Home {}, class: "nav-logo", "⚡ Blaze" }
                div { class: "nav-links",
                    Link { to: Route::Home {}, "Home" }
                    Link { to: Route::CommandsPage {}, "Commands" }
                    Link { to: Route::DemoPage {}, "Demo" }
                }
            }
        }
        main { class: "web-main",
            Outlet::<Route> {}
        }
        footer { class: "web-footer",
            p { "⚡ Blaze Terminal — Built with Rust & Dioxus" }
        }
    }
}

#[cfg(not(feature = "desktop"))]
pub use commands::CommandsPage;
#[cfg(not(feature = "desktop"))]
pub use demo::DemoPage;
#[cfg(not(feature = "desktop"))]
pub use home::Home;
#[cfg(not(feature = "desktop"))]
pub use not_found::NotFound;
