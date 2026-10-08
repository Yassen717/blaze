use dioxus::prelude::*;

#[cfg(not(feature = "desktop"))]
use crate::terminal::commands::web::run_web_command;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::pty::keys::key_to_bytes;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::pty::PtySession;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::screen::{display_title, ScreenSnapshot};
#[cfg(not(feature = "desktop"))]
use crate::terminal::state::{LineType, TerminalLine};
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::PtyScreen;

/// Measures the character cell via `#pty-measure` and reports the
/// `[cols, rows]` that fit inside `#terminal-output` whenever it resizes.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const FIT_SCRIPT: &str = r#"
const host = document.getElementById('terminal-output');
const probe = document.getElementById('pty-measure');
if (host && probe) {
    let last = '', timer = null;
    const fit = () => {
        const cell = probe.getBoundingClientRect();
        const cs = getComputedStyle(host);
        const w = host.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
        const h = host.clientHeight - parseFloat(cs.paddingTop) - parseFloat(cs.paddingBottom);
        if (cell.width <= 0 || cell.height <= 0) return;
        const size = [Math.floor(w / (cell.width / 10)), Math.floor(h / cell.height)];
        const key = size.join('x');
        if (key !== last) { last = key; dioxus.send(size); }
    };
    new ResizeObserver(() => { clearTimeout(timer); timer = setTimeout(fit, 60); }).observe(host);
    document.fonts.ready.then(fit);
    fit();
    host.focus();
}
await new Promise(() => {});
"#;

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
                // Same-size resize on purpose: portable-pty enables
                // PSEUDOCONSOLE_RESIZE_QUIRK, under which older inbox ConPTY
                // withholds the first frame until a resize arrives.
                let _ = pty_session.resize_tx.send((80, 24));
                let resize_tx = pty_session.resize_tx.clone();
                session.set(Some(pty_session));

                // Keep the PTY grid matched to the visible area of the window.
                spawn(async move {
                    let mut fit = document::eval(FIT_SCRIPT);
                    while let Ok((cols, rows)) = fit.recv::<(u16, u16)>().await {
                        if resize_tx.send((cols.max(20), rows.max(5))).is_err() {
                            break;
                        }
                    }
                });

                let mut screen_sig = screen;
                let mut title_sig = terminal_title;
                spawn(async move {
                    while let Some(mut update) = update_rx.recv().await {
                        // Render only the newest screen if several queued up.
                        while let Ok(next) = update_rx.try_recv() {
                            update = next;
                        }
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

    let title = display_title(&terminal_title());

    rsx! {
        div { class: "terminal-container terminal-fullscreen",
            div {
                class: "terminal-header",
                ondoubleclick: move |_| dioxus::desktop::window().toggle_maximized(),
                div { class: "terminal-brand",
                    span { class: "brand-mark", "⚡" }
                    span { class: "terminal-title", title: "{title}", "{title}" }
                }
                div {
                    class: "terminal-controls",
                    // Keep keyboard focus on the terminal when clicking buttons.
                    onmousedown: move |e| e.prevent_default(),
                    button {
                        class: "win-btn win-btn-minimize",
                        title: "Minimize",
                        onclick: move |_| dioxus::desktop::window().set_minimized(true),
                        svg { width: "10", height: "10", view_box: "0 0 10 10",
                            path { d: "M0 5.5h10", stroke: "currentColor", stroke_width: "1" }
                        }
                    }
                    button {
                        class: "win-btn win-btn-maximize",
                        title: "Maximize",
                        onclick: move |_| dioxus::desktop::window().toggle_maximized(),
                        svg { width: "10", height: "10", view_box: "0 0 10 10",
                            rect { x: "0.5", y: "0.5", width: "9", height: "9", fill: "none", stroke: "currentColor", stroke_width: "1" }
                        }
                    }
                    button {
                        class: "win-btn win-btn-close",
                        title: "Close",
                        onclick: move |_| dioxus::desktop::window().close(),
                        svg { width: "10", height: "10", view_box: "0 0 10 10",
                            path { d: "M0.5 0.5l9 9M9.5 0.5l-9 9", stroke: "currentColor", stroke_width: "1.1" }
                        }
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

                // Hidden probe used by FIT_SCRIPT to measure one character cell.
                span { id: "pty-measure", class: "pty-measure", "WWWWWWWWWW" }

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
