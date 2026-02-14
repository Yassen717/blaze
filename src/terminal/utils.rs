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
