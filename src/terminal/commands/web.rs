use dioxus::prelude::*;

use crate::terminal::state::{LineType, TerminalLine};
use crate::terminal::utils::push_line_trim;

pub fn run_web_command(cmd: &str, demo_dir: &str, mut lines: Signal<Vec<TerminalLine>>) {
    push_line_trim(
        lines,
        TerminalLine {
            content: format!("{} > {}", demo_dir, cmd),
            line_type: LineType::Command,
        },
    );

    let cmd_lower = cmd.to_lowercase();
    let first = cmd_lower.split_whitespace().next().unwrap_or("");

    match first {
        "clear" | "cls" => {
            lines.write().clear();
        }
        "help" => {
            for line in [
                "⚡ Blaze Terminal — Demo Commands:",
                "",
                "  help          Show this message",
                "  clear / cls   Clear the screen",
                "  dir / ls      List files and folders",
                "  echo <text>   Print text to the terminal",
                "  curl <url>    Fetch a URL (simulated)",
                "  wget <url>    Fetch a URL (simulated)",
                "  whoami        Show current user",
                "  date          Show the date",
                "  ipconfig      Show network configuration (Windows-style)",
                "  ifconfig      Show network configuration (Unix-style)",
                "  ip            Show network configuration (simulated)",
                "  ping <host>   Test network connectivity",
                "  mkdir <dir>   Create a directory",
                "  rm / del <p>  Delete a file or directory",
                "  mv <a> <b>    Move or rename",
                "  pwd           Print working directory",
                "  cat / type <file>  Print a file",
                "  grep <pat> <file>  Find text in a file",
            ] {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: line.into(),
                        line_type: LineType::System,
                    },
                );
            }
        }
        "dir" | "ls" => {
            for line in [
                " Directory of C:\\Users\\You",
                "",
                " 02/07/2026  10:00 AM    <DIR>          Documents",
                " 02/07/2026  10:00 AM    <DIR>          Projects",
                " 02/07/2026  10:00 AM    <DIR>          Downloads",
                " 01/15/2026  03:45 PM           2,048   notes.txt",
                "               1 File(s)         2,048 bytes",
            ] {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: line.into(),
                        line_type: LineType::Output,
                    },
                );
            }
        }
        "echo" => {
            let text = if cmd.len() > 5 {
                cmd[5..].to_string()
            } else {
                String::new()
            };
            push_line_trim(
                lines,
                TerminalLine {
                    content: text,
                    line_type: LineType::Output,
                },
            );
        }
        "whoami" => {
            push_line_trim(
                lines,
                TerminalLine {
                    content: "You".into(),
                    line_type: LineType::Output,
                },
            );
        }
        "curl" | "wget" => {
            let mut parts = cmd.split_whitespace();
            let _ = parts.next();
            let url = parts.next();
            match url {
                Some(url) => {
                    for line in [
                        format!("(simulated) fetching {}...", url),
                        "HTTP/1.1 200 OK".to_string(),
                        "content-type: text/html; charset=utf-8".to_string(),
                        "".to_string(),
                        "<html>... (body truncated) ...</html>".to_string(),
                    ] {
                        push_line_trim(
                            lines,
                            TerminalLine {
                                content: line,
                                line_type: LineType::Output,
                            },
                        );
                    }
                }
                None => {
                    push_line_trim(
                        lines,
                        TerminalLine {
                            content: "Usage: curl <url>".into(),
                            line_type: LineType::Error,
                        },
                    );
                }
            }
        }
        "pwd" => {
            push_line_trim(
                lines,
                TerminalLine {
                    content: demo_dir.into(),
                    line_type: LineType::Output,
                },
            );
        }
        "cat" | "type" => {
            if cmd_lower.split_whitespace().count() < 2 {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "Usage: cat <file>".into(),
                        line_type: LineType::Error,
                    },
                );
            } else {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "(simulated) file contents...".into(),
                        line_type: LineType::Output,
                    },
                );
            }
        }
        "grep" => {
            if cmd_lower.split_whitespace().count() < 3 {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "Usage: grep <pattern> <file>".into(),
                        line_type: LineType::Error,
                    },
                );
            } else {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "(simulated) matching lines...".into(),
                        line_type: LineType::Output,
                    },
                );
            }
        }
        "date" => {
            push_line_trim(
                lines,
                TerminalLine {
                    content: "Fri 02/07/2026".into(),
                    line_type: LineType::Output,
                },
            );
        }
        "ip" | "ipconfig" => {
            for line in [
                "Windows IP Configuration",
                "",
                "Ethernet adapter Ethernet:",
                "   IPv4 Address. . . . . : 192.168.1.100",
                "   Subnet Mask . . . . . : 255.255.255.0",
                "   Default Gateway . . . : 192.168.1.1",
            ] {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: line.into(),
                        line_type: LineType::Output,
                    },
                );
            }
        }
        "ifconfig" => {
            for line in [
                "eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500",
                "        inet 192.168.1.100  netmask 255.255.255.0  broadcast 192.168.1.255",
                "        inet6 fe80::1  prefixlen 64  scopeid 0x20<link>",
                "        ether aa:bb:cc:dd:ee:ff  txqueuelen 1000  (Ethernet)",
            ] {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: line.into(),
                        line_type: LineType::Output,
                    },
                );
            }
        }
        "ping" => {
            if cmd_lower.split_whitespace().count() < 2 {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "Usage: ping <host>".into(),
                        line_type: LineType::Error,
                    },
                );
            } else {
                for line in [
                    "Pinging host with 32 bytes of data:",
                    "Reply from 93.184.216.34: bytes=32 time=12ms TTL=56",
                    "Reply from 93.184.216.34: bytes=32 time=11ms TTL=56",
                    "Reply from 93.184.216.34: bytes=32 time=13ms TTL=56",
                ] {
                    push_line_trim(
                        lines,
                        TerminalLine {
                            content: line.into(),
                            line_type: LineType::Output,
                        },
                    );
                }
            }
        }
        "mkdir" => {
            if cmd_lower.split_whitespace().count() < 2 {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "Usage: mkdir <dir>".into(),
                        line_type: LineType::Error,
                    },
                );
            } else {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "Directory created (simulated)".into(),
                        line_type: LineType::Output,
                    },
                );
            }
        }
        "rm" | "del" => {
            if cmd_lower.split_whitespace().count() < 2 {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "Usage: rm <path>".into(),
                        line_type: LineType::Error,
                    },
                );
            } else {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "Deleted (simulated)".into(),
                        line_type: LineType::Output,
                    },
                );
            }
        }
        "mv" => {
            if cmd_lower.split_whitespace().count() < 3 {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "Usage: mv <source> <dest>".into(),
                        line_type: LineType::Error,
                    },
                );
            } else {
                push_line_trim(
                    lines,
                    TerminalLine {
                        content: "Moved (simulated)".into(),
                        line_type: LineType::Output,
                    },
                );
            }
        }
        "cd" => {
            push_line_trim(
                lines,
                TerminalLine {
                    content: "Directory changed (simulated)".into(),
                    line_type: LineType::System,
                },
            );
        }
        "exit" => {
            push_line_trim(
                lines,
                TerminalLine {
                    content: "Can't exit the web demo! Download the real thing.".into(),
                    line_type: LineType::System,
                },
            );
        }
        _ => {
            push_line_trim(
                lines,
                TerminalLine {
                    content: format!("'{}': command not recognized. Type 'help' for commands.", cmd),
                    line_type: LineType::Error,
                },
            );
        }
    }
}
