use dioxus::prelude::*;

use crate::components::CmdCard;

#[component]
pub fn CommandsPage() -> Element {
    rsx! {
        section { class: "page-section",
            h1 { class: "page-title", "Command Reference" }
            p { class: "page-intro",
                "Blaze v0.2.1 runs a real shell inside a PTY on desktop. Command parsing, history, and completion come from your shell."
            }

            h2 { "Desktop Terminal (PTY-backed)" }
            div { class: "commands-grid",
                CmdCard { cmd: "Any shell command", desc: "Run whatever your shell supports.", example: "git log --oneline" }
                CmdCard { cmd: "Pipes and redirects", desc: "Use native shell syntax.", example: "cargo test | findstr error" }
                CmdCard { cmd: "Interactive apps", desc: "Full-screen TUIs and REPLs are supported.", example: "vim src/main.rs" }
                CmdCard { cmd: "Shell history", desc: "Arrow keys and shell-native search work.", example: "ArrowUp / Ctrl+R" }
                CmdCard { cmd: "Signals", desc: "Common control keys are forwarded to PTY.", example: "Ctrl+C / Ctrl+D / Ctrl+Z" }
            }

            h2 { "Web Demo (Simulated)" }
            div { class: "commands-grid",
                CmdCard { cmd: "help", desc: "Show demo command list", example: "help" }
                CmdCard { cmd: "clear / cls", desc: "Clear the simulated output", example: "clear" }
                CmdCard { cmd: "dir / ls", desc: "List sample files and folders", example: "dir" }
                CmdCard { cmd: "echo <text>", desc: "Print text to the terminal", example: "echo Hello!" }
                CmdCard { cmd: "curl / wget <url>", desc: "Return simulated fetch output", example: "curl https://example.com" }
                CmdCard { cmd: "cat / type / grep", desc: "Simulated file operations", example: "grep todo notes.txt" }
                CmdCard { cmd: "whoami", desc: "Show current user", example: "whoami" }
                CmdCard { cmd: "pwd / date", desc: "Show simulated environment info", example: "pwd" }
                CmdCard { cmd: "ip / ipconfig / ifconfig", desc: "Show simulated network info", example: "ipconfig" }
                CmdCard { cmd: "mkdir / rm / del / mv", desc: "Simulated mutating operations", example: "mv a.txt b.txt" }
            }

        }
    }
}
