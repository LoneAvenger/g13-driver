use std::collections::HashMap;
use std::collections::HashSet;

pub type VKey = u16;

// Windows virtual-key codes for the left-hand modifiers. We send the
// *left* variants (not the generic VK_CONTROL/VK_SHIFT) so the scancode we
// derive is unambiguous — games that read raw input see a real "left ctrl".
pub const VK_LSHIFT: VKey = 0xA0;
pub const VK_LCONTROL: VKey = 0xA2;
pub const VK_LMENU: VKey = 0xA4; // left alt
pub const VK_LWIN: VKey = 0x5B;

pub fn build_key_map() -> HashMap<String, VKey> {
    let mut m: HashMap<String, VKey> = HashMap::new();

    // a-z → VK_A (0x41) through VK_Z (0x5A)
    for c in b'a'..=b'z' {
        m.insert((c as char).to_string(), (c - b'a' + 0x41) as VKey);
    }
    // 0-9 (top row) → VK_0 (0x30) through VK_9 (0x39)
    for c in b'0'..=b'9' {
        m.insert((c as char).to_string(), (c - b'0' + 0x30) as VKey);
    }
    // Numpad 0-9 → VK_NUMPAD0 (0x60) through VK_NUMPAD9 (0x69)
    for i in 0u16..=9 {
        m.insert(format!("num{i}"), 0x60 + i);
    }
    // F1-F24 → 0x70 through 0x87
    for i in 1u16..=24 {
        m.insert(format!("f{i}"), 0x6F + i);
    }

    let specials: &[(&str, VKey)] = &[
        ("enter",       0x0D), ("return",       0x0D),
        ("esc",         0x1B), ("escape",        0x1B),
        ("space",       0x20), ("tab",           0x09),
        ("backspace",   0x08), ("delete",        0x2E),
        ("insert",      0x2D), ("home",          0x24),
        ("end",         0x23), ("pageup",        0x21),
        ("pagedown",    0x22), ("up",            0x26),
        ("down",        0x28), ("left",          0x25),
        ("right",       0x27), ("capslock",      0x14),
        ("printscreen", 0x2C), ("pause",         0x13),
        ("numlock",     0x90), ("scrolllock",    0x91),
        ("menu",        0x5D),
        // Modifiers as plain keys (left-hand variants, see consts above).
        ("ctrl",        VK_LCONTROL), ("control", VK_LCONTROL),
        ("shift",       VK_LSHIFT),
        ("alt",         VK_LMENU),
        ("windows",     VK_LWIN), ("win", VK_LWIN),
        // Right-hand modifiers, bindable as ordinary keys (handy for push-to-talk).
        ("rctrl",       0xA3), ("rshift",       0xA1),
        ("ralt",        0xA5), ("rwin",         0x5C),
        // Numpad operators.
        ("nummultiply", 0x6A), ("numplus",      0x6B),
        ("numminus",    0x6D), ("numdecimal",   0x6E),
        ("numdivide",   0x6F),
        // Punctuation (US-layout VKs; the Windows injector re-resolves these
        // through the active keyboard layout, see `punctuation_char`).
        ("minus",       0xBD), ("equals",       0xBB),
        ("comma",       0xBC), ("period",       0xBE),
        ("slash",       0xBF), ("semicolon",    0xBA),
        ("quote",       0xDE), ("backslash",    0xDC),
        ("lbracket",    0xDB), ("rbracket",     0xDD),
        ("grave",       0xC0), ("hash",         0xDE),
        // Multimedia keys (tap-only).
        ("playpause",   0xB3),
        ("nexttrack",   0xB0), ("next",         0xB0),
        ("prevtrack",   0xB1), ("prev",         0xB1),
        ("mediastop",   0xB2),
        ("volup",       0xAF), ("volumeup",     0xAF),
        ("voldown",     0xAE), ("volumedown",   0xAE),
        ("mute",        0xAD),
    ];
    for (name, vk) in specials {
        m.insert(name.to_string(), *vk);
    }
    m
}

/// Map friendly / alternative spellings to the canonical key name used in
/// `build_key_map`. Unknown names pass through unchanged (and are later
/// rejected as invalid). Input is expected lower-case and trimmed.
pub fn canonical_name(name: &str) -> &str {
    match name {
        "spacebar" | "space bar" | "spc" => "space",
        "ctl" | "control" | "lctrl" | "lcontrol" | "leftctrl" | "leftcontrol" => "ctrl",
        "lshift" | "leftshift" => "shift",
        "lalt" | "leftalt" | "option" => "alt",
        "lwin" | "windows" | "super" | "meta" | "cmd" => "win",
        "rightctrl" | "rcontrol" | "rightcontrol" => "rctrl",
        "rightshift" => "rshift",
        "rightalt" | "altgr" => "ralt",
        "rightwin" => "rwin",
        "return" => "enter",
        "escape" => "esc",
        "del" => "delete",
        "ins" => "insert",
        "pgup" => "pageup",
        "pgdn" | "pgdown" => "pagedown",
        "bksp" | "back" => "backspace",
        "caps" => "capslock",
        "prtsc" | "prtscr" | "print" => "printscreen",
        "arrowup" | "uparrow" => "up",
        "arrowdown" | "downarrow" => "down",
        "arrowleft" | "leftarrow" => "left",
        "arrowright" | "rightarrow" => "right",
        "next" => "nexttrack",
        "prev" => "prevtrack",
        "volumeup" => "volup",
        "volumedown" => "voldown",
        "apps" | "context" => "menu",
        // Literal punctuation characters.
        "-" => "minus",
        "=" => "equals",
        "," => "comma",
        "." => "period",
        "/" => "slash",
        ";" => "semicolon",
        "'" => "quote",
        "\\" => "backslash",
        "[" => "lbracket",
        "]" => "rbracket",
        "`" | "backtick" => "grave",
        "#" => "hash",
        "numpad0" => "num0", "numpad1" => "num1", "numpad2" => "num2",
        "numpad3" => "num3", "numpad4" => "num4", "numpad5" => "num5",
        "numpad6" => "num6", "numpad7" => "num7", "numpad8" => "num8",
        "numpad9" => "num9",
        "num*" | "multiply" => "nummultiply",
        "num+" | "add" => "numplus",
        "num-" | "subtract" => "numminus",
        "num." | "decimal" => "numdecimal",
        "num/" | "divide" => "numdivide",
        other => other,
    }
}

/// The character a punctuation key name types, so the Windows injector can
/// find the right physical key for the user's keyboard layout (UK and US
/// differ for ' # ` etc.).
pub fn punctuation_char(name: &str) -> Option<char> {
    Some(match name {
        "minus" => '-', "equals" => '=', "comma" => ',', "period" => '.',
        "slash" => '/', "semicolon" => ';', "quote" => '\'', "backslash" => '\\',
        "lbracket" => '[', "rbracket" => ']', "grave" => '`', "hash" => '#',
        _ => return None,
    })
}

/// Names of keys that should tap (down+up on press) rather than hold — the
/// multimedia keys, where holding is meaningless. Everything else holds.
pub fn tap_only_keys() -> HashSet<String> {
    ["playpause", "nexttrack", "next", "prevtrack", "prev", "mediastop",
     "volup", "volumeup", "voldown", "volumedown", "mute"]
        .iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_a_to_z() {
        let m = build_key_map();
        assert_eq!(m["a"], 0x41);
        assert_eq!(m["z"], 0x5A);
        assert_eq!(m["m"], 0x4D);
    }

    #[test]
    fn digits_0_to_9() {
        let m = build_key_map();
        assert_eq!(m["0"], 0x30);
        assert_eq!(m["9"], 0x39);
        assert_eq!(m["num0"], 0x60);
        assert_eq!(m["num9"], 0x69);
    }

    #[test]
    fn function_keys() {
        let m = build_key_map();
        assert_eq!(m["f1"],  0x70);
        assert_eq!(m["f12"], 0x7B);
        assert_eq!(m["f24"], 0x87);
    }

    #[test]
    fn special_keys() {
        let m = build_key_map();
        assert_eq!(m["enter"],     0x0D);
        assert_eq!(m["esc"],       0x1B);
        assert_eq!(m["space"],     0x20);
        assert_eq!(m["backspace"], 0x08);
        assert_eq!(m["delete"],    0x2E);
        assert_eq!(m["pageup"],    0x21);
        assert_eq!(m["up"],        0x26);
    }

    #[test]
    fn modifier_keys_are_left_hand_variants() {
        let m = build_key_map();
        assert_eq!(m["ctrl"],    VK_LCONTROL);
        assert_eq!(m["shift"],   VK_LSHIFT);
        assert_eq!(m["alt"],     VK_LMENU);
        assert_eq!(m["windows"], VK_LWIN);
        assert_eq!(m["rctrl"],   0xA3);
    }

    #[test]
    fn aliases_resolve_to_known_keys() {
        let m = build_key_map();
        for alias in ["spacebar", "lctrl", "control", "return", "escape", "pgdn", "-", "'", "#", "numpad5", "altgr"] {
            let c = canonical_name(alias);
            assert!(m.contains_key(c), "{alias} -> {c} not in key map");
        }
    }

    #[test]
    fn every_punctuation_name_has_a_char() {
        for n in ["minus", "equals", "comma", "period", "slash", "semicolon", "quote",
                  "backslash", "lbracket", "rbracket", "grave", "hash"] {
            assert!(punctuation_char(n).is_some(), "{n}");
        }
        assert!(punctuation_char("a").is_none());
    }

    #[test]
    fn unknown_key_absent() {
        let m = build_key_map();
        assert!(!m.contains_key("xyzzy"));
    }

    #[test]
    fn media_keys_present() {
        let m = build_key_map();
        assert_eq!(m["playpause"], 0xB3);
        assert_eq!(m["nexttrack"], 0xB0);
        assert_eq!(m["next"], 0xB0);
        assert_eq!(m["prevtrack"], 0xB1);
        assert_eq!(m["volup"], 0xAF);
        assert_eq!(m["voldown"], 0xAE);
        assert_eq!(m["mute"], 0xAD);
        assert_eq!(m["mediastop"], 0xB2);
    }

    #[test]
    fn tap_only_is_media_only() {
        let t = tap_only_keys();
        assert!(t.contains("playpause"));
        assert!(t.contains("volup"));
        assert!(t.contains("mute"));
        assert!(!t.contains("a"));
        assert!(!t.contains("shift"));
        assert!(!t.contains("f5"));
    }
}
