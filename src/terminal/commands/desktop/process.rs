#[cfg(not(target_os = "windows"))]
use dioxus::prelude::*;

#[cfg(not(target_os = "windows"))]
use crate::terminal::utils::push_line_trim;
#[cfg(target_os = "windows")]
use crate::terminal::state::LineType;
use crate::terminal::state::TerminalLine;

#[cfg(all(feature = "desktop", target_os = "windows"))]
const MAX_CMD_OUTPUT_BYTES: usize = 1024 * 1024;

#[cfg(target_os = "windows")]
pub fn handle_windows_process_command(
    cwd: &str,
    program: &str,
    argv: &[String],
) -> Option<Vec<TerminalLine>> {
    match program {
        "echo" => {
            let text = argv.iter().skip(1).cloned().collect::<Vec<_>>().join(" ");
            Some(vec![TerminalLine {
                content: text,
                line_type: LineType::Output,
            }])
        }
        "whoami" => {
            let user = std::env::var("USERNAME")
                .or_else(|_| std::env::var("USER"))
                .unwrap_or_else(|_| "unknown".to_string());
            Some(vec![TerminalLine {
                content: user,
                line_type: LineType::Output,
            }])
        }
        "vim" => Some(vec![TerminalLine {
            content: "vim is not supported in this UI (interactive TTY required).".into(),
            line_type: LineType::Error,
        }]),
        "ip" => {
            let extra_args = argv.iter().skip(1).cloned().collect::<Vec<_>>();
            Some(run_external_command_lines(cwd, "ipconfig", &extra_args))
        }
        "ipconfig" | "curl" | "wget" => {
            let extra_args = argv.iter().skip(1).cloned().collect::<Vec<_>>();
            Some(run_external_command_lines(cwd, program, &extra_args))
        }
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn run_external_command_lines(cwd: &str, program: &str, args: &[String]) -> Vec<TerminalLine> {
    use std::process::Command;

    let output = Command::new(program).args(args).current_dir(cwd).output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            return vec![TerminalLine {
                content: format!("{}: {}", program, e),
                line_type: LineType::Error,
            }]
        }
    };

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&output.stdout);
    bytes.extend_from_slice(&output.stderr);

    if bytes.len() > MAX_CMD_OUTPUT_BYTES {
        bytes.truncate(MAX_CMD_OUTPUT_BYTES);
        bytes.extend_from_slice(b"\n...(output truncated)\n");
    }

    let text = String::from_utf8_lossy(&bytes);
    let line_type = if output.status.success() {
        LineType::Output
    } else {
        LineType::Error
    };

    let mut out = Vec::new();
    for line in text.lines() {
        out.push(TerminalLine {
            content: line.to_string(),
            line_type: line_type.clone(),
        });
    }

    if out.is_empty() {
        out.push(TerminalLine {
            content: String::new(),
            line_type,
        });
    }

    out
}

#[cfg(not(target_os = "windows"))]
pub async fn stream_unix_command(
    cwd: String,
    program: String,
    program_args: Vec<String>,
    mut lines: Signal<Vec<TerminalLine>>,
) {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;
    use tokio::sync::mpsc;

    let child = Command::new(&program)
        .args(&program_args)
        .current_dir(&cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(mut child) => {
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let (tx, mut rx) = mpsc::unbounded_channel::<TerminalLine>();

            if let Some(out) = stdout {
                let tx_out = tx.clone();
                spawn(async move {
                    let mut reader = BufReader::new(out).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        let _ = tx_out.send(TerminalLine {
                            content: line,
                            line_type: LineType::Output,
                        });
                    }
                });
            }

            if let Some(err) = stderr {
                let tx_err = tx.clone();
                spawn(async move {
                    let mut reader = BufReader::new(err).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        let _ = tx_err.send(TerminalLine {
                            content: line,
                            line_type: LineType::Error,
                        });
                    }
                });
            }

            drop(tx);

            while let Some(line) = rx.recv().await {
                push_line_trim(lines, line);
            }

            let _ = child.wait().await;
        }
        Err(e) => {
            push_line_trim(
                lines,
                TerminalLine {
                    content: format!("Error: {}", e),
                    line_type: LineType::Error,
                },
            );
        }
    }
}
