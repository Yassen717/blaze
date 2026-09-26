use dioxus::prelude::*;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::pty::keys::key_to_bytes;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::pty::PtySession;
#[cfg(not(feature = "desktop"))]
use crate::terminal::commands::web::run_web_command;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::screen::ScreenSnapshot;
#[cfg(not(feature = "desktop"))]
use crate::terminal::state::{LineType, TerminalLine};
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::PtyScreen;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[component]
pub fn DesktopTerminal() -> Element {
    let screen = use_signal(ScreenSnapshot::default);
    let terminal_title = use_signal(|| "⚡ Blaze Terminal".to_string());
    let mut startup_error = use_signal(|| Option::<String>::None);
    let mut session = use_signal(|| Option::<PtySession>::None);
    let mut is_started = use_signal(|| false);

    // Start PTY session exactly once.
    use_effect(move || {
        if is_started() {
            return;
        }
        is_started.set(true);

        match PtySession::spawn(80, 24) {
            Ok((pty_session, mut update_rx)) => {
                let _ = pty_session.resize_tx.send((80, 24));
                session.set(Some(pty_session));

                let mut screen_sig = screen;
                let mut title_sig = terminal_title;
                spawn(async move {
                    while let Some(update) = update_rx.recv().await {
                        *screen_sig.write() = update.snapshot;
                        if !update.title.is_empty() {
                            *title_sig.write() = update.title;
                        }
                    }
                });
            }
            Err(e) => {
                startup_error.set(Some(format!("Failed to start PTY session: {e}")));
            }
        }
    });

    let handle_key = move |e: KeyboardEvent| {
        // Keep browser/webview behavior terminal-like.
        e.prevent_default();

        if let Some(bytes) = key_to_bytes(e.key(), e.modifiers()) {
            if let Some(s) = session.read().as_ref() {
                let _ = s.write_tx.send(bytes);
            }
        }
    };

    let screen_signal: ReadSignal<ScreenSnapshot> = screen.into();

    rsx! {
        div { class: "terminal-container terminal-fullscreen",
            div { class: "terminal-header",
                span { class: "terminal-title", "{terminal_title()}" }
                div { class: "terminal-controls",
                    button {
                        class: "win-btn win-btn-minimize",
                        onclick: move |_| {
                            dioxus::desktop::window().set_minimized(true);
                        },
                        "−"
                    }
                    button {
                        class: "win-btn win-btn-maximize",
                        onclick: move |_| {
                            dioxus::desktop::window().toggle_maximized();
                        },
                        "□"
                    }
                    button {
                        class: "win-btn win-btn-close",
                        onclick: move |_| {
                            dioxus::desktop::window().close();
                        },
                        "✕"
                    }
                }
            }

            div {
                id: "terminal-output",
                class: "terminal-body",
                tabindex: 0,
                autofocus: true,
                onkeydown: handle_key,
                onclick: move |_| {
                    document::eval(r#"document.getElementById('terminal-output')?.focus()"#);
                },

                if let Some(err) = startup_error() {
                    div { class: "line-error", "{err}" }
                } else {
                    PtyScreen { snapshot: screen_signal }
                }
            }
        }
    }
}

#[cfg(not(feature = "desktop"))]
#[component]
pub fn WebTerminalDemo() -> Element {
    let lines = use_signal(|| {
        vec![
            TerminalLine {
                content: "⚡ Blaze Terminal v0.2.1 (Web Demo)".into(),
                line_type: LineType::System,
            },
            TerminalLine {
                content: "Type 'help' to see commands.".into(),
                line_type: LineType::System,
            },
            TerminalLine {
                content: String::new(),
                line_type: LineType::System,
            },
        ]
    });
    let mut input_value = use_signal(String::new);
    let demo_dir = "C:\\Users\\You";

    let handle_key = move |e: KeyboardEvent| {
        if e.key() != Key::Enter {
            return;
        }
        let cmd = input_value().trim().to_string();
        if cmd.is_empty() {
            return;
        }

        run_web_command(&cmd, demo_dir, lines);
        input_value.set(String::new());
    };

    use_effect(move || {
        let _ = lines();
        document::eval(
            r#"setTimeout(()=>{let e=document.getElementById('demo-output');if(e)e.scrollTop=e.scrollHeight},10)"#,
        );
    });

    rsx! {
        div { class: "terminal-container demo-terminal",
            div { class: "terminal-header",
                div { class: "terminal-dots",
                    span { class: "dot dot-red" }
                    span { class: "dot dot-yellow" }
                    span { class: "dot dot-green" }
                }
                span { class: "terminal-title", "⚡ Blaze Terminal (Demo)" }
            }
            div {
                id: "demo-output",
                class: "terminal-body",
                onclick: move |_| {
                    document::eval(r#"document.getElementById('demo-input').focus()"#);
                },
                for (i, line) in lines().iter().enumerate() {
                    div {
                        key: "{i}",
                        class: match line.line_type {
                            LineType::Command => "line-command",
                            LineType::Output  => "line-output",
                            LineType::Error   => "line-error",
                            LineType::System  => "line-system",
                        },
                        "{line.content}"
                    }
                }
                div { class: "terminal-input-line",
                    span { class: "prompt", "{demo_dir} > " }
                    input {
                        id: "demo-input",
                        class: "terminal-input",
                        r#type: "text",
                        value: "{input_value}",
                        oninput: move |e| input_value.set(e.value()),
                        onkeydown: handle_key,
                    }
                }
            }
        }
    }
}
