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
