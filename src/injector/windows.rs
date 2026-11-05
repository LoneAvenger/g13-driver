use anyhow::{Context, Result};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, SendInput, VkKeyScanW, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC_EX,
};
use super::{KeyCombo, KeyInjector, Modifier};
use super::key_map::{build_key_map, canonical_name, punctuation_char, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN};
use std::collections::HashMap;

/// Injects keystrokes with `SendInput`.
///
/// Every key is sent as a *hardware scancode* (KEYEVENTF_SCANCODE) where the
/// key has one, exactly like a physical keyboard. The original driver sent
/// virtual-key codes with a zero scancode, which many games ignore because
/// they read raw input / DirectInput, where a zero scancode is "no key".
/// Media keys keep the virtual-key path (that's how Windows expects them).
pub struct WindowsInjector {
    key_map: HashMap<String, u16>,
}

/// A key resolved to its virtual-key code, plus whether the current keyboard
/// layout needs Shift held to produce it (only for punctuation like '#').
#[derive(Clone, Copy)]
struct Resolved {
    vk: u16,
    needs_shift: bool,
}

impl WindowsInjector {
    pub fn new() -> Self {
        Self { key_map: build_key_map() }
    }

    fn modifier_vk(m: &Modifier) -> u16 {
        match m {
            Modifier::Ctrl    => VK_LCONTROL,
            Modifier::Shift   => VK_LSHIFT,
            Modifier::Alt     => VK_LMENU,
            Modifier::Windows => VK_LWIN,
        }
    }

    fn resolve(&self, key: &str) -> Result<Resolved> {
        // Joystick directions arrive as raw config text (not via KeyCombo::parse),
        // so normalise case and aliases here too ("Spacebar" -> "space").
        let lower = key.trim().to_lowercase();
        let key = canonical_name(&lower);
        // Punctuation: ask the active keyboard layout which physical key types
        // this character (UK vs US layouts differ for ' # ` and friends).
        if let Some(ch) = punctuation_char(key) {
            let r = unsafe { VkKeyScanW(ch as u16) };
            if r != -1 {
                let vk = (r as u16) & 0xFF;
                let shift_state = ((r as u16) >> 8) & 0xFF;
                return Ok(Resolved { vk, needs_shift: shift_state & 1 != 0 });
            }
        }
        let vk = *self.key_map.get(key).with_context(|| format!("unknown key: {}", key))?;
        Ok(Resolved { vk, needs_shift: false })
    }

    /// Media / browser keys (VK 0xA6..=0xB7) are sent as virtual keys only.
    fn vk_only(vk: u16) -> bool {
        (0xA6..=0xB7).contains(&vk) || vk == 0x13 /* pause: odd E1 scancode */
    }

    fn make_input(vk: u16, up: bool) -> INPUT {
        let mut flags = if up { KEYEVENTF_KEYUP } else { 0 };
        let mut scan: u16 = 0;
        if !Self::vk_only(vk) {
            let sc = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC_EX) };
            if sc != 0 {
                scan = (sc & 0xFF) as u16;
                flags |= KEYEVENTF_SCANCODE;
                let prefix = sc & 0xFF00;
                if prefix == 0xE000 || prefix == 0xE100 {
                    flags |= KEYEVENTF_EXTENDEDKEY;
                }
            }
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn send_batch(inputs: &[INPUT], what: &str) {
        if inputs.is_empty() { return; }
        let sent = unsafe {
            SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32)
        };
        if (sent as usize) < inputs.len() {
            // Most common cause: the target window runs as Administrator and
            // we don't (Windows blocks input injection "upwards").
            log::warn!("SendInput sent {sent}/{} events for {what} — if the target app runs as admin, run g13-driver as admin too", inputs.len());
        }
    }

    /// Down events for a combo: modifiers first, then (layout shift), then key.
    fn down_events(&self, combo: &KeyCombo) -> Result<Vec<INPUT>> {
        let key = combo.key.as_deref().map(|k| self.resolve(k)).transpose()?;
        let mut inputs = Vec::new();
        for m in &combo.modifiers {
            inputs.push(Self::make_input(Self::modifier_vk(m), false));
        }
        if let Some(r) = key {
            if r.needs_shift && !combo.modifiers.contains(&Modifier::Shift) {
                inputs.push(Self::make_input(VK_LSHIFT, false));
            }
            inputs.push(Self::make_input(r.vk, false));
        }
        Ok(inputs)
    }

    /// Up events for a combo, in reverse order of `down_events`.
    fn up_events(&self, combo: &KeyCombo) -> Result<Vec<INPUT>> {
        let key = combo.key.as_deref().map(|k| self.resolve(k)).transpose()?;
        let mut inputs = Vec::new();
        if let Some(r) = key {
            inputs.push(Self::make_input(r.vk, true));
            if r.needs_shift && !combo.modifiers.contains(&Modifier::Shift) {
                inputs.push(Self::make_input(VK_LSHIFT, true));
            }
        }
        for m in combo.modifiers.iter().rev() {
            inputs.push(Self::make_input(Self::modifier_vk(m), true));
        }
        Ok(inputs)
    }
}

impl KeyInjector for WindowsInjector {
    fn press(&self, combo: &KeyCombo) -> Result<()> {
        let mut inputs = self.down_events(combo)?;
        inputs.extend(self.up_events(combo)?);
        Self::send_batch(&inputs, "press");
        Ok(())
    }

    fn key_down(&self, key: &str) -> Result<()> {
        let r = self.resolve(key)?;
        Self::send_batch(&[Self::make_input(r.vk, false)], "key_down");
        Ok(())
    }

    fn key_up(&self, key: &str) -> Result<()> {
        let r = self.resolve(key)?;
        Self::send_batch(&[Self::make_input(r.vk, true)], "key_up");
        Ok(())
    }

    fn combo_down(&self, combo: &KeyCombo) -> Result<()> {
        let inputs = self.down_events(combo)?;
        Self::send_batch(&inputs, "combo_down");
        Ok(())
    }

    fn combo_up(&self, combo: &KeyCombo) -> Result<()> {
        let inputs = self.up_events(combo)?;
        Self::send_batch(&inputs, "combo_up");
        Ok(())
    }
}
