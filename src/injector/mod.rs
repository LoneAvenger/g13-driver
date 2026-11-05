pub mod key_map;
#[cfg(windows)]
pub mod windows;

use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCombo {
    pub modifiers: Vec<Modifier>,
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Windows,
}

pub trait KeyInjector: Send + Sync {
    fn press(&self, combo: &KeyCombo) -> Result<()>;
    /// Press and hold a single key down (no release). For joystick hold-to-move.
    fn key_down(&self, key: &str) -> Result<()>;
    /// Release a single key previously held with `key_down`.
    fn key_up(&self, key: &str) -> Result<()>;
    /// Press modifiers + key down and hold them (for hold-means-hold G-keys).
    fn combo_down(&self, combo: &KeyCombo) -> Result<()>;
    /// Release a combo previously held with `combo_down`.
    fn combo_up(&self, combo: &KeyCombo) -> Result<()>;
}

impl Modifier {
    pub fn name(&self) -> &'static str {
        match self {
            Modifier::Ctrl => "ctrl",
            Modifier::Shift => "shift",
            Modifier::Alt => "alt",
            Modifier::Windows => "win",
        }
    }
}

impl KeyCombo {
    /// Parse `modifier+modifier+key`. Forgiving about spelling: names are
    /// case-insensitive, aliases like `spacebar`/`lctrl`/`return` are accepted
    /// (see `key_map::canonical_name`), spaces around `+` are ignored, and a
    /// bare space character means the Space key (so pressing the space bar in
    /// the text box works too).
    pub fn parse(s: &str) -> Result<Self> {
        let lower = s.to_lowercase();
        let mut modifiers: Vec<Modifier> = Vec::new();
        let mut key: Option<String> = None;

        for raw in lower.split('+') {
            let trimmed = raw.trim();
            let part: &str = if trimmed.is_empty() {
                // A part made only of spaces is the space key itself
                // ("ctrl+ " or just " "). An empty part is a stray '+'.
                if raw.contains(' ') { "space" } else { continue }
            } else {
                key_map::canonical_name(trimmed)
            };
            let modifier = match part {
                "ctrl" => Some(Modifier::Ctrl),
                "shift" => Some(Modifier::Shift),
                "alt" => Some(Modifier::Alt),
                "win" => Some(Modifier::Windows),
                _ => None,
            };
            match modifier {
                Some(m) => {
                    if !modifiers.contains(&m) { modifiers.push(m); }
                }
                None => {
                    if key.is_some() {
                        bail!("multiple non-modifier keys in combo: {}", s);
                    }
                    key = Some(part.to_string());
                }
            }
        }

        if key.is_none() && modifiers.is_empty() {
            bail!("empty combo: {}", s);
        }
        Ok(Self { modifiers, key })
    }

    /// Canonical text form, e.g. `ctrl+shift+1`, `space`, `ctrl`.
    pub fn to_canonical(&self) -> String {
        let mut parts: Vec<&str> = self.modifiers.iter().map(Modifier::name).collect();
        if let Some(k) = &self.key { parts.push(k); }
        parts.join("+")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_key() {
        let c = KeyCombo::parse("f5").unwrap();
        assert_eq!(c.key.as_deref(), Some("f5"));
        assert!(c.modifiers.is_empty());
    }

    #[test]
    fn parse_ctrl_c() {
        let c = KeyCombo::parse("ctrl+c").unwrap();
        assert_eq!(c.key.as_deref(), Some("c"));
        assert_eq!(c.modifiers, vec![Modifier::Ctrl]);
    }

    #[test]
    fn parse_shift_ctrl_esc() {
        let c = KeyCombo::parse("shift+ctrl+esc").unwrap();
        assert_eq!(c.key.as_deref(), Some("esc"));
        assert!(c.modifiers.contains(&Modifier::Ctrl));
        assert!(c.modifiers.contains(&Modifier::Shift));
    }

    #[test]
    fn parse_is_case_insensitive() {
        let c = KeyCombo::parse("CTRL+C").unwrap();
        assert_eq!(c.key.as_deref(), Some("c"));
        assert_eq!(c.modifiers, vec![Modifier::Ctrl]);
    }

    #[test]
    fn parse_windows_key() {
        let c = KeyCombo::parse("windows+d").unwrap();
        assert_eq!(c.key.as_deref(), Some("d"));
        assert_eq!(c.modifiers, vec![Modifier::Windows]);
    }

    #[test]
    fn parse_modifier_only_is_ok() {
        let c = KeyCombo::parse("ctrl+shift").unwrap();
        assert!(c.key.is_none());
        assert_eq!(c.modifiers, vec![Modifier::Ctrl, Modifier::Shift]);
        let c = KeyCombo::parse("shift").unwrap();
        assert!(c.key.is_none());
        assert_eq!(c.modifiers, vec![Modifier::Shift]);
    }

    #[test]
    fn parse_empty_is_error() {
        assert!(KeyCombo::parse("").is_err());
        assert!(KeyCombo::parse("+").is_err());
    }

    #[test]
    fn parse_space_alone_any_spelling() {
        for s in ["space", "Space", "SPACE", " space ", "spacebar", " ", "   "] {
            let c = KeyCombo::parse(s).unwrap_or_else(|e| panic!("{s:?}: {e}"));
            assert_eq!(c.key.as_deref(), Some("space"), "{s:?}");
            assert!(c.modifiers.is_empty(), "{s:?}");
        }
    }

    #[test]
    fn parse_ctrl_alone_any_spelling() {
        for s in ["ctrl", "Ctrl", "control", "lctrl", "CTL"] {
            let c = KeyCombo::parse(s).unwrap();
            assert!(c.key.is_none(), "{s:?}");
            assert_eq!(c.modifiers, vec![Modifier::Ctrl], "{s:?}");
        }
    }

    #[test]
    fn parse_ctrl_digit_and_ctrl_space() {
        let c = KeyCombo::parse("Ctrl + 1").unwrap();
        assert_eq!(c.key.as_deref(), Some("1"));
        assert_eq!(c.modifiers, vec![Modifier::Ctrl]);
        let c = KeyCombo::parse("ctrl+ ").unwrap();
        assert_eq!(c.key.as_deref(), Some("space"));
        let c = KeyCombo::parse("ctrl+space").unwrap();
        assert_eq!(c.to_canonical(), "ctrl+space");
    }

    #[test]
    fn parse_aliases_and_punctuation() {
        assert_eq!(KeyCombo::parse("Return").unwrap().to_canonical(), "enter");
        assert_eq!(KeyCombo::parse("shift+-").unwrap().to_canonical(), "shift+minus");
        assert_eq!(KeyCombo::parse("alt+#").unwrap().to_canonical(), "alt+hash");
        assert_eq!(KeyCombo::parse("Numpad5").unwrap().to_canonical(), "num5");
    }

    #[test]
    fn duplicate_modifiers_collapse() {
        assert_eq!(KeyCombo::parse("ctrl+control+a").unwrap().to_canonical(), "ctrl+a");
    }

    #[test]
    fn canonical_round_trips() {
        for s in ["ctrl+shift+z", "space", "ctrl", "alt+f4", "win+d", "ctrl+1"] {
            let c = KeyCombo::parse(s).unwrap();
            assert_eq!(c.to_canonical(), s);
            assert_eq!(KeyCombo::parse(&c.to_canonical()).unwrap(), c);
        }
    }

    #[test]
    fn parse_two_keys_is_error() {
        assert!(KeyCombo::parse("a+b").is_err());
    }
}
