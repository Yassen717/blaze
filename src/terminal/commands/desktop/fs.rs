#[cfg(target_os = "windows")]
use crate::terminal::state::{LineType, TerminalLine};
#[cfg(target_os = "windows")]
use crate::terminal::utils::resolve_in_dir;

#[cfg(target_os = "windows")]
pub fn handle_windows_fs_command(
    cwd: &str,
    program: &str,
    argv: &[String],
) -> Option<Vec<TerminalLine>> {
    match program {
        "dir" | "ls" => {
            let target = argv.get(1).map(|s| s.as_str()).unwrap_or(".");
            let path = resolve_in_dir(cwd, target);
            Some(list_dir_lines(&path))
        }
        "mkdir" => {
            if argv.len() < 2 {
                return Some(vec![TerminalLine {
                    content: "Usage: mkdir <dir>".into(),
                    line_type: LineType::Error,
                }]);
            }
            let path = resolve_in_dir(cwd, &argv[1]);
            Some(match std::fs::create_dir_all(&path) {
                Ok(()) => vec![TerminalLine {
                    content: "Directory created".into(),
                    line_type: LineType::Output,
                }],
                Err(e) => vec![TerminalLine {
                    content: format!("mkdir: {}", e),
                    line_type: LineType::Error,
                }],
            })
        }
        "rm" | "del" => {
            if argv.len() < 2 {
                return Some(vec![TerminalLine {
                    content: "Usage: rm [-r] <path>".into(),
                    line_type: LineType::Error,
                }]);
            }
            let mut recursive = false;
            let mut idx = 1;
            if argv.get(1).map(|s| s.as_str()) == Some("-r")
                || argv.get(1).map(|s| s.as_str()) == Some("-R")
            {
                recursive = true;
                idx = 2;
            }
            let Some(target) = argv.get(idx) else {
                return Some(vec![TerminalLine {
                    content: "Usage: rm [-r] <path>".into(),
                    line_type: LineType::Error,
                }]);
            };
            let path = resolve_in_dir(cwd, target);
            let result = match std::fs::metadata(&path) {
                Ok(m) if m.is_dir() => {
                    if recursive {
                        std::fs::remove_dir_all(&path)
                    } else {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Is a directory (use rm -r)",
                        ))
                    }
                }
                Ok(_) => std::fs::remove_file(&path),
                Err(e) => Err(e),
            };
            Some(match result {
                Ok(()) => vec![TerminalLine {
                    content: "Deleted".into(),
                    line_type: LineType::Output,
                }],
                Err(e) => vec![TerminalLine {
                    content: format!("rm: {}", e),
                    line_type: LineType::Error,
                }],
            })
        }
        "mv" => {
            if argv.len() < 3 {
                return Some(vec![TerminalLine {
                    content: "Usage: mv <from> <to>".into(),
                    line_type: LineType::Error,
                }]);
            }
            let from = resolve_in_dir(cwd, &argv[1]);
            let to = resolve_in_dir(cwd, &argv[2]);
            Some(match std::fs::rename(&from, &to) {
                Ok(()) => vec![TerminalLine {
                    content: "Moved".into(),
                    line_type: LineType::Output,
                }],
                Err(e) => vec![TerminalLine {
                    content: format!("mv: {}", e),
                    line_type: LineType::Error,
                }],
            })
        }
        "cat" | "type" => {
            if argv.len() < 2 {
                return Some(vec![TerminalLine {
                    content: "Usage: cat <file>".into(),
                    line_type: LineType::Error,
                }]);
            }
            let path = resolve_in_dir(cwd, &argv[1]);
            Some(read_file_lines(&path))
        }
        "grep" => {
            if argv.len() < 3 {
                return Some(vec![TerminalLine {
                    content: "Usage: grep <pattern> <file>".into(),
                    line_type: LineType::Error,
                }]);
            }
            let pat = &argv[1];
            let path = resolve_in_dir(cwd, &argv[2]);
            Some(grep_file_lines(pat, &path))
        }
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn list_dir_lines(path: &std::path::Path) -> Vec<TerminalLine> {
    let mut out = Vec::new();
    match std::fs::read_dir(path) {
        Ok(entries) => {
            out.push(TerminalLine {
                content: format!(" Directory of {}", path.display()),
                line_type: LineType::Output,
            });
            out.push(TerminalLine {
                content: String::new(),
                line_type: LineType::Output,
            });

            let mut names: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            names.sort();

            for name in names {
                out.push(TerminalLine {
                    content: name,
                    line_type: LineType::Output,
                });
            }
        }
        Err(e) => out.push(TerminalLine {
            content: format!("dir: {}", e),
            line_type: LineType::Error,
        }),
    }
    out
}

#[cfg(target_os = "windows")]
fn read_file_lines(path: &std::path::Path) -> Vec<TerminalLine> {
    const MAX_BYTES: usize = 512 * 1024;
    match std::fs::read(path) {
        Ok(bytes) => {
            let bytes = if bytes.len() > MAX_BYTES {
                &bytes[..MAX_BYTES]
            } else {
                &bytes
            };
            let text = String::from_utf8_lossy(bytes);
            text.lines()
                .map(|l| TerminalLine {
                    content: l.to_string(),
                    line_type: LineType::Output,
                })
                .collect()
        }
        Err(e) => vec![TerminalLine {
            content: format!("cat: {}", e),
            line_type: LineType::Error,
        }],
    }
}

#[cfg(target_os = "windows")]
fn grep_file_lines(pattern: &str, path: &std::path::Path) -> Vec<TerminalLine> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return vec![TerminalLine {
                content: format!("grep: {}", e),
                line_type: LineType::Error,
            }]
        }
    };

    let mut out = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if line.contains(pattern) {
            out.push(TerminalLine {
                content: format!("{}:{}", idx + 1, line),
                line_type: LineType::Output,
            });
        }
    }

    if out.is_empty() {
        out.push(TerminalLine {
            content: "(no matches)".to_string(),
            line_type: LineType::Output,
        });
    }

    out
}
