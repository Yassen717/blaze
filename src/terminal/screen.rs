//! Screen model types shared between the PTY backend and the Dioxus renderer.
//!
//! The PTY read task converts raw `vt100::Screen` state into a `ScreenSnapshot`
//! and writes it into a `Signal<ScreenSnapshot>`. The renderer then reads the
//! signal and turns it into styled RSX spans. Neither side holds a reference to
//! the other; this module is the only shared contract between them.

// ── Colour ───────────────────────────────────────────────────────────────────

/// A 24-bit sRGB colour value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Format as a CSS `rgb(r, g, b)` string.
    #[inline]
    pub fn to_css(&self) -> String {
        format!("rgb({},{},{})", self.r, self.g, self.b)
    }
}

/// The standard xterm 256-colour palette, indexed 0-255.
///
/// Indices 0-15   → system colours (terminal-theme-dependent; we use common defaults).
/// Indices 16-231 → 6×6×6 colour cube.
/// Indices 232-255 → greyscale ramp.
pub fn xterm256_to_rgb(idx: u8) -> Rgb {
    match idx {
        // ── system colours (0-15) ────────────────────────────────────────────
        0  => Rgb::new(0,   0,   0  ), // Black
        1  => Rgb::new(128, 0,   0  ), // Maroon
        2  => Rgb::new(0,   128, 0  ), // Green
        3  => Rgb::new(128, 128, 0  ), // Olive
        4  => Rgb::new(0,   0,   128), // Navy
        5  => Rgb::new(128, 0,   128), // Purple
        6  => Rgb::new(0,   128, 128), // Teal
        7  => Rgb::new(192, 192, 192), // Silver
        8  => Rgb::new(128, 128, 128), // Grey
        9  => Rgb::new(255, 0,   0  ), // Red
        10 => Rgb::new(0,   255, 0  ), // Lime
        11 => Rgb::new(255, 255, 0  ), // Yellow
        12 => Rgb::new(0,   0,   255), // Blue
        13 => Rgb::new(255, 0,   255), // Fuchsia
        14 => Rgb::new(0,   255, 255), // Aqua
        15 => Rgb::new(255, 255, 255), // White
        // ── 6×6×6 colour cube (16-231) ───────────────────────────────────────
        16..=231 => {
            let i = idx - 16;
            let b = i % 6;
            let g = (i / 6) % 6;
            let r = i / 36;
            let scale = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            Rgb::new(scale(r), scale(g), scale(b))
        }
        // ── greyscale ramp (232-255) ─────────────────────────────────────────
        232..=255 => {
            let v = 8 + (idx - 232) * 10;
            Rgb::new(v, v, v)
        }
    }
}

// ── Cell style ───────────────────────────────────────────────────────────────

/// The complete visual style for a run of terminal cells.
///
/// `fg` / `bg` of `None` means "use the terminal default colour" (i.e. inherit
/// from the `.pty-screen` CSS rule rather than applying an inline colour).
#[derive(Clone, Debug, PartialEq)]
pub struct CellStyle {
    pub fg:        Option<Rgb>,
    pub bg:        Option<Rgb>,
    pub bold:      bool,
    pub italic:    bool,
    pub underline: bool,
    pub dim:       bool,
    pub blink:     bool,
    pub reverse:   bool,
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            fg:        None,
            bg:        None,
            bold:      false,
            italic:    false,
            underline: false,
            dim:       false,
            blink:     false,
            reverse:   false,
        }
    }
}

impl CellStyle {
    /// Build an inline CSS `style` attribute string for this cell style.
    ///
    /// Returns an empty string when the style is default (avoids injecting
    /// `style=""` onto every span).
    pub fn to_inline_css(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if let Some(fg) = &self.fg {
            parts.push(format!("color:{}", fg.to_css()));
        }
        if let Some(bg) = &self.bg {
            parts.push(format!("background-color:{}", bg.to_css()));
        }
        if self.bold {
            parts.push("font-weight:bold".into());
        }
        if self.italic {
            parts.push("font-style:italic".into());
        }
        if self.underline {
            parts.push("text-decoration:underline".into());
        }
        if self.dim {
            parts.push("opacity:0.5".into());
        }
        if self.blink {
            // CSS animation is defined in main.css as `.pty-blink`.
            // We add it via class rather than inline style; this flag is a
            // hint to the renderer.
        }

        parts.join(";")
    }

    /// Returns `true` when this style is entirely default (no inline CSS needed).
    #[inline]
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

// ── Styled run ───────────────────────────────────────────────────────────────

/// A contiguous sequence of characters that all share the same `CellStyle`.
///
/// The renderer groups adjacent cells with identical styles into a single
/// `<span>` to minimise DOM node count.
#[derive(Clone, Debug, PartialEq)]
pub struct StyledRun {
    pub text:  String,
    pub style: CellStyle,
}

// ── Screen row ───────────────────────────────────────────────────────────────

/// One horizontal row of the terminal screen, expressed as an ordered list of
/// styled runs.
///
/// A row with no runs (or a single run of spaces) is treated as blank. The
/// renderer emits a non-breaking space so the row still occupies line-height.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenRow {
    pub runs: Vec<StyledRun>,
}

impl ScreenRow {
    /// Returns `true` when the row contains no visible text.
    pub fn is_blank(&self) -> bool {
        self.runs.iter().all(|r| r.text.trim().is_empty())
    }
}

// ── Screen snapshot ──────────────────────────────────────────────────────────

/// An owned, cheaply cloneable snapshot of the full terminal screen state.
///
/// Produced by the PTY read task after each batch of output bytes; consumed
/// by the Dioxus renderer. Because `Signal` re-renders only when the value
/// changes (`PartialEq`), keeping this type small and well-diffable matters.
///
/// # Revision counter
/// `revision` is incremented every time a new snapshot is produced. The
/// renderer can use it as a fast inequality check before doing a full
/// structural diff.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenSnapshot {
    /// Rendered rows, top to bottom. Length == `lines`.
    pub rows: Vec<ScreenRow>,

    /// Cursor position (0-based row index into `rows`).
    pub cursor_row: usize,

    /// Cursor position (0-based column index within the row).
    pub cursor_col: usize,

    /// Terminal window title set by the running program via OSC 0 / OSC 2
    /// escape sequences (e.g. `\x1b]0;vim — main.rs\x07`).
    pub title: String,

    /// Terminal width in character columns.
    pub cols: u16,

    /// Terminal height in character rows (the visible screen, not scrollback).
    pub lines: u16,

    /// Monotonically increasing counter. Bumped on every snapshot production.
    /// Use this for a fast "has anything changed?" check.
    pub revision: u64,
}

impl Default for ScreenSnapshot {
    fn default() -> Self {
        let cols:  u16 = 80;
        let lines: u16 = 24;
        Self {
            rows: (0..lines)
                .map(|_| ScreenRow { runs: Vec::new() })
                .collect(),
            cursor_row: 0,
            cursor_col: 0,
            title:    "⚡ Blaze Terminal".to_string(),
            cols,
            lines,
            revision: 0,
        }
    }
}

impl ScreenSnapshot {
    /// Convenience constructor used in tests — creates a blank screen of the
    /// given dimensions.
    #[cfg(test)]
    pub fn blank(cols: u16, lines: u16) -> Self {
        Self {
            rows: (0..lines).map(|_| ScreenRow { runs: Vec::new() }).collect(),
            cursor_row: 0,
            cursor_col: 0,
            title: String::new(),
            cols,
            lines,
            revision: 0,
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_snapshot_has_correct_dimensions() {
        let snap = ScreenSnapshot::default();
        assert_eq!(snap.cols, 80);
        assert_eq!(snap.lines, 24);
        assert_eq!(snap.rows.len(), 24);
    }

    #[test]
    fn cell_style_default_produces_empty_css() {
        let style = CellStyle::default();
        assert!(style.to_inline_css().is_empty());
        assert!(style.is_default());
    }

    #[test]
    fn cell_style_fg_color_appears_in_css() {
        let style = CellStyle {
            fg: Some(Rgb::new(255, 128, 0)),
            ..Default::default()
        };
        let css = style.to_inline_css();
        assert!(css.contains("color:rgb(255,128,0)"), "got: {css}");
    }

    #[test]
    fn cell_style_bold_appears_in_css() {
        let style = CellStyle { bold: true, ..Default::default() };
        assert!(style.to_inline_css().contains("font-weight:bold"));
    }

    #[test]
    fn xterm256_black_and_white() {
        assert_eq!(xterm256_to_rgb(0),  Rgb::new(0, 0, 0));
        assert_eq!(xterm256_to_rgb(15), Rgb::new(255, 255, 255));
    }

    #[test]
    fn xterm256_greyscale_ramp_bounds() {
        // Index 232 → darkest grey (8, 8, 8)
        let dark = xterm256_to_rgb(232);
        assert_eq!(dark, Rgb::new(8, 8, 8));
        // Index 255 → lightest grey (238, 238, 238)
        let light = xterm256_to_rgb(255);
        assert_eq!(light, Rgb::new(238, 238, 238));
    }

    #[test]
    fn xterm256_colour_cube_midpoint() {
        // Index 16 is the first cube entry: r=0, g=0, b=0 (same as black but
        // separate from the system palette).
        assert_eq!(xterm256_to_rgb(16), Rgb::new(0, 0, 0));
        // Index 231 is the last cube entry: r=5, g=5, b=5 → (255, 255, 255).
        assert_eq!(xterm256_to_rgb(231), Rgb::new(255, 255, 255));
    }

    #[test]
    fn screen_row_blank_detection() {
        let blank = ScreenRow { runs: Vec::new() };
        assert!(blank.is_blank());

        let spaces = ScreenRow {
            runs: vec![StyledRun {
                text:  "   ".to_string(),
                style: CellStyle::default(),
            }],
        };
        assert!(spaces.is_blank());

        let text = ScreenRow {
            runs: vec![StyledRun {
                text:  "hello".to_string(),
                style: CellStyle::default(),
            }],
        };
        assert!(!text.is_blank());
    }

    #[test]
    fn rgb_to_css_format() {
        let c = Rgb::new(10, 20, 30);
        assert_eq!(c.to_css(), "rgb(10,20,30)");
    }
}
