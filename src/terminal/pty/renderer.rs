//! Dioxus component that turns a [`ScreenSnapshot`] into HTML.
//!
//! # Rendering model
//!
//! ```text
//! ScreenSnapshot
//!   └── Vec<ScreenRow>  (one per visible terminal line)
//!         └── Vec<StyledRun>  (adjacent cells grouped by identical style)
//!               ├── text:      String          → text content of <span>
//!               ├── style:     CellStyle       → inline CSS (color, bold, …)
//!               └── is_cursor: bool            → adds .pty-cursor class
//! ```
//!
//! Each `ScreenRow` becomes a `<div class="pty-row">`.
//! Each `StyledRun` becomes a `<span>` with optional inline `style` and
//! `class` attributes.
//! Blank rows (all whitespace) render a single `&nbsp;` so they retain their
//! line-height without emitting unnecessary DOM nodes.
//!
//! # Re-render budget
//!
//! `PtyScreen` re-renders only when the signal value changes, which Dioxus
//! determines via `PartialEq` on `ScreenSnapshot`.  The `revision` counter on
//! the snapshot acts as a fast inequality check — two snapshots with the same
//! revision are identical and the component is skipped entirely.

use dioxus::prelude::*;

use crate::terminal::screen::{ScreenRow, ScreenSnapshot, StyledRun};

// ── Main component ────────────────────────────────────────────────────────────

/// Renders a [`ScreenSnapshot`] as a styled terminal screen.
///
/// Mount this inside the desktop terminal container:
///
/// ```rust,ignore
/// rsx! {
///     div { class: "terminal-container terminal-fullscreen",
///         // … header …
///         PtyScreen { snapshot: screen_signal.into() }
///     }
/// }
/// ```
///
/// The component owns one clone of the snapshot per render cycle (obtained by
/// calling the signal) and releases it before returning the element tree.
#[component]
pub fn PtyScreen(snapshot: ReadSignal<ScreenSnapshot>) -> Element {
    // Clone the snapshot out of the signal so we own the data for the lifetime
    // of this render call.  Re-renders are gated by PartialEq so this clone
    // only happens when the PTY backend produces a new screen state.
    let ScreenSnapshot { rows, .. } = snapshot();

    rsx! {
        div { id: "pty-screen", class: "pty-screen",
            for (row_idx, row) in rows.into_iter().enumerate() {
                div { key: "{row_idx}", class: "pty-row",
                    { render_row(row) }
                }
            }
        }
    }
}

// ── Row renderer ──────────────────────────────────────────────────────────────

/// Render the contents of a single terminal row.
///
/// Blank rows (no visible text) emit a non-breaking space so the `div` retains
/// its `min-height` without cluttering the DOM with empty spans.
fn render_row(row: ScreenRow) -> Element {
    if row.is_blank() {
        // U+00A0 NON-BREAKING SPACE — preserves the line box height.
        return rsx! { "\u{00A0}" };
    }

    rsx! {
        for (run_idx, run) in row.runs.into_iter().enumerate() {
            { render_run(run_idx, run) }
        }
    }
}

// ── Run renderer ──────────────────────────────────────────────────────────────

/// Render a single styled run as a `<span>`.
///
/// * The inline `style` attribute carries colours and text decorations.
/// * The `class` attribute carries the cursor-blink and blink animations.
/// * When both would be empty (default, non-cursor run) the span is still
///   emitted — Dioxus diffs it cheaply and it keeps the tree shape stable.
fn render_run(run_idx: usize, run: StyledRun) -> Element {
    let css   = run.style.to_inline_css();
    let class = run_class(run.is_cursor, run.style.blink);

    rsx! {
        span {
            key:   "{run_idx}",
            class: "{class}",
            style: "{css}",
            "{run.text}"
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return the CSS class string for a run.
///
/// | `is_cursor` | `blink` | result                   |
/// |-------------|---------|--------------------------|
/// | false       | false   | `""`                     |
/// | false       | true    | `"pty-blink"`            |
/// | true        | false   | `"pty-cursor"`           |
/// | true        | true    | `"pty-cursor pty-blink"` |
#[inline]
pub fn run_class(is_cursor: bool, blink: bool) -> &'static str {
    match (is_cursor, blink) {
        (true,  true)  => "pty-cursor pty-blink",
        (true,  false) => "pty-cursor",
        (false, true)  => "pty-blink",
        (false, false) => "",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── run_class ─────────────────────────────────────────────────────────────

    #[test]
    fn run_class_no_cursor_no_blink_is_empty() {
        assert_eq!(run_class(false, false), "");
    }

    #[test]
    fn run_class_cursor_only() {
        assert_eq!(run_class(true, false), "pty-cursor");
    }

    #[test]
    fn run_class_blink_only() {
        assert_eq!(run_class(false, true), "pty-blink");
    }

    #[test]
    fn run_class_cursor_and_blink() {
        assert_eq!(run_class(true, true), "pty-cursor pty-blink");
    }

    #[test]
    fn run_class_returns_static_str() {
        // Verify the returned slice has 'static lifetime by binding to a
        // variable with an explicit 'static annotation.
        let s: &'static str = run_class(true, false);
        assert!(!s.is_empty());
    }
}
