# Configuration reference

The GUI (Bindings / Profiles / Settings tabs) is the easy way to configure g13-driver and covers
everything most people need. This document is the reference for hand-editing the config files.

## Where config lives

Next to `g13-driver.exe`:

- `config.toml` — the **manifest**: global settings + which profile file each M-key uses.
- `profiles/*.toml` — one file per profile (a full binding set).

The app looks for `config.toml` **next to the exe first, then in the current directory**. Files are
**hot-reloaded** — save and changes apply immediately (no restart).

> A GUI **Save** rewrites the profile file and does **not** preserve comments or key order (known
> limitation). If you keep comments, edit by hand.

## The manifest (`config.toml`)

```toml
profiles_dir = "profiles"   # folder holding the profile files
m1 = "default.toml"         # profile loaded on M1
m2 = "game.toml"            # profile loaded on M2 (optional)
m3 = "media.toml"           # profile loaded on M3 (optional)

[autorepeat]                # optional; global auto-repeat timing
delay_ms = 400              # wait before repeating starts
interval_ms = 40            # gap between repeats (min 1)

[app]                       # managed by the GUI
start_active = false        # resume Active (true) or Dry-run (false) on launch
```

(A legacy `config.toml` that is itself a bare `[keys]` profile still works as a single M1 profile.)

## A profile file (`profiles/<name>.toml`)

```toml
[keys]
G1  = "ctrl+c"
G2  = "w"
BTN1 = "space"
STICK = "enter"

[repeat]        # optional: which bindings auto-repeat while held
G2 = true

[joystick]      # optional: map the stick to WASD
mode = "wasd"   # "wasd" is implemented; "mouse" parses but is not functional yet
deadzone = 30   # 0-127; distance from center before a direction fires
up = "w"
down = "s"
left = "a"
right = "d"
```

### Bindable inputs

`G1`–`G22`, `BTN1`, `BTN2` (the two thumb buttons), `STICK` (the joystick click). Names are
case-insensitive.

### Binding syntax

The easy way: on the **Bindings** tab click **Set key** and press the key or combo — `Space`,
`Ctrl` on its own, `Ctrl+1`, `Ctrl+Shift+F5`… Press a modifier and let go of it with nothing
else to bind just that modifier (handy for push-to-talk). `Esc` or a click cancels.

By hand, a binding is optional **modifiers** + one **key**: `modifier+modifier+key`. Modifiers:
`ctrl`, `shift`, `alt`, `win`. Examples: `space`, `ctrl`, `ctrl+1`, `ctrl+shift+z`, `win+d`.
Spelling is forgiving: case doesn't matter, spaces around `+` are fine (`Ctrl + 1`), and
aliases like `spacebar`, `lctrl`, `control`, `return`, `escape`, `pgdn` are accepted. The
GUI saves bindings in tidy canonical form (`Ctrl + 1` → `ctrl+1`).

Bindings are **hold-means-hold**: the key stays down for as long as the G-key is held.
Multimedia keys are the tap-only exception.

### Valid key names

- **Letters:** `a`–`z`
- **Digits:** `0`–`9` (top row), `num0`–`num9` (numpad)
- **Function keys:** `f1`–`f24`
- **Modifiers:** `ctrl`, `shift`, `alt`, `win`; right-hand versions as plain keys: `rctrl`,
  `rshift`, `ralt`, `rwin`
- **Editing / navigation:** `enter`, `esc`, `space`, `tab`, `backspace`, `delete`, `insert`,
  `home`, `end`, `pageup`, `pagedown`, `up`, `down`, `left`, `right`, `menu`
- **Punctuation:** `minus`, `equals`, `comma`, `period`, `slash`, `semicolon`, `quote`, `hash`,
  `backslash`, `lbracket`, `rbracket`, `grave` (or the character itself, e.g. `ctrl+-`).
  These follow your keyboard layout (UK and US differ).
- **Numpad operators:** `numplus`, `numminus`, `nummultiply`, `numdivide`, `numdecimal`
- **Locks / misc:** `capslock`, `printscreen`, `pause`, `numlock`, `scrolllock`
- **Media (tap-only):** `playpause`, `nexttrack`, `prevtrack`, `mediastop`, `volup`,
  `voldown`, `mute`

An invalid key name is rejected (the GUI marks the row `bad`).

### Macros

G-keys and thumb buttons can run a sequence. Separate steps with commas; each key or combo is
tapped in turn, and `wait N` (or `delay N` / `Nms`) pauses for N milliseconds (max 60000):

```toml
[keys]
G5 = "ctrl+1, wait 150, ctrl+2, wait 150, ctrl+3"
```

A quick tap of the G-key plays the whole macro once. Tick **repeat** (or add it to `[repeat]`)
to loop the macro for as long as the G-key is held. Macros run in the background, so long waits
never block the rest of the pad. Joystick directions take a single key, not macros.

### How keys are sent

Keys are injected as real hardware **scancodes**, so games and apps that read the keyboard
directly (raw input / DirectInput) see them like a physical key. If a target app runs as
**Administrator**, Windows blocks input from a non-admin app — run g13-driver the same way.

## Auto-repeat

Holding a bound key does **not** auto-repeat by default (a held key = one character). Tick **repeat**
on a binding (GUI) or add it to `[repeat]` to make it repeat while held, like a physical keyboard.
Timing is global in `[autorepeat]` (`delay_ms`, `interval_ms`). Joystick directions and modifier-only
bindings never repeat.

## Worked example

`profiles/game.toml` — WASD on the stick, jump on a thumb button, an auto-repeating attack:

```toml
[keys]
BTN1 = "space"     # jump
G1   = "e"         # interact
G2   = "r"         # reload / attack

[repeat]
G2 = true          # hold G2 to repeat "r"

[joystick]
mode = "wasd"
deadzone = 30
up = "w"
down = "s"
left = "a"
right = "d"
```
