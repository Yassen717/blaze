mod fs;
mod process;

use crate::terminal::state::{LineType, TerminalLine};

#[cfg(all(
    feature = "desktop",
    not(target_arch = "wasm32"),
    target_os = "windows",
    not(feature = "safe-mode")
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
    target_os = "windows",
    feature = "safe-mode"
))]
const ALLOWED_EXTERNAL: [&str; 12] = [
    "ls",
    "dir",
    "echo",
    "vim",
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
    not(target_os = "windows"),
    not(feature = "safe-mode")
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

#[cfg(all(
    feature = "desktop",
    not(target_arch = "wasm32"),
    not(target_os = "windows"),
    feature = "safe-mode"
))]
const ALLOWED_EXTERNAL: [&str; 11] = [
    "ls",
    "dir",
    "echo",
    "vim",
    "whoami",
    "cat",
    "grep",
    "curl",
    "wget",
    "ifconfig",
    "ip",
];

pub fn is_allowed_external(command: &str) -> bool {
    ALLOWED_EXTERNAL.contains(&command)
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
