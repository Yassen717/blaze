//! Keyboard event → PTY byte sequence encoder.
//!
//! This module is the single source of truth for all key-to-bytes mappings.
//! It is intentionally kept free of side-effects: every public function is
//! pure (input → output, no I/O, no global state).
//!
//! # Public API
//!
//! * [`key_to_bytes`] — pure mapping from a `(Key, Modifiers)` pair to the
//!   byte sequence that should be written to the PTY master.  Call this from
//!   Dioxus event handlers: `key_to_bytes(e.key(), e.modifiers())`.
//! * [`paste_to_bytes`] — converts clipboard text to PTY bytes, with optional
//!   bracketed-paste wrapping.
//!
//! # VT100 / xterm sequences used
//!
//! | Key group        | Sequence family                              |
//! |------------------|----------------------------------------------|
//! | Arrow keys       | CSI A–D  (`\x1b[A` … `\x1b[D`)              |
//! | Home / End       | CSI H / CSI F                                |
//! | F1–F4            | SS3 P–S  (`\x1bOP` … `\x1bOS`)              |
//! | F5–F12           | CSI tilde  (`\x1b[15~` … `\x1b[24~`)        |
//! | Ins/Del/PgUp/Dn  | CSI tilde  (`\x1b[2~` … `\x1b[6~`)          |
//! | Ctrl + letter    | `letter & 0x1f`  (C0 control codes)          |
//! | Alt + key        | ESC prefix (`\x1b` + normal bytes)           |

use dioxus::prelude::{Key, Modifiers};

// ── Public API ────────────────────────────────────────────────────────────────

/// Convert a `(Key, Modifiers)` pair into the byte sequence that should be
/// written to the PTY master.
///
/// Call this from a Dioxus `onkeydown` handler:
/// ```rust,ignore
/// onkeydown: move |e| {
///     e.prevent_default();
///     if let Some(bytes) = key_to_bytes(e.key(), e.modifiers()) {
///         session.write_tx.send(bytes).ok();
///     }
/// }
/// ```
///
/// Returns `None` for:
/// * modifier-only key presses (Shift, Ctrl, Alt, Meta alone),
/// * dead keys / compose sequences,
/// * any key combination without a defined terminal encoding.
pub fn key_to_bytes(key: Key, mods: Modifiers) -> Option<Vec<u8>> {
    let ctrl  = mods.contains(Modifiers::CONTROL);
    let alt   = mods.contains(Modifiers::ALT);

    match key {
        // ── Modifier-only keys ────────────────────────────────────────────────
        // These produce no bytes on their own.
        Key::Alt
        | Key::AltGraph
        | Key::CapsLock
        | Key::Control
        | Key::Fn
        | Key::FnLock
        | Key::Meta
        | Key::NumLock
        | Key::ScrollLock
        | Key::Shift
        | Key::Super
        | Key::Hyper
        | Key::Symbol
        | Key::SymbolLock => None,

        // ── Dead / compose keys ───────────────────────────────────────────────
        Key::Dead => None,

        // ── Standard editing keys ─────────────────────────────────────────────

        // Enter always sends CR (\r), never LF (\n).
        Key::Enter => Some(b"\r".to_vec()),

        // Backspace sends DEL (0x7f), matching xterm default behaviour.
        Key::Backspace => Some(b"\x7f".to_vec()),

        // Tab sends HT (0x09).
        Key::Tab => Some(b"\x09".to_vec()),

        // Escape sends ESC (0x1b).
        Key::Escape => Some(b"\x1b".to_vec()),

        // ── Arrow keys (CSI A–D) ──────────────────────────────────────────────
        Key::ArrowUp    => Some(b"\x1b[A".to_vec()),
        Key::ArrowDown  => Some(b"\x1b[B".to_vec()),
        Key::ArrowRight => Some(b"\x1b[C".to_vec()),
        Key::ArrowLeft  => Some(b"\x1b[D".to_vec()),

        // ── Navigation keys ───────────────────────────────────────────────────
        Key::Home     => Some(b"\x1b[H".to_vec()),
        Key::End      => Some(b"\x1b[F".to_vec()),
        Key::PageUp   => Some(b"\x1b[5~".to_vec()),
        Key::PageDown => Some(b"\x1b[6~".to_vec()),
        Key::Insert   => Some(b"\x1b[2~".to_vec()),
        Key::Delete   => Some(b"\x1b[3~".to_vec()),

        // ── Function keys ─────────────────────────────────────────────────────
        //
        // F1–F4 use the older VT220 SS3 P–S sequences.
        // F5–F12 use the xterm CSI tilde sequences.
        // Note: there is no \x1b[16~ — F6 jumps from 15 to 17.
        Key::F1  => Some(b"\x1bOP".to_vec()),
        Key::F2  => Some(b"\x1bOQ".to_vec()),
        Key::F3  => Some(b"\x1bOR".to_vec()),
        Key::F4  => Some(b"\x1bOS".to_vec()),
        Key::F5  => Some(b"\x1b[15~".to_vec()),
        Key::F6  => Some(b"\x1b[17~".to_vec()),
        Key::F7  => Some(b"\x1b[18~".to_vec()),
        Key::F8  => Some(b"\x1b[19~".to_vec()),
        Key::F9  => Some(b"\x1b[20~".to_vec()),
        Key::F10 => Some(b"\x1b[21~".to_vec()),
        Key::F11 => Some(b"\x1b[23~".to_vec()),
        Key::F12 => Some(b"\x1b[24~".to_vec()),

        // ── Printable / character keys ────────────────────────────────────────
        Key::Character(s) => {
            if s.is_empty() {
                return None;
            }

            if ctrl {
                return ctrl_char_bytes(&s, alt);
            }

            // Plain printable character, possibly with Alt prefix.
            let mut bytes = Vec::with_capacity(s.len() + if alt { 1 } else { 0 });
            if alt {
                bytes.push(0x1b);
            }
            bytes.extend_from_slice(s.as_bytes());
            Some(bytes)
        }

        // ── Everything else ───────────────────────────────────────────────────
        _ => None,
    }
}

/// Encode a Ctrl+<character> combination.
///
/// `s` is the character string from `Key::Character` (as reported by the
/// webview — usually the unmodified lowercase letter, e.g. `"c"` for Ctrl+C).
///
/// Returns `None` when the combination has no standard terminal encoding.
fn ctrl_char_bytes(s: &str, alt: bool) -> Option<Vec<u8>> {
    let c = s.chars().next()?;

    // Ctrl + letter: `letter & 0x1f` gives C0 control codes 0x01–0x1a.
    // Uppercase and lowercase both map to the same control byte.
    let lower = c.to_ascii_lowercase();
    let code: Option<u8> = if lower.is_ascii_alphabetic() {
        Some(lower as u8 & 0x1f)
    } else {
        // Ctrl + symbol combinations with defined terminal meanings.
        match c {
            // Ctrl+@ and Ctrl+Space → NUL (0x00)
            '@' | ' ' | '2' => Some(0x00),
            // Ctrl+[ → ESC (0x1b)  — also achievable as plain Escape key
            '[' | '3'       => Some(0x1b),
            // Ctrl+\ → FS (0x1c)
            '\\' | '4'      => Some(0x1c),
            // Ctrl+] → GS (0x1d)
            ']' | '5'       => Some(0x1d),
            // Ctrl+^ → RS (0x1e)
            '^' | '6'       => Some(0x1e),
            // Ctrl+_ → US (0x1f)
            '_' | '7'       => Some(0x1f),
            // Ctrl+8 → DEL (0x7f)
            '8'             => Some(0x7f),
            _               => None,
        }
    };

    let byte = code?;

    // Alt+Ctrl → ESC prefix + control byte.
    if alt {
        Some(vec![0x1b, byte])
    } else {
        Some(vec![byte])
    }
}

// ── Paste support ─────────────────────────────────────────────────────────────

/// Convert clipboard text into the byte sequence to send to the PTY.
///
/// When `bracketed_paste` is `true` (the shell or running program has enabled
/// bracketed paste mode via `\x1b[?2004h`), the text is wrapped in the
/// bracketed-paste start/end markers.  This tells the shell to treat the
/// entire pasted block as an atomic insertion rather than executing each line
/// as a separate command — important for multi-line pastes and for preventing
/// accidental command execution when pasting into prompts.
///
/// The `bracketed_paste` flag should be read from
/// [`ScreenSnapshot::bracketed_paste`] which reflects the current state
/// reported by the vt100 parser.
pub fn paste_to_bytes(text: &str, bracketed_paste: bool) -> Vec<u8> {
    let extra = if bracketed_paste { 12 } else { 0 }; // len("\x1b[200~") * 2
    let mut out = Vec::with_capacity(text.len() + extra);

    if bracketed_paste {
        out.extend_from_slice(b"\x1b[200~");
    }
    out.extend_from_slice(text.as_bytes());
    if bracketed_paste {
        out.extend_from_slice(b"\x1b[201~");
    }

    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::{Key, Modifiers};
    #[allow(unused_imports)]
    use super::*;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn no_mods() -> Modifiers { Modifiers::empty() }
    fn ctrl()    -> Modifiers { Modifiers::CONTROL }
    fn alt()     -> Modifiers { Modifiers::ALT }
    fn ctrl_alt()-> Modifiers { Modifiers::CONTROL | Modifiers::ALT }

    /// Build a `Key::Character` from a string slice.
    fn k(s: &str) -> Key { Key::Character(s.into()) }

    // ── Editing keys ──────────────────────────────────────────────────────────

    #[test]
    fn enter_encodes_to_cr_not_lf() {
        assert_eq!(key_to_bytes(Key::Enter, no_mods()), Some(vec![0x0d]));
    }

    #[test]
    fn backspace_encodes_to_del_0x7f() {
        assert_eq!(key_to_bytes(Key::Backspace, no_mods()), Some(vec![0x7f]));
    }

    #[test]
    fn tab_encodes_to_ht_0x09() {
        assert_eq!(key_to_bytes(Key::Tab, no_mods()), Some(vec![0x09]));
    }

    #[test]
    fn escape_encodes_to_esc_0x1b() {
        assert_eq!(key_to_bytes(Key::Escape, no_mods()), Some(vec![0x1b]));
    }

    // ── Arrow keys ────────────────────────────────────────────────────────────

    #[test]
    fn arrow_up_encodes_to_csi_a() {
        assert_eq!(key_to_bytes(Key::ArrowUp, no_mods()), Some(b"\x1b[A".to_vec()));
    }

    #[test]
    fn arrow_down_encodes_to_csi_b() {
        assert_eq!(key_to_bytes(Key::ArrowDown, no_mods()), Some(b"\x1b[B".to_vec()));
    }

    #[test]
    fn arrow_right_encodes_to_csi_c() {
        assert_eq!(key_to_bytes(Key::ArrowRight, no_mods()), Some(b"\x1b[C".to_vec()));
    }

    #[test]
    fn arrow_left_encodes_to_csi_d() {
        assert_eq!(key_to_bytes(Key::ArrowLeft, no_mods()), Some(b"\x1b[D".to_vec()));
    }

    // ── Navigation ────────────────────────────────────────────────────────────

    #[test]
    fn home_encodes_to_csi_h() {
        assert_eq!(key_to_bytes(Key::Home, no_mods()), Some(b"\x1b[H".to_vec()));
    }

    #[test]
    fn end_encodes_to_csi_f() {
        assert_eq!(key_to_bytes(Key::End, no_mods()), Some(b"\x1b[F".to_vec()));
    }

    #[test]
    fn page_up_encodes_to_csi_5_tilde() {
        assert_eq!(key_to_bytes(Key::PageUp, no_mods()), Some(b"\x1b[5~".to_vec()));
    }

    #[test]
    fn page_down_encodes_to_csi_6_tilde() {
        assert_eq!(key_to_bytes(Key::PageDown, no_mods()), Some(b"\x1b[6~".to_vec()));
    }

    #[test]
    fn insert_encodes_to_csi_2_tilde() {
        assert_eq!(key_to_bytes(Key::Insert, no_mods()), Some(b"\x1b[2~".to_vec()));
    }

    #[test]
    fn delete_encodes_to_csi_3_tilde() {
        assert_eq!(key_to_bytes(Key::Delete, no_mods()), Some(b"\x1b[3~".to_vec()));
    }

    // ── Function keys ─────────────────────────────────────────────────────────

    #[test]
    fn f1_to_f4_use_ss3_sequences() {
        assert_eq!(key_to_bytes(Key::F1, no_mods()), Some(b"\x1bOP".to_vec()));
        assert_eq!(key_to_bytes(Key::F2, no_mods()), Some(b"\x1bOQ".to_vec()));
        assert_eq!(key_to_bytes(Key::F3, no_mods()), Some(b"\x1bOR".to_vec()));
        assert_eq!(key_to_bytes(Key::F4, no_mods()), Some(b"\x1bOS".to_vec()));
    }

    #[test]
    fn f5_encodes_to_csi_15_tilde() {
        assert_eq!(key_to_bytes(Key::F5, no_mods()), Some(b"\x1b[15~".to_vec()));
    }

    #[test]
    fn f6_skips_16_and_encodes_to_csi_17_tilde() {
        // There is no \x1b[16~ in the xterm sequence table.
        assert_eq!(key_to_bytes(Key::F6, no_mods()), Some(b"\x1b[17~".to_vec()));
    }

    #[test]
    fn f7_to_f12_use_csi_tilde_sequences() {
        assert_eq!(key_to_bytes(Key::F7,  no_mods()), Some(b"\x1b[18~".to_vec()));
        assert_eq!(key_to_bytes(Key::F8,  no_mods()), Some(b"\x1b[19~".to_vec()));
        assert_eq!(key_to_bytes(Key::F9,  no_mods()), Some(b"\x1b[20~".to_vec()));
        assert_eq!(key_to_bytes(Key::F10, no_mods()), Some(b"\x1b[21~".to_vec()));
        assert_eq!(key_to_bytes(Key::F11, no_mods()), Some(b"\x1b[23~".to_vec()));
        assert_eq!(key_to_bytes(Key::F12, no_mods()), Some(b"\x1b[24~".to_vec()));
    }

    // ── Printable characters ──────────────────────────────────────────────────

    #[test]
    fn printable_ascii_is_identity() {
        assert_eq!(key_to_bytes(k("a"), no_mods()), Some(b"a".to_vec()));
        assert_eq!(key_to_bytes(k("Z"), no_mods()), Some(b"Z".to_vec()));
        assert_eq!(key_to_bytes(k("1"), no_mods()), Some(b"1".to_vec()));
        assert_eq!(key_to_bytes(k("!"), no_mods()), Some(b"!".to_vec()));
        assert_eq!(key_to_bytes(k(" "), no_mods()), Some(b" ".to_vec()));
    }

    #[test]
    fn unicode_character_is_encoded_as_utf8() {
        // "é" is U+00E9, UTF-8: [0xC3, 0xA9]
        let result = key_to_bytes(k("é"), no_mods()).unwrap();
        assert_eq!(result, "é".as_bytes());
    }

    #[test]
    fn multibyte_unicode_roundtrips() {
        let snowman = "☃"; // U+2603, UTF-8: [0xE2, 0x98, 0x83]
        let result = key_to_bytes(k(snowman), no_mods()).unwrap();
        assert_eq!(result, snowman.as_bytes());
    }

    // ── Ctrl sequences ────────────────────────────────────────────────────────

    #[test]
    fn ctrl_a_encodes_to_soh() {
        assert_eq!(key_to_bytes(k("a"), ctrl()), Some(vec![0x01]));
    }

    #[test]
    fn ctrl_c_encodes_to_etx() {
        assert_eq!(key_to_bytes(k("c"), ctrl()), Some(vec![0x03]));
    }

    #[test]
    fn ctrl_d_encodes_to_eot() {
        assert_eq!(key_to_bytes(k("d"), ctrl()), Some(vec![0x04]));
    }

    #[test]
    fn ctrl_l_encodes_to_ff() {
        assert_eq!(key_to_bytes(k("l"), ctrl()), Some(vec![0x0c]));
    }

    #[test]
    fn ctrl_r_encodes_to_dc2() {
        assert_eq!(key_to_bytes(k("r"), ctrl()), Some(vec![0x12]));
    }

    #[test]
    fn ctrl_u_encodes_to_nak() {
        assert_eq!(key_to_bytes(k("u"), ctrl()), Some(vec![0x15]));
    }

    #[test]
    fn ctrl_w_encodes_to_etb() {
        assert_eq!(key_to_bytes(k("w"), ctrl()), Some(vec![0x17]));
    }

    #[test]
    fn ctrl_z_encodes_to_sub() {
        assert_eq!(key_to_bytes(k("z"), ctrl()), Some(vec![0x1a]));
    }

    #[test]
    fn ctrl_uppercase_maps_same_as_lowercase() {
        // Ctrl+A and Ctrl+a both produce SOH (0x01).
        assert_eq!(key_to_bytes(k("A"), ctrl()), Some(vec![0x01]));
        assert_eq!(key_to_bytes(k("C"), ctrl()), Some(vec![0x03]));
    }

    #[test]
    fn ctrl_full_alphabet_uses_c0_range() {
        for (i, ch) in (b'a'..=b'z').enumerate() {
            let key = k(std::str::from_utf8(&[ch]).unwrap());
            let expected = vec![(i + 1) as u8]; // SOH=1, STX=2, … SUB=26
            assert_eq!(
                key_to_bytes(key, ctrl()),
                Some(expected),
                "Ctrl+{} should map to C0 byte {}",
                ch as char,
                i + 1
            );
        }
    }

    #[test]
    fn ctrl_bracket_encodes_to_esc() {
        assert_eq!(key_to_bytes(k("["), ctrl()), Some(vec![0x1b]));
    }

    #[test]
    fn ctrl_backslash_encodes_to_fs() {
        assert_eq!(key_to_bytes(k("\\"), ctrl()), Some(vec![0x1c]));
    }

    #[test]
    fn ctrl_right_bracket_encodes_to_gs() {
        assert_eq!(key_to_bytes(k("]"), ctrl()), Some(vec![0x1d]));
    }

    #[test]
    fn ctrl_caret_encodes_to_rs() {
        assert_eq!(key_to_bytes(k("^"), ctrl()), Some(vec![0x1e]));
    }

    #[test]
    fn ctrl_underscore_encodes_to_us() {
        assert_eq!(key_to_bytes(k("_"), ctrl()), Some(vec![0x1f]));
    }

    // ── Alt sequences ─────────────────────────────────────────────────────────

    #[test]
    fn alt_char_prepends_esc() {
        assert_eq!(key_to_bytes(k("a"), alt()), Some(vec![0x1b, b'a']));
        assert_eq!(key_to_bytes(k("b"), alt()), Some(vec![0x1b, b'b']));
    }

    #[test]
    fn alt_ctrl_char_prepends_esc_before_control_byte() {
        // Alt+Ctrl+A → ESC + SOH
        assert_eq!(key_to_bytes(k("a"), ctrl_alt()), Some(vec![0x1b, 0x01]));
        // Alt+Ctrl+C → ESC + ETX
        assert_eq!(key_to_bytes(k("c"), ctrl_alt()), Some(vec![0x1b, 0x03]));
    }

    #[test]
    fn alt_unicode_char_prepends_esc_before_utf8() {
        let mut expected = vec![0x1b_u8];
        expected.extend_from_slice("é".as_bytes());
        assert_eq!(key_to_bytes(k("é"), alt()), Some(expected));
    }

    // ── Modifier-only keys → None ─────────────────────────────────────────────

    #[test]
    fn modifier_only_keys_return_none() {
        assert_eq!(key_to_bytes(Key::Control,    no_mods()), None);
        assert_eq!(key_to_bytes(Key::Alt,        no_mods()), None);
        assert_eq!(key_to_bytes(Key::Shift,      no_mods()), None);
        assert_eq!(key_to_bytes(Key::Meta,       no_mods()), None);
        assert_eq!(key_to_bytes(Key::CapsLock,   no_mods()), None);
        assert_eq!(key_to_bytes(Key::NumLock,    no_mods()), None);
        assert_eq!(key_to_bytes(Key::ScrollLock, no_mods()), None);
    }

    #[test]
    fn dead_key_returns_none() {
        assert_eq!(key_to_bytes(Key::Dead, no_mods()), None);
    }

    #[test]
    fn empty_character_returns_none() {
        assert_eq!(key_to_bytes(k(""), no_mods()), None);
    }

    // ── paste_to_bytes ────────────────────────────────────────────────────────

    #[test]
    fn paste_without_bracketed_mode_is_raw_bytes() {
        assert_eq!(paste_to_bytes("hello", false), b"hello");
    }

    #[test]
    fn paste_with_bracketed_mode_wraps_in_markers() {
        let out = paste_to_bytes("hello", true);
        assert_eq!(&out[..6],        b"\x1b[200~");
        assert_eq!(&out[6..11],      b"hello");
        assert_eq!(&out[11..],       b"\x1b[201~");
    }

    #[test]
    fn paste_multiline_bracketed_preserves_newlines() {
        let text = "line1\nline2\nline3";
        let out  = paste_to_bytes(text, true);
        let inner = &out[6..out.len() - 6];
        assert_eq!(inner, text.as_bytes(),
            "multiline content must be preserved verbatim inside markers");
    }

    #[test]
    fn paste_empty_string_without_brackets() {
        assert_eq!(paste_to_bytes("", false), b"" as &[u8]);
    }

    #[test]
    fn paste_empty_string_with_brackets() {
        assert_eq!(paste_to_bytes("", true), b"\x1b[200~\x1b[201~" as &[u8]);
    }

    #[test]
    fn paste_unicode_is_encoded_as_utf8() {
        let text = "héllo wörld";
        let out  = paste_to_bytes(text, false);
        assert_eq!(out, text.as_bytes());
    }

    #[test]
    fn paste_capacity_hint_is_correct_without_brackets() {
        let text = "abc";
        let out  = paste_to_bytes(text, false);
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn paste_capacity_hint_is_correct_with_brackets() {
        let text = "abc"; // 3 bytes + 6 + 6 = 15
        let out  = paste_to_bytes(text, true);
        assert_eq!(out.len(), 15);
    }
}
