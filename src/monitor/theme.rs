//! App-wide look: a calm dark theme with an amber accent (a nod to the G13's
//! backlight), roomier spacing and rounded controls.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Vec2};

pub const BG: Color32 = Color32::from_rgb(0x15, 0x17, 0x1c);
pub const PANEL: Color32 = Color32::from_rgb(0x1b, 0x1e, 0x25);
pub const CARD: Color32 = Color32::from_rgb(0x22, 0x26, 0x2f);
pub const CARD_HOVER: Color32 = Color32::from_rgb(0x2b, 0x30, 0x3b);
pub const BORDER: Color32 = Color32::from_rgb(0x30, 0x35, 0x41);
pub const TEXT: Color32 = Color32::from_rgb(0xe6, 0xe8, 0xee);
pub const MUTED: Color32 = Color32::from_rgb(0x8b, 0x92, 0xa3);
pub const ACCENT: Color32 = Color32::from_rgb(0xf5, 0xa6, 0x23);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(0x5a, 0x40, 0x14);
pub const GOOD: Color32 = Color32::from_rgb(0x5f, 0xd6, 0x8a);
pub const BAD: Color32 = Color32::from_rgb(0xe8, 0x5d, 0x5d);

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = [
        (TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.5, FontFamily::Monospace)),
        (TextStyle::Small, FontId::new(11.5, FontFamily::Proportional)),
    ].into();

    let s = &mut style.spacing;
    s.item_spacing = Vec2::new(8.0, 7.0);
    s.button_padding = Vec2::new(10.0, 4.0);
    s.interact_size.y = 24.0;

    let v = &mut style.visuals;
    *v = egui::Visuals::dark();
    v.override_text_color = None;
    v.panel_fill = BG;
    v.window_fill = PANEL;
    v.window_stroke = Stroke::new(1.0_f32, BORDER);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    v.extreme_bg_color = Color32::from_rgb(0x10, 0x12, 0x16); // text-edit background
    v.faint_bg_color = Color32::from_rgb(0x1d, 0x20, 0x27);   // grid stripes
    v.code_bg_color = CARD;
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = ACCENT_DIM;
    v.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    v.striped = true;

    let r = CornerRadius::same(4);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = PANEL;
    w.noninteractive.weak_bg_fill = PANEL;
    w.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.noninteractive.corner_radius = r;
    w.inactive.bg_fill = CARD;
    w.inactive.weak_bg_fill = CARD;
    w.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    w.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.inactive.corner_radius = r;
    w.hovered.bg_fill = CARD_HOVER;
    w.hovered.weak_bg_fill = CARD_HOVER;
    w.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT_DIM);
    w.hovered.fg_stroke = Stroke::new(1.5_f32, TEXT);
    w.hovered.corner_radius = r;
    w.hovered.expansion = 0.0;
    w.active.bg_fill = ACCENT_DIM;
    w.active.weak_bg_fill = ACCENT_DIM;
    w.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    w.active.fg_stroke = Stroke::new(1.5_f32, TEXT);
    w.active.corner_radius = r;
    w.active.expansion = 0.0;
    w.open = w.active;

    ctx.set_style(style);
}

/// A rounded card frame for grouping a section.
pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::same(10))
}

/// Small coloured status pill ("ok" / "bad" / "macro").
pub fn pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.18))
        .corner_radius(CornerRadius::same(9))
        .inner_margin(egui::Margin::symmetric(7, 1))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).small().color(color));
        });
}

/// Section title in the accent colour.
pub fn section(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).strong().color(ACCENT));
}
