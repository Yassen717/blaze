use dioxus::prelude::*;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::commands::desktop::is_allowed_external;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32"), target_os = "windows"))]
use crate::terminal::commands::desktop::execute_windows_command;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32"), not(target_os = "windows")))]
use crate::terminal::commands::desktop::stream_unix_command;
#[cfg(not(feature = "desktop"))]
use crate::terminal::commands::web::run_web_command;
use crate::terminal::state::{LineType, TerminalLine};
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::utils::split_args;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use crate::terminal::utils::push_line_trim;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[component]
pub fn DesktopTerminal() -> Element {
    let mut lines = use_signal(|| {
        vec![
            TerminalLine {
                content: "⚡ Blaze Terminal v0.1.0".into(),
                line_type: LineType::System,
            },
            TerminalLine {
                content: "Type 'help' for available commands.".into(),
                line_type: LineType::System,
            },
            TerminalLine {
                content: String::new(),
                line_type: LineType::System,
            },
        ]
    });
    let mut input_value = use_signal(String::new);
    let mut current_dir = use_signal(|| std::env::current_dir().unwrap_or_default().display().to_string());
    let mut cmd_history = use_signal(Vec::<String>::new);
    let mut history_idx = use_signal(|| -1i32);

    let handle_key = move |e: KeyboardEvent| match e.key() {
        Key::Enter => {
            let cmd = input_value().trim().to_string();
            if cmd.is_empty() {
                return;
            }
            let cwd = current_dir().clone();

            cmd_history.write().push(cmd.clone());
            history_idx.set(-1);

            push_line_trim(
                lines,
                TerminalLine {
                    content: format!("{} > {}", cwd, cmd),
                    line_type: LineType::Command,
                },
            );
            input_value.set(String::new());

            let args = split_args(&cmd);
            let first = args.first().map(|s| s.to_lowercase()).unwrap_or_default();

            match first.as_str() {
                "clear" | "cls" => {
                    lines.write().clear();
                    return;
                }
                "help" => {
                    let help = [
                        "⚡ Blaze Terminal — Commands:",
                        "",
                        "  help            Show this help message",
                        "  clear / cls     Clear terminal output",
                        "  cd <dir>        Change directory",
                        "  pwd             Print working directory",
                        "  exit            Exit the terminal",
                        "",
                        #[cfg(all(target_os = "windows", feature = "safe-mode"))]
                        "Allowed system commands (safe mode): ls, dir, echo, vim, whoami, cat/type, grep, curl, wget, ipconfig (ip).",
                        #[cfg(all(target_os = "windows", not(feature = "safe-mode"), not(feature = "unsafe-fs")))]
                        "Allowed system commands: ls, dir, echo, vim, whoami, cat/type, grep, curl, wget, ipconfig (ip).",
                        #[cfg(all(target_os = "windows", not(feature = "safe-mode"), feature = "unsafe-fs"))]
                        "Allowed system commands: ls, dir, echo, vim, mkdir, rm/del, mv, whoami, cat/type, grep, curl, wget, ipconfig (ip).",
                        #[cfg(all(not(target_os = "windows"), feature = "safe-mode"))]
                        "Allowed system commands (safe mode): ls, dir, echo, vim, whoami, cat, grep, curl, wget, ifconfig, ip.",
                        #[cfg(all(not(target_os = "windows"), not(feature = "safe-mode"), not(feature = "unsafe-fs")))]
                        "Allowed system commands: ls, dir, echo, vim, whoami, cat, grep, curl, wget, ifconfig, ip.",
                        #[cfg(all(not(target_os = "windows"), not(feature = "safe-mode"), feature = "unsafe-fs"))]
                        "Allowed system commands: ls, dir, echo, vim, mkdir, rm/del, mv, whoami, cat, grep, curl, wget, ifconfig, ip.",
                    ];
                    let mut v = lines.write();
                    for h in help {
                        v.push(TerminalLine {
                            content: h.to_string(),
                            line_type: LineType::System,
                        });
                    }
                    return;
                }
                "exit" => {
                    dioxus::desktop::window().close();
                    return;
                }
                "cd" => {
                    let rest = args.iter().skip(1).cloned().collect::<Vec<_>>().join(" ");
                    if rest.is_empty() {
                        push_line_trim(
                            lines,
                            TerminalLine {
                                content: cwd.clone(),
                                line_type: LineType::Output,
                            },
                        );
                        return;
                    }
                    let target = if std::path::Path::new(&rest).is_absolute() {
                        std::path::PathBuf::from(&rest)
                    } else {
                        std::path::PathBuf::from(&cwd).join(&rest)
                    };
                    match target.canonicalize() {
                        Ok(p) if p.is_dir() => {
                            let s = p.display().to_string();
                            let clean = s.strip_prefix(r"\\?\").unwrap_or(&s).to_string();
                            current_dir.set(clean);
                        }
                        Ok(_) => {
                            push_line_trim(
                                lines,
                                TerminalLine {
                                    content: format!("Not a directory: {}", rest),
                                    line_type: LineType::Error,
                                },
                            );
                        }
                        Err(e) => {
                            push_line_trim(
                                lines,
                                TerminalLine {
                                    content: format!("cd: {}: {}", rest, e),
                                    line_type: LineType::Error,
                                },
                            );
                        }
                    }
                    return;
                }
                "pwd" => {
                    push_line_trim(
                        lines,
                        TerminalLine {
                            content: cwd.clone(),
                            line_type: LineType::Output,
                        },
                    );
                    return;
                }
                _ => {}
            }

            if !is_allowed_external(&first) {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: format!(
                            "Command '{}' is not allowed. Type 'help' for a list of available commands.",
                            first
                        ),
                        line_type: LineType::Error,
                    },
                );
                return;
            }

            #[cfg(target_os = "windows")]
            {
                let lines_sig = lines;
                let program = first.clone();
                let argv = args;
                spawn(async move {
                    let result = tokio::task::spawn_blocking(move || -> Vec<TerminalLine> {
                        execute_windows_command(&cwd, &program, &argv)
                    })
                    .await;

                    match result {
                        Ok(lines_out) => {
                            for line in lines_out {
                                push_line_trim(lines_sig, line);
                            }
                        }
                        Err(e) => {
                            push_line_trim(
                                lines_sig,
                                TerminalLine {
                                    content: format!("Error: {}", e),
                                    line_type: LineType::Error,
                                },
                            );
                        }
                    }
                });
            }

            #[cfg(not(target_os = "windows"))]
            {
                let lines = lines;
                let program = first.clone();
                let program_args = args.iter().skip(1).cloned().collect::<Vec<_>>();

                spawn(async move {
                    stream_unix_command(cwd, program, program_args, lines).await;
                });
            }
        }
        Key::ArrowUp => {
            let history = cmd_history();
            if history.is_empty() {
                return;
            }
            let idx = history_idx();
            let new_idx = if idx < 0 {
                history.len() as i32 - 1
            } else {
                (idx - 1).max(0)
            };
            history_idx.set(new_idx);
            input_value.set(history[new_idx as usize].clone());
        }
        Key::ArrowDown => {
            let history = cmd_history();
            let idx = history_idx();
            if idx < 0 {
                return;
            }
            let new_idx = idx + 1;
            if new_idx >= history.len() as i32 {
                history_idx.set(-1);
                input_value.set(String::new());
            } else {
                history_idx.set(new_idx);
                input_value.set(history[new_idx as usize].clone());
            }
        }
        _ => {}
    };

    use_effect(move || {
        let _ = lines();
        document::eval(
            r#"setTimeout(()=>{let e=document.getElementById('terminal-output');if(e)e.scrollTop=e.scrollHeight},10)"#,
        );
    });

    rsx! {
        div { class: "terminal-container terminal-fullscreen",
            div { class: "terminal-header",
                span { class: "terminal-title", "⚡ Blaze Terminal" }
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
                onclick: move |_| {
                    document::eval(r#"document.getElementById('terminal-input').focus()"#);
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
                    span { class: "prompt", "{current_dir()} > " }
                    input {
                        id: "terminal-input",
                        class: "terminal-input",
                        r#type: "text",
                        value: "{input_value}",
                        autofocus: true,
                        oninput: move |e| input_value.set(e.value()),
                        onkeydown: handle_key,
                    }
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
                content: "⚡ Blaze Terminal v0.1.0 (Web Demo)".into(),
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
