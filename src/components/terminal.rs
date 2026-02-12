use dioxus::prelude::*;

#[cfg(all(feature = "desktop", not(target_os = "windows")))]
use std::process::Stdio;

#[cfg(all(feature = "desktop", not(target_os = "windows")))]
use tokio::io::{AsyncBufReadExt, BufReader};
#[cfg(all(feature = "desktop", not(target_os = "windows")))]
use tokio::process::Command;
#[cfg(all(feature = "desktop", not(target_os = "windows")))]
use tokio::sync::mpsc;

use crate::state::{LineType, TerminalLine};

const MAX_LINES: usize = 5000;

#[cfg(all(feature = "desktop", target_os = "windows"))]
const MAX_CMD_OUTPUT_BYTES: usize = 1024 * 1024; // 1 MiB

#[cfg(all(
    feature = "desktop",
    not(target_arch = "wasm32"),
    target_os = "windows"
))]
const ALLOWED_EXTERNAL: [&str; 16] = [
    "ls",
    "dir",
    "echo",
    "vim",
    "mkdir",
    "rm",
    "del",
    "mv",
    "whoami",
    "cat",
    "type",
    "grep",
    "curl",
    "wget",
    "ipconfig",
    "ip",
];

#[cfg(all(
    feature = "desktop",
    not(target_arch = "wasm32"),
    not(target_os = "windows")
))]
const ALLOWED_EXTERNAL: [&str; 15] = [
    "ls",
    "dir",
    "echo",
    "vim",
    "mkdir",
    "rm",
    "del",
    "mv",
    "whoami",
    "cat",
    "grep",
    "curl",
    "wget",
    "ifconfig",
    "ip",
];

fn push_line_trim(mut lines: Signal<Vec<TerminalLine>>, line: TerminalLine) {
    let mut v = lines.write();
    v.push(line);
    if v.len() > MAX_LINES {
        let excess = v.len() - MAX_LINES;
        v.drain(0..excess);
    }
}

/// Split a command string into args.
///
/// This is intentionally not shell parsing: it supports quotes for spaces, but treats `&`, `|`, `;` etc as normal characters.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '\\' => {
                if let Some('"') = chars.peek().copied() {
                    chars.next();
                    current.push('"');
                } else {
                    current.push('\\');
                }
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
                while let Some(c2) = chars.peek().copied() {
                    if c2.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

#[cfg(all(feature = "desktop", target_os = "windows"))]
fn resolve_in_dir(cwd: &str, target: &str) -> std::path::PathBuf {
    let target_path = std::path::Path::new(target);
    if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        std::path::Path::new(cwd).join(target_path)
    }
}

#[cfg(all(feature = "desktop", target_os = "windows"))]
fn run_external_command_lines(cwd: &str, program: &str, args: &[String]) -> Vec<TerminalLine> {
    use std::process::Command;

    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            return vec![TerminalLine {
                content: format!("{}: {}", program, e),
                line_type: LineType::Error,
            }]
        }
    };

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&output.stdout);
    bytes.extend_from_slice(&output.stderr);

    if bytes.len() > MAX_CMD_OUTPUT_BYTES {
        bytes.truncate(MAX_CMD_OUTPUT_BYTES);
        bytes.extend_from_slice(b"\n...(output truncated)\n");
    }

    let text = String::from_utf8_lossy(&bytes);
    let line_type = if output.status.success() {
        LineType::Output
    } else {
        LineType::Error
    };

    let mut out = Vec::new();
    for line in text.lines() {
        out.push(TerminalLine {
            content: line.to_string(),
            line_type: line_type.clone(),
        });
    }

    if out.is_empty() {
        out.push(TerminalLine {
            content: String::new(),
            line_type,
        });
    }

    out
}

#[cfg(all(feature = "desktop", target_os = "windows"))]
fn list_dir_lines(path: &std::path::Path) -> Vec<TerminalLine> {
    let mut out = Vec::new();
    match std::fs::read_dir(path) {
        Ok(entries) => {
            out.push(TerminalLine {
                content: format!(" Directory of {}", path.display()),
                line_type: LineType::Output,
            });
            out.push(TerminalLine {
                content: String::new(),
                line_type: LineType::Output,
            });

            let mut names: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            names.sort();

            for name in names {
                out.push(TerminalLine {
                    content: name,
                    line_type: LineType::Output,
                });
            }
        }
        Err(e) => out.push(TerminalLine {
            content: format!("dir: {}", e),
            line_type: LineType::Error,
        }),
    }
    out
}

#[cfg(all(feature = "desktop", target_os = "windows"))]
fn read_file_lines(path: &std::path::Path) -> Vec<TerminalLine> {
    const MAX_BYTES: usize = 512 * 1024;
    match std::fs::read(path) {
        Ok(bytes) => {
            let bytes = if bytes.len() > MAX_BYTES { &bytes[..MAX_BYTES] } else { &bytes };
            let text = String::from_utf8_lossy(bytes);
            text.lines()
                .map(|l| TerminalLine {
                    content: l.to_string(),
                    line_type: LineType::Output,
                })
                .collect()
        }
        Err(e) => vec![TerminalLine {
            content: format!("cat: {}", e),
            line_type: LineType::Error,
        }],
    }
}

#[cfg(all(feature = "desktop", target_os = "windows"))]
fn grep_file_lines(pattern: &str, path: &std::path::Path) -> Vec<TerminalLine> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return vec![TerminalLine {
                content: format!("grep: {}", e),
                line_type: LineType::Error,
            }]
        }
    };

    let mut out = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if line.contains(pattern) {
            out.push(TerminalLine {
                content: format!("{}:{}", idx + 1, line),
                line_type: LineType::Output,
            });
        }
    }

    if out.is_empty() {
        out.push(TerminalLine {
            content: "(no matches)".to_string(),
            line_type: LineType::Output,
        });
    }

    out
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
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

                push_line_trim(
                    lines,
                    TerminalLine {
                        content: format!("{} > {}", cwd, cmd),
                        line_type: LineType::Command,
                    },
                );
                input_value.set(String::new());

                let args = split_args(&cmd);
                let first = args
                    .first()
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

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
                            #[cfg(target_os = "windows")]
                            "Allowed system commands: ls, dir, echo, vim, mkdir, rm/del, mv, whoami, cat/type, grep, curl, wget, ipconfig (ip).",
                            #[cfg(not(target_os = "windows"))]
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
                        std::process::exit(0);
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

                if !ALLOWED_EXTERNAL.contains(&first.as_str()) {
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

                // Run allowed commands without invoking a shell.
                #[cfg(target_os = "windows")]
                {
                    let lines_sig = lines;
                    let program = first.clone();
                    let argv = args;
                    spawn(async move {
                        let result = tokio::task::spawn_blocking(move || -> Vec<TerminalLine> {
                            match program.as_str() {
                                "dir" | "ls" => {
                                    let target = argv.get(1).map(|s| s.as_str()).unwrap_or(".");
                                    let path = resolve_in_dir(&cwd, target);
                                    list_dir_lines(&path)
                                }
                                "echo" => {
                                    let text = argv.iter().skip(1).cloned().collect::<Vec<_>>().join(" ");
                                    vec![TerminalLine { content: text, line_type: LineType::Output }]
                                }
                                "whoami" => {
                                    let user = std::env::var("USERNAME")
                                        .or_else(|_| std::env::var("USER"))
                                        .unwrap_or_else(|_| "unknown".to_string());
                                    vec![TerminalLine { content: user, line_type: LineType::Output }]
                                }
                                "mkdir" => {
                                    if argv.len() < 2 {
                                        return vec![TerminalLine { content: "Usage: mkdir <dir>".into(), line_type: LineType::Error }];
                                    }
                                    let path = resolve_in_dir(&cwd, &argv[1]);
                                    match std::fs::create_dir_all(&path) {
                                        Ok(()) => vec![TerminalLine { content: "Directory created".into(), line_type: LineType::Output }],
                                        Err(e) => vec![TerminalLine { content: format!("mkdir: {}", e), line_type: LineType::Error }],
                                    }
                                }
                                "rm" | "del" => {
                                    if argv.len() < 2 {
                                        return vec![TerminalLine { content: "Usage: rm [-r] <path>".into(), line_type: LineType::Error }];
                                    }
                                    let mut recursive = false;
                                    let mut idx = 1;
                                    if argv.get(1).map(|s| s.as_str()) == Some("-r") || argv.get(1).map(|s| s.as_str()) == Some("-R") {
                                        recursive = true;
                                        idx = 2;
                                    }
                                    let Some(target) = argv.get(idx) else {
                                        return vec![TerminalLine { content: "Usage: rm [-r] <path>".into(), line_type: LineType::Error }];
                                    };
                                    let path = resolve_in_dir(&cwd, target);
                                    let result = match std::fs::metadata(&path) {
                                        Ok(m) if m.is_dir() => {
                                            if recursive {
                                                std::fs::remove_dir_all(&path)
                                            } else {
                                                Err(std::io::Error::new(std::io::ErrorKind::Other, "Is a directory (use rm -r)"))
                                            }
                                        }
                                        Ok(_) => std::fs::remove_file(&path),
                                        Err(e) => Err(e),
                                    };
                                    match result {
                                        Ok(()) => vec![TerminalLine { content: "Deleted".into(), line_type: LineType::Output }],
                                        Err(e) => vec![TerminalLine { content: format!("rm: {}", e), line_type: LineType::Error }],
                                    }
                                }
                                "mv" => {
                                    if argv.len() < 3 {
                                        return vec![TerminalLine { content: "Usage: mv <from> <to>".into(), line_type: LineType::Error }];
                                    }
                                    let from = resolve_in_dir(&cwd, &argv[1]);
                                    let to = resolve_in_dir(&cwd, &argv[2]);
                                    match std::fs::rename(&from, &to) {
                                        Ok(()) => vec![TerminalLine { content: "Moved".into(), line_type: LineType::Output }],
                                        Err(e) => vec![TerminalLine { content: format!("mv: {}", e), line_type: LineType::Error }],
                                    }
                                }
                                "cat" | "type" => {
                                    if argv.len() < 2 {
                                        return vec![TerminalLine { content: "Usage: cat <file>".into(), line_type: LineType::Error }];
                                    }
                                    let path = resolve_in_dir(&cwd, &argv[1]);
                                    read_file_lines(&path)
                                }
                                "grep" => {
                                    if argv.len() < 3 {
                                        return vec![TerminalLine { content: "Usage: grep <pattern> <file>".into(), line_type: LineType::Error }];
                                    }
                                    let pat = &argv[1];
                                    let path = resolve_in_dir(&cwd, &argv[2]);
                                    grep_file_lines(pat, &path)
                                }
                                "vim" => vec![TerminalLine {
                                    content: "vim is not supported in this UI (interactive TTY required).".into(),
                                    line_type: LineType::Error,
                                }],
                                "ip" => {
                                    let extra_args = argv.iter().skip(1).cloned().collect::<Vec<_>>();
                                    run_external_command_lines(&cwd, "ipconfig", &extra_args)
                                }
                                "ipconfig" | "curl" | "wget" => {
                                    let extra_args = argv.iter().skip(1).cloned().collect::<Vec<_>>();
                                    run_external_command_lines(&cwd, &program, &extra_args)
                                }
                                _ => vec![TerminalLine {
                                    content: format!("Unhandled command: {}", program),
                                    line_type: LineType::Error,
                                }],
                            }
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
                    let mut lines = lines;
                    let program = first.clone();
                    let program_args = args.iter().skip(1).cloned().collect::<Vec<_>>();

                    spawn(async move {
                        let child = Command::new(&program)
                            .args(&program_args)
                            .current_dir(&cwd)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn();

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
                                    push_line_trim(lines, line);
                                }

                                let _ = child.wait().await;
                            }
                            Err(e) => {
                                push_line_trim(
                                    lines,
                                    TerminalLine {
                                        content: format!("Error: {}", e),
                                        line_type: LineType::Error,
                                    },
                                );
                            }
                        }
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

        push_line_trim(
            lines,
            TerminalLine {
                content: format!("{} > {}", demo_dir, cmd),
                line_type: LineType::Command,
            },
        );
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
                    "  curl <url>    Fetch a URL (simulated)",
                    "  wget <url>    Fetch a URL (simulated)",
                    "  whoami        Show current user",
                    "  date          Show the date",
                    "  ipconfig      Show network configuration (Windows-style)",
                    "  ifconfig      Show network configuration (Unix-style)",
                    "  ip            Show network configuration (simulated)",
                    "  ping <host>   Test network connectivity",
                    "  mkdir <dir>   Create a directory",
                    "  rm / del <p>  Delete a file or directory",
                    "  mv <a> <b>    Move or rename",
                    "  pwd           Print working directory",
                    "  cat / type <file>  Print a file",
                    "  grep <pat> <file>  Find text in a file",
                ] {
                    push_line_trim(lines, TerminalLine { content: line.into(), line_type: LineType::System });
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
                    push_line_trim(lines, TerminalLine { content: line.into(), line_type: LineType::Output });
                }
            }
            "echo" => {
                let text = if cmd.len() > 5 { cmd[5..].to_string() } else { String::new() };
                push_line_trim(lines, TerminalLine { content: text, line_type: LineType::Output });
            }
            "whoami" => {
                push_line_trim(lines, TerminalLine { content: "You".into(), line_type: LineType::Output });
            }
            "curl" | "wget" => {
                let mut parts = cmd.split_whitespace();
                let _ = parts.next();
                let url = parts.next();
                match url {
                    Some(url) => {
                        for line in [
                            format!("(simulated) fetching {}...", url),
                            "HTTP/1.1 200 OK".to_string(),
                            "content-type: text/html; charset=utf-8".to_string(),
                            "".to_string(),
                            "<html>... (body truncated) ...</html>".to_string(),
                        ] {
                            push_line_trim(lines, TerminalLine { content: line, line_type: LineType::Output });
                        }
                    }
                    None => {
                        push_line_trim(lines, TerminalLine { content: "Usage: curl <url>".into(), line_type: LineType::Error });
                    }
                }
            }
            "pwd" => {
                push_line_trim(lines, TerminalLine { content: demo_dir.into(), line_type: LineType::Output });
            }
            "cat" | "type" => {
                if cmd_lower.split_whitespace().count() < 2 {
                    push_line_trim(lines, TerminalLine { content: "Usage: cat <file>".into(), line_type: LineType::Error });
                } else {
                    push_line_trim(lines, TerminalLine { content: "(simulated) file contents...".into(), line_type: LineType::Output });
                }
            }
            "grep" => {
                if cmd_lower.split_whitespace().count() < 3 {
                    push_line_trim(lines, TerminalLine { content: "Usage: grep <pattern> <file>".into(), line_type: LineType::Error });
                } else {
                    push_line_trim(lines, TerminalLine { content: "(simulated) matching lines...".into(), line_type: LineType::Output });
                }
            }
            "date" => {
                push_line_trim(lines, TerminalLine { content: "Fri 02/07/2026".into(), line_type: LineType::Output });
            }
            "ip" | "ipconfig" => {
                for line in [
                    "Windows IP Configuration",
                    "",
                    "Ethernet adapter Ethernet:",
                    "   IPv4 Address. . . . . : 192.168.1.100",
                    "   Subnet Mask . . . . . : 255.255.255.0",
                    "   Default Gateway . . . : 192.168.1.1",
                ] {
                    push_line_trim(lines, TerminalLine { content: line.into(), line_type: LineType::Output });
                }
            }
            "ifconfig" => {
                for line in [
                    "eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500",
                    "        inet 192.168.1.100  netmask 255.255.255.0  broadcast 192.168.1.255",
                    "        inet6 fe80::1  prefixlen 64  scopeid 0x20<link>",
                    "        ether aa:bb:cc:dd:ee:ff  txqueuelen 1000  (Ethernet)",
                ] {
                    push_line_trim(lines, TerminalLine { content: line.into(), line_type: LineType::Output });
                }
            }
            "ping" => {
                if cmd_lower.split_whitespace().count() < 2 {
                    push_line_trim(lines, TerminalLine { content: "Usage: ping <host>".into(), line_type: LineType::Error });
                } else {
                    for line in [
                        "Pinging host with 32 bytes of data:",
                        "Reply from 93.184.216.34: bytes=32 time=12ms TTL=56",
                        "Reply from 93.184.216.34: bytes=32 time=11ms TTL=56",
                        "Reply from 93.184.216.34: bytes=32 time=13ms TTL=56",
                    ] {
                        push_line_trim(lines, TerminalLine { content: line.into(), line_type: LineType::Output });
                    }
                }
            }
            "mkdir" => {
                if cmd_lower.split_whitespace().count() < 2 {
                    push_line_trim(lines, TerminalLine { content: "Usage: mkdir <dir>".into(), line_type: LineType::Error });
                } else {
                    push_line_trim(lines, TerminalLine { content: "Directory created (simulated)".into(), line_type: LineType::Output });
                }
            }
            "rm" | "del" => {
                if cmd_lower.split_whitespace().count() < 2 {
                    push_line_trim(lines, TerminalLine { content: "Usage: rm <path>".into(), line_type: LineType::Error });
                } else {
                    push_line_trim(lines, TerminalLine { content: "Deleted (simulated)".into(), line_type: LineType::Output });
                }
            }
            "mv" => {
                if cmd_lower.split_whitespace().count() < 3 {
                    push_line_trim(lines, TerminalLine { content: "Usage: mv <source> <dest>".into(), line_type: LineType::Error });
                } else {
                    push_line_trim(lines, TerminalLine { content: "Moved (simulated)".into(), line_type: LineType::Output });
                }
            }
            "cd" => {
                push_line_trim(lines, TerminalLine {
                    content: "Directory changed (simulated)".into(),
                    line_type: LineType::System,
                });
            }
            "exit" => {
                push_line_trim(lines, TerminalLine {
                    content: "Can't exit the web demo! Download the real thing.".into(),
                    line_type: LineType::System,
                });
            }
            _ => {
                push_line_trim(lines, TerminalLine {
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
