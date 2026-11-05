//! Simple keyboard macros.
//!
//! A binding becomes a macro when it lists two or more steps separated by
//! commas, e.g.
//!
//! ```text
//! ctrl+1, wait 200, ctrl+2, wait 200, ctrl+3
//! ```
//!
//! Each step is either a key/combo (tapped: pressed then released) or a pause:
//! `wait 200`, `delay 200`, `200ms`. Steps are spaced by a small gap so games
//! register every press. Macros run on their own thread so long waits never
//! block the rest of the pad. With "repeat" ticked, the macro loops for as
//! long as the G-key is held.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use crate::injector::{KeyCombo, KeyInjector};

/// How long each tapped key is held down.
pub const TAP_HOLD_MS: u64 = 30;
/// Gap after each tapped key before the next step.
pub const STEP_GAP_MS: u64 = 40;
/// Longest single wait accepted (one minute).
pub const MAX_WAIT_MS: u64 = 60_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Tap(KeyCombo),
    Wait(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Macro {
    pub steps: Vec<Step>,
}

/// True when the binding text is meant as a macro (2+ comma-separated steps).
/// A lone "," (or "ctrl+,") is the comma key, not a macro.
pub fn is_macro(s: &str) -> bool {
    s.split(',').filter(|p| !p.trim().is_empty()).count() >= 2
}

fn parse_wait(step: &str) -> Option<Result<u64>> {
    let lower = step.trim().to_lowercase();
    let num = if let Some(rest) = lower.strip_prefix("wait").or_else(|| lower.strip_prefix("delay")) {
        rest.trim().trim_end_matches("ms").trim()
    } else if let Some(n) = lower.strip_suffix("ms") {
        n.trim()
    } else {
        return None;
    };
    Some(
        num.parse::<u64>()
            .with_context(|| format!("bad wait time in {step:?} (use e.g. \"wait 200\")"))
            .and_then(|ms| {
                if ms > MAX_WAIT_MS { bail!("wait {ms}ms is too long (max {MAX_WAIT_MS})") }
                Ok(ms)
            }),
    )
}

impl Macro {
    pub fn parse(s: &str) -> Result<Self> {
        let mut steps = Vec::new();
        for raw in s.split(',') {
            if raw.trim().is_empty() { continue; }
            match parse_wait(raw) {
                Some(ms) => steps.push(Step::Wait(ms?)),
                None => steps.push(Step::Tap(KeyCombo::parse(raw)?)),
            }
        }
        if !steps.iter().any(|s| matches!(s, Step::Tap(_))) {
            bail!("macro has no keys: {s}");
        }
        Ok(Self { steps })
    }

    /// Every tapped key is a known key name.
    pub fn keys_valid(&self, valid_keys: &HashSet<String>) -> bool {
        self.steps.iter().all(|s| match s {
            Step::Tap(c) => c.key.as_ref().is_none_or(|k| valid_keys.contains(k)),
            Step::Wait(_) => true,
        })
    }
}

/// Validity check used by the editor for any binding text (combo or macro).
pub fn binding_valid(s: &str, valid_keys: &HashSet<String>) -> bool {
    if is_macro(s) {
        Macro::parse(s).is_ok_and(|m| m.keys_valid(valid_keys))
    } else {
        match KeyCombo::parse(s) {
            Ok(c) => c.key.as_ref().is_none_or(|k| valid_keys.contains(k)),
            Err(_) => false,
        }
    }
}

/// A running macro. Dropping the handle does not stop it; call `stop()`.
pub struct MacroHandle {
    stop: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
}

impl MacroHandle {
    /// Ask a repeating macro to stop after the step it is on.
    pub fn stop(&self) { self.stop.store(true, Ordering::Relaxed); }
    pub fn is_done(&self) -> bool { self.done.load(Ordering::Relaxed) }
}

/// Sleep for `ms`, waking early if `stop` is set (only honoured when `interruptible`).
fn sleep_checked(ms: u64, stop: &AtomicBool, interruptible: bool) -> bool {
    let mut left = ms;
    while left > 0 {
        if interruptible && stop.load(Ordering::Relaxed) { return false; }
        let chunk = left.min(10);
        std::thread::sleep(Duration::from_millis(chunk));
        left -= chunk;
    }
    !(interruptible && stop.load(Ordering::Relaxed))
}

/// Run a macro on a background thread. A non-repeating macro always plays to
/// the end (a quick tap of the G-key still fires the whole sequence); a
/// repeating one loops until `stop()` is called.
pub fn spawn(m: Macro, injector: Arc<dyn KeyInjector>, repeat: bool) -> MacroHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicBool::new(false));
    let (stop_t, done_t) = (stop.clone(), done.clone());
    std::thread::spawn(move || {
        'outer: loop {
            for step in &m.steps {
                if repeat && stop_t.load(Ordering::Relaxed) { break 'outer; }
                match step {
                    Step::Tap(combo) => {
                        if let Err(e) = injector.combo_down(combo) {
                            log::warn!("macro injection failed: {e:#}");
                        }
                        std::thread::sleep(Duration::from_millis(TAP_HOLD_MS));
                        if let Err(e) = injector.combo_up(combo) {
                            log::warn!("macro injection failed: {e:#}");
                        }
                        if !sleep_checked(STEP_GAP_MS, &stop_t, repeat) { break 'outer; }
                    }
                    Step::Wait(ms) => {
                        if !sleep_checked(*ms, &stop_t, repeat) { break 'outer; }
                    }
                }
            }
            if !repeat || stop_t.load(Ordering::Relaxed) { break; }
        }
        done_t.store(true, Ordering::Relaxed);
    });
    MacroHandle { stop, done }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::injector::key_map::build_key_map;
    use std::sync::Mutex;
    use std::time::Instant;

    fn keys() -> HashSet<String> { build_key_map().into_keys().collect() }

    #[test]
    fn detects_macros() {
        assert!(is_macro("ctrl+1, ctrl+2"));
        assert!(is_macro("a,wait 100,b"));
        assert!(!is_macro("ctrl+1"));
        assert!(!is_macro(","));
        assert!(!is_macro("ctrl+,"));
        assert!(!is_macro("space"));
    }

    #[test]
    fn parses_steps_and_waits() {
        let m = Macro::parse("ctrl+1, wait 200, 50ms, delay 5, Space").unwrap();
        assert_eq!(m.steps.len(), 5);
        assert_eq!(m.steps[1], Step::Wait(200));
        assert_eq!(m.steps[2], Step::Wait(50));
        assert_eq!(m.steps[3], Step::Wait(5));
        assert!(matches!(&m.steps[4], Step::Tap(c) if c.key.as_deref() == Some("space")));
    }

    #[test]
    fn rejects_bad_macros() {
        assert!(Macro::parse("wait 100, wait 200").is_err());
        assert!(Macro::parse("a, wait abc").is_err());
        assert!(Macro::parse("a, wait 999999").is_err());
        assert!(!binding_valid("a, zzz", &keys()));
    }

    #[test]
    fn binding_valid_covers_plain_and_macro() {
        let k = keys();
        for ok in ["space", "ctrl", "ctrl+1", "Ctrl + Space", "1, 2, 3", "ctrl+1, wait 100, ctrl+2"] {
            assert!(binding_valid(ok, &k), "{ok}");
        }
        for bad in ["zzz", "ctrl+zzz", "a+b", "a, wait x"] {
            assert!(!binding_valid(bad, &k), "{bad}");
        }
    }

    #[derive(Default)]
    struct Rec(Mutex<Vec<String>>);
    impl KeyInjector for Rec {
        fn press(&self, c: &KeyCombo) -> anyhow::Result<()> { self.0.lock().unwrap().push(format!("press:{}", c.to_canonical())); Ok(()) }
        fn key_down(&self, k: &str) -> anyhow::Result<()> { self.0.lock().unwrap().push(format!("kd:{k}")); Ok(()) }
        fn key_up(&self, k: &str) -> anyhow::Result<()> { self.0.lock().unwrap().push(format!("ku:{k}")); Ok(()) }
        fn combo_down(&self, c: &KeyCombo) -> anyhow::Result<()> { self.0.lock().unwrap().push(format!("down:{}", c.to_canonical())); Ok(()) }
        fn combo_up(&self, c: &KeyCombo) -> anyhow::Result<()> { self.0.lock().unwrap().push(format!("up:{}", c.to_canonical())); Ok(()) }
    }

    fn wait_done(h: &MacroHandle) {
        let t = Instant::now();
        while !h.is_done() {
            assert!(t.elapsed() < Duration::from_secs(5), "macro never finished");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn runs_once_in_order() {
        let rec = Arc::new(Rec::default());
        let m = Macro::parse("ctrl+1, wait 20, ctrl+2").unwrap();
        let start = Instant::now();
        let h = spawn(m, rec.clone(), false);
        h.stop(); // non-repeating macros ignore stop and finish
        wait_done(&h);
        assert!(start.elapsed() >= Duration::from_millis(20));
        assert_eq!(*rec.0.lock().unwrap(), vec!["down:ctrl+1", "up:ctrl+1", "down:ctrl+2", "up:ctrl+2"]);
    }

    #[test]
    fn repeating_macro_loops_until_stopped() {
        let rec = Arc::new(Rec::default());
        let h = spawn(Macro::parse("a, b").unwrap(), rec.clone(), true);
        // Generous window: Windows timer granularity (~15ms) stretches each step.
        std::thread::sleep(Duration::from_millis(800));
        h.stop();
        wait_done(&h);
        let log = rec.0.lock().unwrap().clone();
        assert!(log.len() >= 6, "expected at least a few loops, got {log:?}");
        // Never left a key held: every down has its up.
        let downs = log.iter().filter(|s| s.starts_with("down:")).count();
        let ups = log.iter().filter(|s| s.starts_with("up:")).count();
        assert_eq!(downs, ups);
    }
}
