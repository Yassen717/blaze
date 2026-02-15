use dioxus::prelude::*;

use crate::terminal::state::TerminalLine;

const MAX_LINES: usize = 5000;

pub fn push_line_trim(mut lines: Signal<Vec<TerminalLine>>, line: TerminalLine) {
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
pub fn split_args(input: &str) -> Vec<String> {
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
pub fn resolve_in_dir(cwd: &str, target: &str) -> std::path::PathBuf {
    let target_path = std::path::Path::new(target);
    if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        std::path::Path::new(cwd).join(target_path)
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32"), target_os = "windows"))]
pub fn windows_hidden_command(program: &str, cwd: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut command = std::process::Command::new(program);
    command.current_dir(cwd).creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(all(test, feature = "desktop", not(target_arch = "wasm32")))]
mod tests {
    use super::split_args;

    #[test]
    fn split_args_handles_quoted_segments() {
        let args = split_args("echo \"hello world\" test");
        assert_eq!(args, vec!["echo", "hello world", "test"]);
    }

    #[test]
    fn split_args_handles_escaped_quote_inside_quotes() {
        let args = split_args("echo \"he\\\"llo\"");
        assert_eq!(args, vec!["echo", "he\"llo"]);
    }

    #[test]
    fn split_args_preserves_shell_metacharacters_as_literals() {
        let args = split_args("echo hello && rm file.txt");
        assert_eq!(args, vec!["echo", "hello", "&&", "rm", "file.txt"]);
    }

    #[test]
    fn split_args_collapses_whitespace_between_args() {
        let args = split_args("  grep    TODO   file.rs  ");
        assert_eq!(args, vec!["grep", "TODO", "file.rs"]);
    }
}
