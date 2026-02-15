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

#[cfg(test)]
mod tests {
    use super::is_allowed_external;

    #[test]
    fn allows_core_non_mutating_commands() {
        for cmd in ["ls", "dir", "echo", "whoami", "cat", "grep", "curl", "wget", "ip"] {
            assert!(is_allowed_external(cmd), "expected '{cmd}' to be allowed");
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn allows_windows_specific_non_mutating_commands() {
        assert!(is_allowed_external("type"));
        assert!(is_allowed_external("ipconfig"));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn allows_unix_specific_non_mutating_commands() {
        assert!(is_allowed_external("ifconfig"));
    }

    #[cfg(all(not(feature = "safe-mode"), feature = "unsafe-fs"))]
    #[test]
    fn allows_mutating_commands_when_unsafe_fs_enabled() {
        for cmd in ["mkdir", "rm", "del", "mv"] {
            assert!(is_allowed_external(cmd), "expected '{cmd}' to be allowed");
        }
    }

    #[cfg(not(all(not(feature = "safe-mode"), feature = "unsafe-fs")))]
    #[test]
    fn denies_mutating_commands_when_unsafe_fs_disabled() {
        for cmd in ["mkdir", "rm", "del", "mv"] {
            assert!(!is_allowed_external(cmd), "expected '{cmd}' to be denied");
        }
    }
}
