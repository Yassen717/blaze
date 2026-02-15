mod fs;
mod process;

use crate::terminal::state::{LineType, TerminalLine};

pub fn is_allowed_external(command: &str) -> bool {
    match command {
        "ls" | "dir" | "echo" | "vim" | "whoami" | "cat" | "grep" | "curl" | "wget" | "ip" => true,
        #[cfg(target_os = "windows")]
        "type" | "ipconfig" => true,
        #[cfg(not(target_os = "windows"))]
        "ifconfig" => true,
        #[cfg(all(not(feature = "safe-mode"), feature = "unsafe-fs"))]
        "mkdir" | "rm" | "del" | "mv" => true,
        _ => false,
    }
}

#[cfg(target_os = "windows")]
pub fn execute_windows_command(cwd: &str, program: &str, argv: &[String]) -> Vec<TerminalLine> {
    if let Some(lines) = fs::handle_windows_fs_command(cwd, program, argv) {
        return lines;
    }

    if let Some(lines) = process::handle_windows_process_command(cwd, program, argv) {
        return lines;
    }

    vec![TerminalLine {
        content: format!("Unhandled command: {}", program),
        line_type: LineType::Error,
    }]
}

#[cfg(not(target_os = "windows"))]
pub use process::stream_unix_command;
