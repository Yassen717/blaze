//! PTY backend for the Blaze desktop terminal.
//!
//! This module is intentionally Dioxus-agnostic. It communicates with the UI
//! layer exclusively through plain Rust channels; all Dioxus Signal wiring
//! lives in `terminal/components.rs`.
//!
//! # Thread model
//!
//! ```text
//!  ┌──────────────┐   raw bytes    ┌───────────────────────────────────────┐
//!  │  read_thread │ ─────────────► │            process_thread             │
//!  │  (blocking   │  sync_channel  │  • feeds PtyParser (vt100 wrapper)    │
//!  │   PTY read)  │                │  • drains resize events (non-blocking) │
//!  └──────────────┘                │  • emits PtyUpdate via tokio channel   │
//!                                  └───────────────────────────────────────┘
//!  ┌──────────────┐   Vec<u8>
//!  │  write_thread│ ◄─────────────  UI (key events, paste)
//!  │  (Box<dyn    │  std mpsc
//!  │   Write+Send>│
//!  └──────────────┘
//! ```
//!
//! Resize events are sent on a separate `std::sync::mpsc` channel and drained
//! by the process thread between each batch of PTY bytes, so resize is applied
//! within ≤ 50 ms even when the shell is idle (the process thread wakes on a
//! 50 ms `recv_timeout`).

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::time::Duration;

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

pub mod keys;
pub mod renderer;

use crate::terminal::screen::{
    CellStyle, Rgb, ScreenRow, ScreenSnapshot, StyledRun, xterm256_to_rgb,
};

// ── Public error type ─────────────────────────────────────────────────────────

pub type PtyError = Box<dyn std::error::Error + Send + Sync>;

// ── PtyParser ─────────────────────────────────────────────────────────────────

/// Thin wrapper around `vt100::Parser` that adds a `set_size` method.
///
/// `vt100` 0.15.x does not expose `Parser::screen_mut()`, so there is no
/// direct way to call `Screen::set_size`. The workaround is to replace the
/// parser with a fresh one of the new dimensions; the shell receives a resize
/// notification (SIGWINCH on Unix, `WINDOW_BUFFER_SIZE_EVENT` on Windows) and
/// immediately redraws, so the new parser correctly renders the refreshed
/// content within one update cycle.
struct PtyParser {
    inner: vt100::Parser,
    scrollback: usize,
}

impl PtyParser {
    fn new(rows: u16, cols: u16, scrollback: usize) -> Self {
        Self {
            inner: vt100::Parser::new(rows, cols, scrollback),
            scrollback,
        }
    }

    fn process(&mut self, bytes: &[u8]) {
        self.inner.process(bytes);
    }

    fn screen(&self) -> &vt100::Screen {
        self.inner.screen()
    }

    /// Reset the parser to new dimensions.
    ///
    /// The shell will receive a resize signal and redraw the full screen; the
    /// replacement parser will handle that redrawn output correctly.
    fn set_size(&mut self, rows: u16, cols: u16) {
        self.inner = vt100::Parser::new(rows, cols, self.scrollback);
    }
}

// ── PtyUpdate ─────────────────────────────────────────────────────────────────

/// A single update emitted by the PTY backend to the UI layer.
///
/// Produced after every batch of bytes from the shell; the Dioxus component
/// receives these via the `tokio::sync::mpsc` receiver returned by
/// [`PtySession::spawn`] and writes them into its `Signal`s.
#[derive(Debug)]
pub struct PtyUpdate {
    /// Full snapshot of the terminal screen after parsing the latest bytes.
    pub snapshot: ScreenSnapshot,
    /// OSC window title set by the running program. Empty if not changed.
    pub title: String,
}

// ── PtySession ────────────────────────────────────────────────────────────────

/// Handle to a running shell inside a PTY.
///
/// Obtained from [`PtySession::spawn`]. The three background threads keep
/// running independently until the child shell exits; they then shut down
/// cleanly by themselves.
///
/// Dropping this struct closes the write and resize channels, which signals
/// the write/process threads to exit on their next iteration.
pub struct PtySession {
    /// Send raw bytes to the shell (keyboard input, paste data, Ctrl-sequences).
    pub write_tx: mpsc::Sender<Vec<u8>>,
    /// Resize the PTY. Payload is `(cols, rows)`.
    pub resize_tx: mpsc::Sender<(u16, u16)>,
}

impl PtySession {
    /// Spawn a shell inside a PTY and start the three background threads.
    ///
    /// Returns `(session_handle, update_receiver)`.
    ///
    /// The caller **must** drive `update_rx` — typically inside a
    /// `dioxus::prelude::spawn(async move { … })` block — to route snapshots
    /// into Dioxus `Signal`s:
    ///
    /// ```rust,ignore
    /// let (session, mut update_rx) = PtySession::spawn(80, 24)?;
    /// spawn(async move {
    ///     while let Some(update) = update_rx.recv().await {
    ///         *screen_signal.write() = update.snapshot;
    ///         if !update.title.is_empty() {
    ///             *title_signal.write() = update.title;
    ///         }
    ///     }
    /// });
    /// ```
    ///
    /// # Shell selection
    ///
    /// | Platform | Default          | Override        |
    /// |----------|------------------|-----------------|
    /// | Windows  | `powershell.exe` | `BLAZE_SHELL`   |
    /// | Unix     | `$SHELL`         | `BLAZE_SHELL`   |
    ///
    /// # Errors
    ///
    /// Returns an error if the PTY cannot be opened or the shell binary cannot
    /// be spawned (e.g. not found on `PATH`).
    pub fn spawn(
        cols: u16,
        rows: u16,
    ) -> Result<(Self, tokio::sync::mpsc::UnboundedReceiver<PtyUpdate>), PtyError> {
        // ── 1. Open the PTY pair ──────────────────────────────────────────────
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // ── 2. Spawn the shell on the slave side ──────────────────────────────
        let shell = resolve_shell();
        let cmd = CommandBuilder::new(&shell);
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell '{}': {}", shell, e))?;

        // Slave is no longer needed after the child has been spawned.
        drop(pair.slave);

        // ── 3. Split the master ───────────────────────────────────────────────
        //
        // • reader  — given to the read thread for blocking PTY reads.
        // • writer  — given to the write thread; obtained via `take_writer()`
        //             which returns a `Box<dyn Write + Send>` independently of
        //             the master (MasterPty does not extend Write in 0.8.x).
        // • master  — kept in an Arc<Mutex<…>> used only for resize() calls
        //             from the process thread.
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        let master: Arc<Mutex<Box<dyn MasterPty + Send>>> =
            Arc::new(Mutex::new(pair.master));

        // ── 4. Channels ───────────────────────────────────────────────────────

        // UI → write_thread: keyboard input and paste bytes.
        let (write_tx, write_rx) = mpsc::channel::<Vec<u8>>();

        // UI → process_thread: terminal resize events (cols, rows).
        let (resize_tx, resize_rx) = mpsc::channel::<(u16, u16)>();

        // read_thread → process_thread: raw PTY output bytes.
        // Bounded to provide back-pressure if the parser falls behind.
        let (raw_tx, raw_rx) = mpsc::sync_channel::<Vec<u8>>(256);

        // process_thread → Dioxus component: parsed screen snapshots.
        // `UnboundedSender` is `Send + Sync` so it is safe to use from a
        // `std::thread`. The corresponding `UnboundedReceiver` is returned to
        // the caller for use inside a Dioxus async spawn block.
        let (update_tx, update_rx) = tokio::sync::mpsc::unbounded_channel::<PtyUpdate>();

        // ── 5. Write thread ───────────────────────────────────────────────────
        std::thread::Builder::new()
            .name("blaze-pty-write".into())
            .spawn(move || write_loop(writer, write_rx))?;

        // ── 6. Read thread ────────────────────────────────────────────────────
        std::thread::Builder::new()
            .name("blaze-pty-read".into())
            .spawn(move || read_loop(reader, raw_tx))?;

        // ── 7. Process thread ─────────────────────────────────────────────────
        //
        // Owns the vt100 parser, drains resize events, emits PtyUpdates, and
        // answers host-directed queries (e.g. DSR cursor reports) via a clone
        // of the write channel. Also waits for the child process to exit so
        // it can be reaped cleanly.
        let process_write_tx = write_tx.clone();
        std::thread::Builder::new()
            .name("blaze-pty-process".into())
            .spawn(move || {
                process_loop(
                    raw_rx,
                    resize_rx,
                    master,
                    rows,
                    cols,
                    update_tx,
                    process_write_tx,
                );
                // Reap the child process after the session ends to avoid zombies.
                let mut child = child;
                let _ = child.wait();
            })?;

        Ok((PtySession { write_tx, resize_tx }, update_rx))
    }
}

// ── Shell resolution ──────────────────────────────────────────────────────────

/// Returns the shell binary path to use for this session.
///
/// Priority: `BLAZE_SHELL` env var → platform default.
fn resolve_shell() -> String {
    if let Ok(s) = std::env::var("BLAZE_SHELL") {
        if !s.is_empty() {
            return s;
        }
    }

    #[cfg(target_os = "windows")]
    {
        "powershell.exe".into()
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Respect the user's configured login shell.
        if let Ok(s) = std::env::var("SHELL") {
            if !s.is_empty() {
                return s;
            }
        }
        // Guaranteed to exist on every POSIX system.
        "bash".into()
    }
}

// ── Thread: read ──────────────────────────────────────────────────────────────

/// Reads raw bytes from the PTY master in a tight blocking loop and forwards
/// them to the process thread.
///
/// Exits naturally when the PTY master returns EOF (child process exited) or
/// an I/O error occurs, closing `tx` which signals the process thread to exit.
fn read_loop(mut reader: Box<dyn Read + Send>, tx: mpsc::SyncSender<Vec<u8>>) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break, // EOF — child exited
            Ok(n) => {
                if tx.send(buf[..n].to_vec()).is_err() {
                    break; // process thread dropped the receiver
                }
            }
            Err(_) => break,
        }
    }
}

// ── Thread: write ─────────────────────────────────────────────────────────────

/// Drains the write channel and forwards each payload to the PTY via the
/// `Box<dyn Write + Send>` obtained from `MasterPty::take_writer()`.
fn write_loop(mut writer: Box<dyn Write + Send>, rx: mpsc::Receiver<Vec<u8>>) {
    for bytes in rx {
        if writer.write_all(&bytes).is_err() {
            break;
        }
        let _ = writer.flush();
    }
}

// ── Thread: process ───────────────────────────────────────────────────────────

/// Parses PTY output, handles resize events, and emits [`PtyUpdate`]s.
///
/// # Resize responsiveness
///
/// The loop uses `recv_timeout(50 ms)` on the raw-bytes channel so that resize
/// events (checked non-blocking before each read attempt) are applied within
/// ≤ 50 ms even when the shell produces no output.
fn process_loop(
    raw_rx: mpsc::Receiver<Vec<u8>>,
    resize_rx: mpsc::Receiver<(u16, u16)>,
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    init_rows: u16,
    init_cols: u16,
    update_tx: tokio::sync::mpsc::UnboundedSender<PtyUpdate>,
    write_tx: mpsc::Sender<Vec<u8>>,
) {
    let mut parser = PtyParser::new(init_rows, init_cols, 1000);
    let mut revision: u64 = 0;
    // Last bytes of the previous batch, kept so an escape sequence split
    // across a batch boundary is still recognised.
    let mut tail: Vec<u8> = Vec::new();

    loop {
        // ── (a) Drain all pending resize events first (non-blocking) ──────────
        loop {
            match resize_rx.try_recv() {
                Ok((new_cols, new_rows)) => {
                    // Resize the kernel-level TTY window.
                    if let Ok(m) = master.lock() {
                        let _ = m.resize(PtySize {
                            rows: new_rows,
                            cols: new_cols,
                            pixel_width: 0,
                            pixel_height: 0,
                        });
                    }
                    // Reset the vt100 parser to the new dimensions.
                    // The shell will redraw completely after receiving the resize signal.
                    parser.set_size(new_rows, new_cols);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => return,
            }
        }

        // ── (b) Wait for the next batch of raw bytes (with a short timeout) ───
        let first = match raw_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(b) => b,
            Err(mpsc::RecvTimeoutError::Timeout) => continue, // re-check resize
            Err(mpsc::RecvTimeoutError::Disconnected) => break, // read thread exited
        };

        // Coalesce any additional bytes that arrived while we were unblocked.
        let mut batch = first;
        loop {
            match raw_rx.try_recv() {
                Ok(more) => batch.extend(more),
                Err(_) => break,
            }
        }

        // ── (c) Feed the bytes into the VT100 parser ──────────────────────────
        parser.process(&batch);

        // ── (c2) Answer terminal-directed queries in this batch ───────────────
        // ConPTY (INHERIT_CURSOR) and some applications send DSR queries and
        // wait for a reply on the input pipe; never answering can stall them.
        let mut hay = tail.clone();
        hay.extend_from_slice(&batch);
        let (cursor_reports, status_requests) = count_dsr_queries(&hay);
        if cursor_reports > 0 || status_requests > 0 {
            let (cur_row, cur_col) = parser.screen().cursor_position();
            for _ in 0..cursor_reports {
                // CPR — 1-based row;column.
                let _ = write_tx.send(
                    format!("\x1b[{};{}R", cur_row + 1, cur_col + 1).into_bytes(),
                );
            }
            for _ in 0..status_requests {
                // "Ready, no malfunctions detected."
                let _ = write_tx.send(b"\x1b[0n".to_vec());
            }
        }
        tail = batch[batch.len().saturating_sub(3)..].to_vec();

        // ── (d) Build a snapshot and push it to the Dioxus layer ─────────────
        revision += 1;
        let snapshot = build_snapshot(parser.screen(), revision);
        let title = parser.screen().title().to_string();

        if update_tx.send(PtyUpdate { snapshot, title }).is_err() {
            break; // Dioxus component dropped the receiver (window closed)
        }
    }
}

// ── DSR query detection ───────────────────────────────────────────────────────

/// Count Device Status Report queries in a raw output batch.
///
/// Returns `(cursor_position_reports, status_requests)` for `\x1b[6n` and
/// `\x1b[5n` respectively. The caller should prepend the tail of the previous
/// batch so sequences split across batches are still detected.
fn count_dsr_queries(batch: &[u8]) -> (u32, u32) {
    let mut cursor_reports = 0u32;
    let mut status_requests = 0u32;
    for w in batch.windows(4) {
        match w {
            b"\x1b[6n" => cursor_reports += 1,
            b"\x1b[5n" => status_requests += 1,
            _ => {}
        }
    }
    (cursor_reports, status_requests)
}

// ── Snapshot builder ──────────────────────────────────────────────────────────

/// Convert a `vt100::Screen` reference into an owned [`ScreenSnapshot`].
fn build_snapshot(screen: &vt100::Screen, revision: u64) -> ScreenSnapshot {
    let (rows, cols) = screen.size();
    let (cur_row, cur_col) = screen.cursor_position();
    let cursor_visible = !screen.hide_cursor();

    ScreenSnapshot {
        rows: (0..rows)
            .map(|r| build_row(screen, r, cols, cur_row, cur_col, cursor_visible))
            .collect(),
        cursor_row: cur_row as usize,
        cursor_col: cur_col as usize,
        cursor_visible,
        title: screen.title().to_string(),
        cols,
        lines: rows,
        revision,
        bracketed_paste: screen.bracketed_paste(),
    }
}

/// Build a single [`ScreenRow`] by iterating every cell and grouping adjacent
/// cells that share the same [`CellStyle`] into [`StyledRun`]s.
fn build_row(
    screen: &vt100::Screen,
    row_idx: u16,
    cols: u16,
    cur_row: u16,
    cur_col: u16,
    cursor_visible: bool,
) -> ScreenRow {
    let mut runs = Vec::<StyledRun>::new();
    let mut text = String::new();
    let mut current_style = CellStyle::default();
    // Tracks whether the run currently being accumulated contains the cursor
    // cell. Reset to false each time a completed run is flushed.
    let mut current_is_cursor = false;

    for col in 0..cols {
        let is_cursor_cell = cursor_visible && row_idx == cur_row && col == cur_col;

        let (ch, cell_style) = match screen.cell(row_idx, col) {
            Some(cell) => {
                let s = cell.contents();
                // Wide-character continuation cells have empty contents; treat
                // them as a space to preserve column alignment.
                let ch = if s.is_empty() { " ".to_string() } else { s.to_string() };
                (ch, cell_to_style(cell, is_cursor_cell))
            }
            None => (" ".to_string(), CellStyle::default()),
        };

        if cell_style != current_style {
            if !text.is_empty() {
                runs.push(StyledRun {
                    text: std::mem::take(&mut text),
                    style: current_style.clone(),
                    is_cursor: current_is_cursor,
                });
                current_is_cursor = false;
            }
            current_style = cell_style;
        }

        // Mark the accumulating run as the cursor run when this cell is the
        // cursor. Because cell_to_style applies unique colours for the cursor
        // cell, it will almost always be isolated in its own single-char run.
        if is_cursor_cell {
            current_is_cursor = true;
        }

        text.push_str(&ch);
    }

    if !text.is_empty() {
        runs.push(StyledRun {
            text,
            style: current_style,
            is_cursor: current_is_cursor,
        });
    }

    ScreenRow { runs }
}

// ── Style conversion ──────────────────────────────────────────────────────────

/// Convert a [`vt100::Cell`] into a [`CellStyle`].
///
/// Uses the public methods directly on `Cell` (e.g. `cell.bold()`) since
/// `Cell::attrs()` is `pub(crate)` in vt100 0.15.x.
///
/// Reverse-video and cursor rendering both swap fg/bg; cursor additionally
/// falls back to Blaze's theme colours when no explicit colour is set.
fn cell_to_style(cell: &vt100::Cell, is_cursor: bool) -> CellStyle {
    let mut fg = vt100_color(cell.fgcolor());
    let mut bg = vt100_color(cell.bgcolor());

    if cell.inverse() || is_cursor {
        std::mem::swap(&mut fg, &mut bg);
        if is_cursor {
            // Fall back to Blaze theme colours when the cell has no explicit colour.
            if bg.is_none() {
                bg = Some(Rgb::new(255, 138, 76)); // #ff8a4c — Blaze cursor accent
            }
            if fg.is_none() {
                fg = Some(Rgb::new(11, 14, 19)); // terminal background
            }
        }
    }

    CellStyle {
        fg,
        bg,
        bold: cell.bold(),
        italic: cell.italic(),
        underline: cell.underline(),
        // `dim` is not exposed as a dedicated public method in vt100 0.15;
        // it can be added in a future pass once vt100 exposes it.
        dim: false,
        // `blink` is handled via the `.pty-blink` CSS animation class rather
        // than inline style; the flag is left false here for now.
        blink: false,
        // `reverse` has already been applied above by swapping fg/bg.
        reverse: false,
    }
}

/// Convert a [`vt100::Color`] to `Option<Rgb>`.
///
/// `Color::Default` maps to `None` so the renderer inherits the terminal's
/// default foreground/background from CSS rather than forcing an inline style.
fn vt100_color(c: vt100::Color) -> Option<Rgb> {
    match c {
        vt100::Color::Default => None,
        vt100::Color::Idx(i) => Some(xterm256_to_rgb(i)),
        vt100::Color::Rgb(r, g, b) => Some(Rgb::new(r, g, b)),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    // ── Shell resolution ──────────────────────────────────────────────────────

    /// Serialises tests that mutate the shared `BLAZE_SHELL` env var —
    /// parallel test threads otherwise race `set_var`/`remove_var`.
    static SHELL_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn resolve_shell_respects_blaze_shell_env() {
        let _g = SHELL_ENV_LOCK.lock().unwrap();
        std::env::set_var("BLAZE_SHELL", "my-custom-shell");
        assert_eq!(resolve_shell(), "my-custom-shell");
        std::env::remove_var("BLAZE_SHELL");
    }

    #[test]
    fn resolve_shell_ignores_empty_blaze_shell() {
        let _g = SHELL_ENV_LOCK.lock().unwrap();
        std::env::set_var("BLAZE_SHELL", "");
        let shell = resolve_shell();
        // Should fall through to the platform default, not return empty.
        assert!(!shell.is_empty());
        std::env::remove_var("BLAZE_SHELL");
    }

    #[test]
    fn resolve_shell_returns_non_empty_by_default() {
        let _g = SHELL_ENV_LOCK.lock().unwrap();
        std::env::remove_var("BLAZE_SHELL");
        assert!(!resolve_shell().is_empty());
    }

    // ── vt100 colour conversion ───────────────────────────────────────────────

    #[test]
    fn vt100_color_default_maps_to_none() {
        assert_eq!(vt100_color(vt100::Color::Default), None);
    }

    #[test]
    fn vt100_color_idx_maps_to_rgb() {
        assert_eq!(vt100_color(vt100::Color::Idx(0)), Some(xterm256_to_rgb(0)));
        assert_eq!(
            vt100_color(vt100::Color::Idx(196)),
            Some(Rgb::new(255, 0, 0))
        );
    }

    #[test]
    fn vt100_color_rgb_round_trips() {
        assert_eq!(
            vt100_color(vt100::Color::Rgb(10, 20, 30)),
            Some(Rgb::new(10, 20, 30))
        );
    }

    // ── PtyParser wrapper ─────────────────────────────────────────────────────

    #[test]
    fn pty_parser_initial_dimensions_are_correct() {
        let parser = PtyParser::new(24, 80, 0);
        let (rows, cols) = parser.screen().size();
        assert_eq!(rows, 24);
        assert_eq!(cols, 80);
    }

    #[test]
    fn pty_parser_set_size_updates_dimensions() {
        let mut parser = PtyParser::new(24, 80, 0);
        parser.set_size(10, 40);
        let (rows, cols) = parser.screen().size();
        assert_eq!(rows, 10);
        assert_eq!(cols, 40);
    }

    // ── Snapshot builder ──────────────────────────────────────────────────────

    #[test]
    fn build_snapshot_dimensions_match_parser() {
        let mut parser = PtyParser::new(24, 80, 0);
        parser.process(b"hello");
        let snap = build_snapshot(parser.screen(), 1);
        assert_eq!(snap.cols, 80);
        assert_eq!(snap.lines, 24);
        assert_eq!(snap.rows.len(), 24);
        assert_eq!(snap.revision, 1);
    }

    #[test]
    fn build_snapshot_revision_increments() {
        let parser = PtyParser::new(5, 10, 0);
        let snap1 = build_snapshot(parser.screen(), 1);
        let snap2 = build_snapshot(parser.screen(), 2);
        assert_eq!(snap1.revision, 1);
        assert_eq!(snap2.revision, 2);
    }

    #[test]
    fn build_snapshot_row_count_equals_lines() {
        let parser = PtyParser::new(10, 40, 0);
        let snap = build_snapshot(parser.screen(), 1);
        assert_eq!(snap.rows.len(), snap.lines as usize);
    }

    // ── DSR query detection ───────────────────────────────────────────────────

    #[test]
    fn count_dsr_queries_detects_cursor_report() {
        assert_eq!(count_dsr_queries(b"abc\x1b[6ndef"), (1, 0));
    }

    #[test]
    fn count_dsr_queries_detects_status_request() {
        assert_eq!(count_dsr_queries(b"\x1b[5n"), (0, 1));
    }

    #[test]
    fn count_dsr_queries_counts_multiple() {
        assert_eq!(count_dsr_queries(b"\x1b[6n\x1b[6n\x1b[5n"), (2, 1));
    }

    #[test]
    fn count_dsr_queries_ignores_similar_sequences() {
        assert_eq!(count_dsr_queries(b"\x1b[6h\x1b[16n\x1b[m"), (0, 0));
    }

    #[test]
    fn count_dsr_queries_empty_batch() {
        assert_eq!(count_dsr_queries(b""), (0, 0));
    }

    // ── Cursor visibility ─────────────────────────────────────────────────────

    #[test]
    fn visible_cursor_produces_cursor_run() {
        let mut parser = PtyParser::new(24, 80, 0);
        parser.process(b"PS C:\\> \x1b[?25h");
        let snap = build_snapshot(parser.screen(), 1);
        assert!(snap.cursor_visible);
        assert!(snap
            .rows
            .iter()
            .flat_map(|r| r.runs.iter())
            .any(|r| r.is_cursor));
    }

    #[test]
    fn hidden_cursor_produces_no_cursor_run() {
        let mut parser = PtyParser::new(24, 80, 0);
        parser.process(b"PS C:\\> \x1b[?25l");
        let snap = build_snapshot(parser.screen(), 1);
        assert!(!snap.cursor_visible);
        assert!(!snap
            .rows
            .iter()
            .flat_map(|r| r.runs.iter())
            .any(|r| r.is_cursor));
    }

    // ── PTY integration ───────────────────────────────────────────────────────

    /// Smoke-test: the PTY spawns successfully and produces at least one screen
    /// update within a reasonable timeout.
    ///
    /// PowerShell on Windows can take several seconds to start, so the timeout
    /// is intentionally generous (15 s). The test exits as soon as the first
    /// update arrives.
    #[test]
    fn pty_session_spawns_and_produces_output() {
        let (session, mut rx) =
            PtySession::spawn(80, 24).expect("PTY should spawn without error");

        // Send a command that produces output on every supported shell.
        let cmd: Vec<u8> = if cfg!(target_os = "windows") {
            b"Write-Output hello\r\n".to_vec()
        } else {
            b"echo hello\n".to_vec()
        };
        session.write_tx.send(cmd).expect("write_tx must be open");

        // Poll with try_recv so we do not need a Tokio runtime in the test.
        let timeout = Duration::from_secs(15);
        let start = Instant::now();
        let mut received = false;

        while start.elapsed() < timeout {
            match rx.try_recv() {
                Ok(_) => {
                    received = true;
                    break;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    panic!("update channel closed before any output was received");
                }
            }
        }

        assert!(received, "expected at least one PtyUpdate within {timeout:?}");
    }

    /// Sending multiple rapid resize events must not panic or deadlock.
    #[test]
    fn resize_events_are_applied_without_panic() {
        let (session, _rx) = PtySession::spawn(80, 24).expect("PTY should spawn");

        for &cols in &[120u16, 80, 200, 40, 80] {
            session
                .resize_tx
                .send((cols, 24))
                .expect("resize_tx must be open");
        }

        // Give the process thread time to consume all pending resize events.
        std::thread::sleep(Duration::from_millis(500));
        // Reaching here means no panic or deadlock occurred.
    }
}
