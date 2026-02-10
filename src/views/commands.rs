use dioxus::prelude::*;

use crate::components::CmdCard;

#[component]
pub fn CommandsPage() -> Element {
    rsx! {
        section { class: "page-section",
            h1 { class: "page-title", "Command Reference" }
            p { class: "page-intro",
                "Blaze includes built-in commands and a small set of allowed system commands."
            }

            h2 { "Built-in Commands" }
            div { class: "commands-grid",
                CmdCard { cmd: "help", desc: "Show available commands", example: "help" }
                CmdCard { cmd: "clear / cls", desc: "Clear the terminal screen", example: "clear" }
                CmdCard { cmd: "cd <dir>", desc: "Change working directory", example: "cd C:\\Projects" }
                CmdCard { cmd: "exit", desc: "Quit Blaze Terminal", example: "exit" }
            }

            h2 { "Allowed System Commands" }
            div { class: "commands-grid",
                CmdCard { cmd: "dir", desc: "List files and folders", example: "dir" }
                CmdCard { cmd: "ls", desc: "List files and folders", example: "ls" }
                CmdCard { cmd: "echo <text>", desc: "Print text to the terminal", example: "echo Hello!" }
                CmdCard { cmd: "vim", desc: "Open the Vim editor", example: "vim readme.txt" }
                CmdCard { cmd: "mkdir <dir>", desc: "Create a directory", example: "mkdir src" }
                CmdCard { cmd: "rm / del <path>", desc: "Delete files or directories", example: "rm temp.txt" }
                CmdCard { cmd: "mv <from> <to>", desc: "Move or rename", example: "mv old.txt new.txt" }
            }
        }
    }
}
