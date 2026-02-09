use dioxus::prelude::*;

use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::state::{LineType, TerminalLine};

#[cfg(feature = "desktop")]
const MAX_LINES: usize = 5000;

#[cfg(feature = "desktop")]
const ALLOWED_EXTERNAL: [&str; 4] = ["ls", "dir", "echo", "vim"];

#[cfg(feature = "desktop")]
#[component]
pub fn DesktopTerminal() -> Element {
    let mut lines = use_signal(|| {
        vec![
            TerminalLine { content: "⚡ Blaze Terminal v0.1.0".into(), line_type: LineType::System },
            TerminalLine { content: "Type 'help' for available commands.".into(), line_type: LineType::System },
            TerminalLine { content: String::new(), line_type: LineType::System },
        ]
    });
    let mut input_value = use_signal(String::new);
    let mut current_dir = use_signal(|| {
        std::env::current_dir()
            .unwrap_or_default()
            .display()
            .to_string()
    });
    let mut cmd_history = use_signal(Vec::<String>::new);
    let mut history_idx = use_signal(|| -1i32);

    let handle_key = move |e: KeyboardEvent| {
        match e.key() {
            Key::Enter => {
                let cmd = input_value().trim().to_string();
                if cmd.is_empty() {
                    return;
                }
                let cwd = current_dir().clone();

                cmd_history.write().push(cmd.clone());
                history_idx.set(-1);

                lines.write().push(TerminalLine {
                    content: format!("{} > {}", cwd, cmd),
                    line_type: LineType::Command,
                });
                input_value.set(String::new());

                let first = cmd.split_whitespace().next().unwrap_or("").to_lowercase();

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
                            "  exit            Exit the terminal",
                            "",
                            "Allowed system commands: ls, dir, echo, vim.",
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
                        std::process::exit(0);
                    }
                    "cd" => {
                        let rest: String =
                            cmd.split_whitespace().skip(1).collect::<Vec<&str>>().join(" ");
                        if rest.is_empty() {
                            lines.write().push(TerminalLine {
                                content: cwd.clone(),
                                line_type: LineType::Output,
                            });
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
                                lines.write().push(TerminalLine {
                                    content: format!("Not a directory: {}", rest),
                                    line_type: LineType::Error,
                                });
                            }
                            Err(e) => {
                                lines.write().push(TerminalLine {
                                    content: format!("cd: {}: {}", rest, e),
                                    line_type: LineType::Error,
                                });
                            }
                        }
                        return;
                    }
                    _ => {}
                }

                if !ALLOWED_EXTERNAL.contains(&first.as_str()) {
                    lines.write().push(TerminalLine {
                        content: format!(
                            "Command '{}' is not allowed. Type 'help' for a list of available commands.",
                            first
                        ),
                        line_type: LineType::Error,
                    });
                    return;
                }

                spawn(async move {
                    let mut lines = lines;
                    #[cfg(target_os = "windows")]
                    let child = Command::new("cmd")
                        .args(["/C", &cmd])
                        .current_dir(&cwd)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .creation_flags(0x08000000)
                        .spawn();

                    #[cfg(not(target_os = "windows"))]
                    let child = Command::new("sh")
                        .args(["-c", &cmd])
                        .current_dir(&cwd)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn();

                    let push_line = |lines: &mut Signal<Vec<TerminalLine>>, line: TerminalLine| {
                        let mut v = lines.write();
                        v.push(line);
                        if v.len() > MAX_LINES {
                            let excess = v.len() - MAX_LINES;
                            v.drain(0..excess);
                        }
                    };

                    match child {
                        Ok(mut child) => {
                            let stdout = child.stdout.take();
                            let stderr = child.stderr.take();

                            let (tx, mut rx) = mpsc::unbounded_channel::<TerminalLine>();

                            if let Some(out) = stdout {
                                let tx_out = tx.clone();
                                spawn(async move {
                                    let mut reader = BufReader::new(out).lines();
                                    while let Ok(Some(line)) = reader.next_line().await {
                                        let _ = tx_out.send(TerminalLine {
                                            content: line,
                                            line_type: LineType::Output,
                                        });
                                    }
                                });
                            }

                            if let Some(err) = stderr {
                                let tx_err = tx.clone();
                                spawn(async move {
                                    let mut reader = BufReader::new(err).lines();
                                    while let Ok(Some(line)) = reader.next_line().await {
                                        let _ = tx_err.send(TerminalLine {
                                            content: line,
                                            line_type: LineType::Error,
                                        });
                                    }
                                });
                            }

                            drop(tx);

                            while let Some(line) = rx.recv().await {
                                push_line(&mut lines, line);
                            }

                            let _ = child.wait().await;
                        }
                        Err(e) => {
                            push_line(
                                &mut lines,
                                TerminalLine {
                                    content: format!("Error: {}", e),
                                    line_type: LineType::Error,
                                },
                            );
                        }
                    }
                });
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
        }
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
    let mut lines = use_signal(|| {
        vec![
            TerminalLine { content: "⚡ Blaze Terminal v0.1.0 (Web Demo)".into(), line_type: LineType::System },
            TerminalLine { content: "Type 'help' to see commands.".into(), line_type: LineType::System },
            TerminalLine { content: String::new(), line_type: LineType::System },
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

        lines.write().push(TerminalLine {
            content: format!("{} > {}", demo_dir, cmd),
            line_type: LineType::Command,
        });
        input_value.set(String::new());

        let cmd_lower = cmd.to_lowercase();
        let first = cmd_lower.split_whitespace().next().unwrap_or("");

        match first {
            "clear" | "cls" => {
                lines.write().clear();
                return;
            }
            "help" => {
                for line in [
                    "⚡ Blaze Terminal — Demo Commands:",
                    "",
                    "  help          Show this message",
                    "  clear / cls   Clear the screen",
                    "  dir / ls      List files and folders",
                    "  echo <text>   Print text to the terminal",
                    "  whoami        Show current user",
                    "  date          Show the date",
                    "  ipconfig      Show network configuration",
                    "  ping <host>   Test network connectivity",
                ] {
                    lines.write().push(TerminalLine { content: line.into(), line_type: LineType::System });
                }
            }
            "dir" | "ls" => {
                for line in [
                    " Directory of C:\\Users\\You",
                    "",
                    " 02/07/2026  10:00 AM    <DIR>          Documents",
                    " 02/07/2026  10:00 AM    <DIR>          Projects",
                    " 02/07/2026  10:00 AM    <DIR>          Downloads",
                    " 01/15/2026  03:45 PM           2,048   notes.txt",
                    "               1 File(s)         2,048 bytes",
                ] {
                    lines.write().push(TerminalLine { content: line.into(), line_type: LineType::Output });
                }
            }
            "echo" => {
                let text = if cmd.len() > 5 { cmd[5..].to_string() } else { String::new() };
                lines.write().push(TerminalLine { content: text, line_type: LineType::Output });
            }
            "whoami" => {
                lines.write().push(TerminalLine { content: "You".into(), line_type: LineType::Output });
            }
            "date" => {
                lines.write().push(TerminalLine { content: "Fri 02/07/2026".into(), line_type: LineType::Output });
            }
            "ipconfig" => {
                for line in [
                    "Windows IP Configuration",
                    "",
                    "Ethernet adapter Ethernet:",
                    "   IPv4 Address. . . . . : 192.168.1.100",
                    "   Subnet Mask . . . . . : 255.255.255.0",
                    "   Default Gateway . . . : 192.168.1.1",
                ] {
                    lines.write().push(TerminalLine { content: line.into(), line_type: LineType::Output });
                }
            }
            "ping" => {
                if cmd_lower.split_whitespace().count() < 2 {
                    lines.write().push(TerminalLine { content: "Usage: ping <host>".into(), line_type: LineType::Error });
                } else {
                    for line in [
                        "Pinging host with 32 bytes of data:",
                        "Reply from 93.184.216.34: bytes=32 time=12ms TTL=56",
                        "Reply from 93.184.216.34: bytes=32 time=11ms TTL=56",
                        "Reply from 93.184.216.34: bytes=32 time=13ms TTL=56",
                    ] {
                        lines.write().push(TerminalLine { content: line.into(), line_type: LineType::Output });
                    }
                }
            }
            "cd" => {
                lines.write().push(TerminalLine {
                    content: "Directory changed (simulated)".into(),
                    line_type: LineType::System,
                });
            }
            "exit" => {
                lines.write().push(TerminalLine {
                    content: "Can't exit the web demo! Download the real thing.".into(),
                    line_type: LineType::System,
                });
            }
            _ => {
                lines.write().push(TerminalLine {
                    content: format!("'{}': command not recognized. Type 'help' for commands.", cmd),
                    line_type: LineType::Error,
                });
            }
        }
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
