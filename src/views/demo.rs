use dioxus::prelude::*;

use crate::components::terminal::WebTerminalDemo;
use crate::components::CmdCard;

#[component]
pub fn DemoPage() -> Element {
    rsx! {
        section { class: "page-section",
            h1 { class: "page-title", "Interactive Demo" }
            p { class: "page-intro",
                "Try Blaze right in your browser! Type commands below."
            }
            WebTerminalDemo {}
            div { class: "demo-commands",
                h2 { class: "section-title", "Demo Commands" }
                div { class: "commands-grid",
                    CmdCard { cmd: "help", desc: "Show available commands", example: "help" }
                    CmdCard { cmd: "clear / cls", desc: "Clear the screen", example: "clear" }
                    CmdCard { cmd: "dir / ls", desc: "List files and folders", example: "dir" }
                    CmdCard { cmd: "echo <text>", desc: "Print text", example: "echo Hello" }
                    CmdCard { cmd: "curl <url>", desc: "Fetch a URL (simulated)", example: "curl https://example.com" }
                    CmdCard { cmd: "wget <url>", desc: "Fetch a URL (simulated)", example: "wget https://example.com" }
                    CmdCard { cmd: "ipconfig / ip", desc: "Show network config", example: "ipconfig" }
                    CmdCard { cmd: "ifconfig", desc: "Show network config", example: "ifconfig" }
                    CmdCard { cmd: "mkdir <dir>", desc: "Create a directory", example: "mkdir docs" }
                    CmdCard { cmd: "rm / del <path>", desc: "Delete files or folders", example: "rm temp.txt" }
                    CmdCard { cmd: "mv <from> <to>", desc: "Move or rename", example: "mv a.txt b.txt" }
                    CmdCard { cmd: "whoami", desc: "Show current user", example: "whoami" }
                    CmdCard { cmd: "pwd", desc: "Print working directory", example: "pwd" }
                    CmdCard { cmd: "cat / type <file>", desc: "Print a file", example: "cat notes.txt" }
                    CmdCard { cmd: "grep <pat> <file>", desc: "Find text in a file", example: "grep todo notes.txt" }
                }
            }
            p { class: "demo-note",
                "This is a simulated demo. Download Blaze for real command execution."
            }
        }
    }
}
