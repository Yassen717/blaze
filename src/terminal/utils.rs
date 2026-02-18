use dioxus::prelude::*;

use crate::terminal::state::TerminalLine;

const MAX_LINES: usize = 5000;

// ======================== History persistence ========================

/// Returns the path to the Blaze command-history file.
/// Stored at `<user home>/.blaze_history`; falls back to the current directory.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub fn history_file_path() -> std::path::PathBuf {
    let base = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    base.join(".blaze_history")
}

/// Load up to `limit` most-recent history lines from disk.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub fn load_history(limit: usize) -> Vec<String> {
    let path = history_file_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => content
            .lines()
            .map(|l| l.to_string())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .take(limit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Append a single command to the history file (one entry per line).
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub fn append_history(cmd: &str) {
    use std::io::Write;
    let path = history_file_path();
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{}", cmd);
    }
}

// ======================== Tab completion ========================

/// All built-in command names available in the desktop terminal.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const BUILTIN_COMMANDS: &[&str] = &["help", "clear", "cls", "cd", "pwd", "exit"];

/// All commands that may be passed through to the OS (mirrors `is_allowed_external`).
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const EXTERNAL_COMMANDS: &[&str] = &[
    "ls", "dir", "echo", "vim", "whoami", "cat", "grep", "curl", "wget", "ip",
    #[cfg(target_os = "windows")]
    "type",
    #[cfg(target_os = "windows")]
    "ipconfig",
    #[cfg(not(target_os = "windows"))]
    "ifconfig",
];

/// Given the current raw input string, return the next completion candidate.
///
/// Strategy:
/// * If only one token is present (typing a command name) → complete against
///   built-ins + externals.
/// * If multiple tokens are present (typing an argument) → complete against
///   filesystem entries under `cwd` that match the current argument prefix.
///
/// `tab_state` tracks how many times Tab has been pressed consecutively so we
/// cycle through multiple matches.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub fn tab_complete(input: &str, cwd: &str, tab_state: usize) -> Option<String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    if tokens.is_empty() {
        return None;
    }

    let completing_cmd = tokens.len() == 1 && !input.ends_with(' ');

    if completing_cmd {
        // Complete the command name itself.
        let prefix = tokens[0].to_lowercase();
        let mut matches: Vec<String> = BUILTIN_COMMANDS
            .iter()
            .chain(EXTERNAL_COMMANDS.iter())
            .filter(|c| c.starts_with(prefix.as_str()))
            .map(|c| c.to_string())
            .collect();
        matches.sort();
        matches.dedup();
        if matches.is_empty() {
            return None;
        }
        let chosen = &matches[tab_state % matches.len()];
        Some(chosen.clone())
    } else {
        // Complete the last token as a filesystem path under cwd.
        let partial = if input.ends_with(' ') {
            ""
        } else {
            tokens.last().copied().unwrap_or("")
        };

        let (dir_part, file_prefix) = if let Some(sep) = partial.rfind(['/', '\\']) {
            (&partial[..=sep], &partial[sep + 1..])
        } else {
            ("", partial)
        };

        let search_dir = if dir_part.is_empty() {
            std::path::PathBuf::from(cwd)
        } else {
            let p = std::path::Path::new(dir_part);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                std::path::Path::new(cwd).join(dir_part)
            }
        };

        let entries: Vec<String> = std::fs::read_dir(&search_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                // Append separator for directories to make it obvious.
                if e.path().is_dir() {
                    #[cfg(target_os = "windows")]
                    return format!("{name}\\");
                    #[cfg(not(target_os = "windows"))]
                    return format!("{name}/");
                }
                name
            })
            .filter(|name| {
                name.to_lowercase()
                    .starts_with(&file_prefix.to_lowercase())
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        if entries.is_empty() {
            return None;
        }
        let chosen = &entries[tab_state % entries.len()];

        // Reconstruct the full new input: keep everything before the last token.
        let prefix_tokens = if input.ends_with(' ') {
            input.to_string()
        } else {
            // Drop the last token from the input string, keep the rest.
            let last_token_start = input.rfind(|c: char| c.is_whitespace())
                .map(|i| i + 1)
                .unwrap_or(0);
            input[..last_token_start].to_string()
        };

        Some(format!("{}{}{}", prefix_tokens, dir_part, chosen))
    }
}

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
