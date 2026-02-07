use dioxus::prelude::*;

use crate::components::CmdCard;

#[component]
pub fn CommandsPage() -> Element {
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
