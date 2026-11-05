//! "Press a key" capture for the Bindings editor.
//!
//! Click **Set**, then press the key or combo you want: `Space`, `Ctrl` on its
//! own, `Ctrl+1`, `Ctrl+Shift+F5`… A modifier pressed and released with no
//! other key becomes a modifier-only binding (great for push-to-talk).
//! `Esc` or a mouse click cancels.

use eframe::egui::{Event, Key, Modifiers};

/// What a capture produced this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureResult {
    Pending,
    Done(String),
    Cancelled,
}

/// In-progress capture. Feed it each frame's input until it finishes.
#[derive(Debug, Clone, Default)]
pub struct KeyCapture {
    /// The most modifiers held at once so far (for modifier-only bindings).
    peak_ctrl: bool,
    peak_shift: bool,
    peak_alt: bool,
}

fn mods_text(ctrl: bool, shift: bool, alt: bool) -> Vec<&'static str> {
    let mut v = Vec::new();
    if ctrl { v.push("ctrl"); }
    if shift { v.push("shift"); }
    if alt { v.push("alt"); }
    v
}

impl KeyCapture {
    pub fn new() -> Self { Self::default() }

    /// Text shown while waiting, e.g. "ctrl+…" once Ctrl is held.
    pub fn preview(&self, mods: Modifiers) -> String {
        let held = mods_text(mods.ctrl, mods.shift, mods.alt);
        if held.is_empty() { "press a key…".into() } else { format!("{}+…", held.join("+")) }
    }

    /// Process one frame of input. `mods` is the current modifier state.
    pub fn feed(&mut self, events: &[Event], mods: Modifiers) -> CaptureResult {
        for ev in events {
            match ev {
                Event::Key { key, physical_key, pressed: true, modifiers, .. } => {
                    let no_mods = !(modifiers.ctrl || modifiers.shift || modifiers.alt);
                    if *key == Key::Escape && no_mods {
                        return CaptureResult::Cancelled;
                    }
                    if let Some(name) = pick_name(*key, *physical_key) {
                        let mut parts = mods_text(modifiers.ctrl, modifiers.shift, modifiers.alt);
                        parts.push(name);
                        return CaptureResult::Done(parts.join("+"));
                    }
                }
                // Some platforms turn ctrl+c/x/v into clipboard events only.
                Event::Copy => return CaptureResult::Done("ctrl+c".into()),
                Event::Cut => return CaptureResult::Done("ctrl+x".into()),
                Event::Paste(_) => return CaptureResult::Done("ctrl+v".into()),
                Event::PointerButton { pressed: true, .. } => return CaptureResult::Cancelled,
                _ => {}
            }
        }
        self.peak_ctrl |= mods.ctrl;
        self.peak_shift |= mods.shift;
        self.peak_alt |= mods.alt;
        let any_held = mods.ctrl || mods.shift || mods.alt;
        let any_peak = self.peak_ctrl || self.peak_shift || self.peak_alt;
        if any_peak && !any_held {
            // Modifier(s) pressed and released on their own.
            return CaptureResult::Done(mods_text(self.peak_ctrl, self.peak_shift, self.peak_alt).join("+"));
        }
        CaptureResult::Pending
    }
}

/// Letters, digits and named keys come from the *physical* key (so Shift+1 is
/// "shift+1", not "!"); punctuation comes from the *logical* key so it follows
/// the user's keyboard layout (UK `#` stays `#`).
fn pick_name(logical: Key, physical: Option<Key>) -> Option<&'static str> {
    if let Some(p) = physical {
        if let Some(n) = key_name(p) {
            if !is_punctuation(p) { return Some(n); }
        }
    }
    key_name(logical)
}

fn is_punctuation(k: Key) -> bool {
    matches!(k, Key::Minus | Key::Equals | Key::Comma | Key::Period | Key::Slash | Key::Semicolon
        | Key::Quote | Key::Backslash | Key::OpenBracket | Key::CloseBracket | Key::Backtick
        | Key::Plus | Key::Colon | Key::Pipe | Key::Questionmark | Key::Exclamationmark
        | Key::OpenCurlyBracket | Key::CloseCurlyBracket)
}

/// egui key → our key name (see `injector::key_map`).
pub fn key_name(k: Key) -> Option<&'static str> {
    use Key::*;
    Some(match k {
        A => "a", B => "b", C => "c", D => "d", E => "e", F => "f", G => "g", H => "h",
        I => "i", J => "j", K => "k", L => "l", M => "m", N => "n", O => "o", P => "p",
        Q => "q", R => "r", S => "s", T => "t", U => "u", V => "v", W => "w", X => "x",
        Y => "y", Z => "z",
        Num0 => "0", Num1 => "1", Num2 => "2", Num3 => "3", Num4 => "4",
        Num5 => "5", Num6 => "6", Num7 => "7", Num8 => "8", Num9 => "9",
        F1 => "f1", F2 => "f2", F3 => "f3", F4 => "f4", F5 => "f5", F6 => "f6",
        F7 => "f7", F8 => "f8", F9 => "f9", F10 => "f10", F11 => "f11", F12 => "f12",
        F13 => "f13", F14 => "f14", F15 => "f15", F16 => "f16", F17 => "f17", F18 => "f18",
        F19 => "f19", F20 => "f20", F21 => "f21", F22 => "f22", F23 => "f23", F24 => "f24",
        Space => "space", Enter => "enter", Tab => "tab", Backspace => "backspace",
        Escape => "esc", Insert => "insert", Delete => "delete", Home => "home", End => "end",
        PageUp => "pageup", PageDown => "pagedown",
        ArrowUp => "up", ArrowDown => "down", ArrowLeft => "left", ArrowRight => "right",
        Minus => "minus", Equals => "equals", Comma => "comma", Period => "period",
        Slash => "slash", Semicolon => "semicolon", Quote => "quote", Backslash => "backslash",
        OpenBracket => "lbracket", CloseBracket => "rbracket", Backtick => "grave",
        _ => return None,
    })
}

/// Pretty label for a binding: "ctrl+shift+1" → "Ctrl + Shift + 1".
pub fn pretty(binding: &str) -> String {
    binding.split(',').map(|step| {
        step.trim().split('+').map(|p| {
            let p = p.trim();
            match p {
                "ctrl" => "Ctrl".to_string(), "shift" => "Shift".into(), "alt" => "Alt".into(),
                "win" => "Win".into(), "space" => "Space".into(), "esc" => "Esc".into(),
                "enter" => "Enter".into(), "tab" => "Tab".into(),
                "" => "Space".into(), // a literal " " part
                s if s.len() == 1 => s.to_uppercase(),
                s => {
                    let mut c = s.chars();
                    match c.next() { Some(f) => f.to_uppercase().collect::<String>() + c.as_str(), None => String::new() }
                }
            }
        }).collect::<Vec<_>>().join(" + ")
    }).collect::<Vec<_>>().join(" → ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(k: Key, phys: Option<Key>, m: Modifiers) -> Event {
        Event::Key { key: k, physical_key: phys, pressed: true, repeat: false, modifiers: m }
    }
    const CTRL: Modifiers = Modifiers { alt: false, ctrl: true, shift: false, mac_cmd: false, command: true };
    const SHIFT: Modifiers = Modifiers { alt: false, ctrl: false, shift: true, mac_cmd: false, command: false };

    #[test]
    fn space_alone() {
        let mut c = KeyCapture::new();
        assert_eq!(c.feed(&[key(Key::Space, Some(Key::Space), Modifiers::NONE)], Modifiers::NONE),
                   CaptureResult::Done("space".into()));
    }

    #[test]
    fn ctrl_alone_is_captured_on_release() {
        let mut c = KeyCapture::new();
        assert_eq!(c.feed(&[], CTRL), CaptureResult::Pending);
        assert_eq!(c.feed(&[], CTRL), CaptureResult::Pending);
        assert_eq!(c.feed(&[], Modifiers::NONE), CaptureResult::Done("ctrl".into()));
    }

    #[test]
    fn ctrl_plus_1() {
        let mut c = KeyCapture::new();
        assert_eq!(c.feed(&[], CTRL), CaptureResult::Pending);
        assert_eq!(c.feed(&[key(Key::Num1, Some(Key::Num1), CTRL)], CTRL),
                   CaptureResult::Done("ctrl+1".into()));
    }

    #[test]
    fn shift_1_uses_physical_key_not_bang() {
        let mut c = KeyCapture::new();
        assert_eq!(c.feed(&[key(Key::Exclamationmark, Some(Key::Num1), SHIFT)], SHIFT),
                   CaptureResult::Done("shift+1".into()));
    }

    #[test]
    fn escape_and_click_cancel() {
        let mut c = KeyCapture::new();
        assert_eq!(c.feed(&[key(Key::Escape, Some(Key::Escape), Modifiers::NONE)], Modifiers::NONE),
                   CaptureResult::Cancelled);
        let click = Event::PointerButton {
            pos: eframe::egui::pos2(0.0, 0.0), button: eframe::egui::PointerButton::Primary,
            pressed: true, modifiers: Modifiers::NONE,
        };
        assert_eq!(KeyCapture::new().feed(&[click], Modifiers::NONE), CaptureResult::Cancelled);
    }

    #[test]
    fn ctrl_c_clipboard_event() {
        assert_eq!(KeyCapture::new().feed(&[Event::Copy], CTRL), CaptureResult::Done("ctrl+c".into()));
    }

    #[test]
    fn captured_names_are_valid_bindings() {
        let valid: std::collections::HashSet<String> =
            crate::injector::key_map::build_key_map().into_keys().collect();
        use Key::*;
        for k in [A, Z, Num0, Num9, F1, F24, Space, Enter, Tab, Backspace, Escape, Insert, Delete,
                  Home, End, PageUp, PageDown, ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Minus,
                  Equals, Comma, Period, Slash, Semicolon, Quote, Backslash, OpenBracket,
                  CloseBracket, Backtick] {
            let n = key_name(k).unwrap();
            assert!(valid.contains(n), "{k:?} -> {n} not a valid key");
        }
    }

    #[test]
    fn pretty_labels() {
        assert_eq!(pretty("ctrl+1"), "Ctrl + 1");
        assert_eq!(pretty("space"), "Space");
        assert_eq!(pretty("ctrl+shift+f5"), "Ctrl + Shift + F5");
        assert_eq!(pretty("ctrl+1, wait 200, ctrl+2"), "Ctrl + 1 → Wait 200 → Ctrl + 2");
    }
}
