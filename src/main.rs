#![allow(non_snake_case)]

use dioxus::prelude::*;

// ======================== Assets ========================

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// ======================== Shared Types ========================

#[derive(Clone, Debug, PartialEq)]
struct TerminalLine {
    content: String,
    line_type: LineType,
}

#[derive(Clone, Debug, PartialEq)]
enum LineType {
    Command,
    Output,
    Error,
    System,
}

// ======================== Main ========================

fn main() {
    dioxus::launch(App);
}

// ======================== Root App ========================

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        AppInner {}
    }
}

/// Desktop: render the real terminal
#[cfg(feature = "desktop")]
#[component]
fn AppInner() -> Element {
    rsx! {
        document::Title { "⚡ Blaze Terminal" }
        DesktopTerminal {}
    }
}

/// Web: render the showcase website
#[cfg(not(feature = "desktop"))]
#[component]
fn AppInner() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

// ================================================================
//  DESKTOP TERMINAL — real command execution
// ================================================================

#[cfg(feature = "desktop")]
const MAX_LINES: usize = 5000;

#[cfg(feature = "desktop")]
#[component]
fn DesktopTerminal() -> Element {
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

    // ---- keyboard handler ----
    let handle_key = move |e: KeyboardEvent| {
        match e.key() {
            Key::Enter => {
                let cmd = input_value().trim().to_string();
                if cmd.is_empty() {
                    return;
                }
                let cwd = current_dir().clone();

                // record history
                cmd_history.write().push(cmd.clone());
                history_idx.set(-1);

                // echo the command
                lines.write().push(TerminalLine {
                    content: format!("{} > {}", cwd, cmd),
                    line_type: LineType::Command,
                });
                input_value.set(String::new());

                // ---- built-in commands ----
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
                            "Any other input is executed as a system command.",
                        ];
                        for h in help {
                            lines.write().push(TerminalLine {
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
                                let clean =
                                    s.strip_prefix(r"\\?\").unwrap_or(&s).to_string();
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

                // ---- external command ----
                spawn(async move {
                    #[cfg(target_os = "windows")]
                    let result = std::process::Command::new("cmd")
                        .args(["/C", &cmd])
                        .current_dir(&cwd)
                        .output();

                    #[cfg(not(target_os = "windows"))]
                    let result = std::process::Command::new("sh")
                        .args(["-c", &cmd])
                        .current_dir(&cwd)
                        .output();

                    match result {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            for line in stdout.lines() {
                                lines.write().push(TerminalLine {
                                    content: line.to_string(),
                                    line_type: LineType::Output,
                                });
                            }
                            for line in stderr.lines() {
                                lines.write().push(TerminalLine {
                                    content: line.to_string(),
                                    line_type: LineType::Error,
                                });
                            }
                        }
                        Err(e) => {
                            lines.write().push(TerminalLine {
                                content: format!("Error: {}", e),
                                line_type: LineType::Error,
                            });
                        }
                    }

                    // cap lines
                    let mut v = lines.write();
                    if v.len() > MAX_LINES {
                        let excess = v.len() - MAX_LINES;
                        v.drain(0..excess);
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

    // auto-scroll when lines change
    use_effect(move || {
        let _ = lines();
        document::eval(
            r#"setTimeout(()=>{let e=document.getElementById('terminal-output');if(e)e.scrollTop=e.scrollHeight},10)"#,
        );
    });

    rsx! {
        div { class: "terminal-container terminal-fullscreen",
            div { class: "terminal-header",
                div { class: "terminal-dots",
                    span { class: "dot dot-red" }
                    span { class: "dot dot-yellow" }
                    span { class: "dot dot-green" }
                }
                span { class: "terminal-title", "⚡ Blaze Terminal" }
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

// ================================================================
//  WEB ROUTES & PAGES — showcase website
// ================================================================

#[cfg(not(feature = "desktop"))]
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(WebLayout)]
        #[route("/")]
        Home {},
        #[route("/commands")]
        CommandsPage {},
        #[route("/demo")]
        DemoPage {},
}

// ---- layout ----

#[cfg(not(feature = "desktop"))]
#[component]
fn WebLayout() -> Element {
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

// ---- home / landing page ----

#[cfg(not(feature = "desktop"))]
#[component]
fn Home() -> Element {
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
                Link { to: Route::CommandsPage {}, class: "btn-secondary", "View Commands →" }
            }
        }

        section { class: "features-section",
            h2 { class: "section-title", "Why Blaze?" }
            div { class: "features-grid",
                FeatureCard { icon: "⚡", title: "Blazingly Fast", desc: "Built in Rust for native speed. Commands execute instantly." }
                FeatureCard { icon: "🎨", title: "Beautiful UI", desc: "Modern interface with color-coded output and smooth scrolling." }
                FeatureCard { icon: "📝", title: "Command History", desc: "Navigate previous commands with the arrow keys." }
                FeatureCard { icon: "🔧", title: "Built-in Commands", desc: "Handy built-ins plus full system shell access." }
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

#[cfg(not(feature = "desktop"))]
#[component]
fn FeatureCard(icon: &'static str, title: &'static str, desc: &'static str) -> Element {
    rsx! {
        div { class: "feature-card",
            div { class: "feature-icon", "{icon}" }
            h3 { "{title}" }
            p { "{desc}" }
        }
    }
}

// ---- commands reference page ----

#[cfg(not(feature = "desktop"))]
#[component]
fn CommandsPage() -> Element {
    rsx! {
        section { class: "page-section",
            h1 { class: "page-title", "Command Reference" }
            p { class: "page-intro",
                "Blaze includes built-in commands. Everything else is forwarded to your system shell."
            }

            h2 { "Built-in Commands" }
            div { class: "commands-grid",
                CmdCard { cmd: "help", desc: "Show available commands", example: "help" }
                CmdCard { cmd: "clear / cls", desc: "Clear the terminal screen", example: "clear" }
                CmdCard { cmd: "cd <dir>", desc: "Change working directory", example: "cd C:\\Projects" }
                CmdCard { cmd: "exit", desc: "Quit Blaze Terminal", example: "exit" }
            }

            h2 { "System Commands (examples)" }
            div { class: "commands-grid",
                CmdCard { cmd: "dir", desc: "List files and folders", example: "dir" }
                CmdCard { cmd: "echo <text>", desc: "Print text to the terminal", example: "echo Hello!" }
                CmdCard { cmd: "mkdir <name>", desc: "Create a new directory", example: "mkdir my-project" }
                CmdCard { cmd: "type <file>", desc: "Display file contents", example: "type readme.txt" }
                CmdCard { cmd: "ping <host>", desc: "Test network connectivity", example: "ping google.com" }
                CmdCard { cmd: "ipconfig", desc: "Show network configuration", example: "ipconfig" }
            }
        }
    }
}

#[cfg(not(feature = "desktop"))]
#[component]
fn CmdCard(cmd: &'static str, desc: &'static str, example: &'static str) -> Element {
    rsx! {
        div { class: "command-card",
            code { class: "cmd-name", "{cmd}" }
            p { "{desc}" }
            pre { class: "cmd-example", "> {example}" }
        }
    }
}

// ---- demo page ----

#[cfg(not(feature = "desktop"))]
#[component]
fn DemoPage() -> Element {
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

// ================================================================
//  WEB TERMINAL DEMO — simulated commands
// ================================================================

#[cfg(not(feature = "desktop"))]
#[component]
fn WebTerminalDemo() -> Element {
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

        // echo command
        lines.write().push(TerminalLine {
            content: format!("{} > {}", demo_dir, cmd),
            line_type: LineType::Command,
        });
        input_value.set(String::new());

        // simulate responses
        let cmd_lower = cmd.to_lowercase();
        let first = cmd_lower.split_whitespace().next().unwrap_or("");

        match first {
            "clear" | "cls" => {
                lines.write().clear();
                return;
            }
            "help" => {
                for line in [
                    "⚡ Blaze Terminal — Commands:",
                    "",
                    "  help          Show this message",
                    "  clear / cls   Clear the screen",
                    "  cd <dir>      Change directory",
                    "  exit          Exit terminal",
                    "",
                    "Try: dir, echo, whoami, date, ipconfig, ping",
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

    // auto-scroll
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
