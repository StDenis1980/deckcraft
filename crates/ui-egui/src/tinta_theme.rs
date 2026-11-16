//! Tinta Office: the Tinta look, the colours and control shapes of Tinta Correo carried over to
//! Tinta Presenta. The window is a soft grey with a warm wash of the app's colour; the ribbon,
//! panes, menus and dialogs are cards; buttons have no frame, only a tint of the accent that
//! deepens under the pointer; corners are rounder. The accent is the ochre of the Tinta Presenta
//! icon, a shade deeper so white text reads on it.
//!
//! Only the chrome changes, never the layout or what is drawn on the slide (selection, handles,
//! guides, caret). Off (View › Tinta look, `view.tintaLook`), the interface is upstream's classic
//! palette. The setting is read through [`on`], which the app sets each frame from its
//! preferences; [`skin`] is the pure mapping, so tests never touch the global switch.

use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Color32, CornerRadius, Stroke, Visuals};

use crate::theme::Tokens;

/// Off until the app reads its setting.
static ON: AtomicBool = AtomicBool::new(false);

/// Whether the Tinta look is on. Never in this crate's unit tests: they share one process, and
/// upstream's compare colours with the classic palette whatever other tests switch on.
pub fn on() -> bool {
    !cfg!(test) && allowed() && ON.load(Ordering::Relaxed)
}

/// Set by the desktop app ([`allow`]): only it shows the Tinta look. Upstream's integration tests
/// and examples compare colours with the classic palette, so they keep it, unless `TINTA_LOOK` is
/// set in the environment (for screenshots of the Tinta look).
static ALLOWED: AtomicBool = AtomicBool::new(false);

/// Let this program show the Tinta look (the desktop app calls it once, at start).
pub fn allow() {
    ALLOWED.store(true, Ordering::Relaxed);
}

fn allowed() -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    ALLOWED.load(Ordering::Relaxed) || *ENV.get_or_init(|| std::env::var_os("TINTA_LOOK").is_some())
}

/// Turn the Tinta look on or off; true when that changed it, so the caller re-applies the theme.
/// Never in unit tests, where re-applying the theme mid-test would change their frames.
pub fn set(on: bool) -> bool {
    !cfg!(test) && ON.swap(on, Ordering::Relaxed) != on
}

/// The palette in use: [`skin`] of `classic` while the Tinta look is on, else `classic` itself.
pub fn tokens(classic: Tokens) -> Tokens {
    if on() { skin(classic) } else { classic }
}

/// The Tinta palette, light or dark: Tinta Correo's neutrals, warmed, with the Presenta ochre.
struct Palette {
    accent: Color32,
    window: Color32,
    band: Color32,
    card: Color32,
    text: Color32,
    text_weak: Color32,
    border: Color32,
    on_accent: Color32,
}

const LIGHT: Palette = Palette {
    accent: Color32::from_rgb(0x9A, 0x58, 0x0C),
    window: Color32::from_rgb(0xF3, 0xF0, 0xEB),
    band: Color32::from_rgb(0xF2, 0xE1, 0xC9),
    card: Color32::WHITE,
    text: Color32::from_rgb(0x1F, 0x24, 0x30),
    text_weak: Color32::from_rgb(0x5E, 0x5D, 0x62),
    border: Color32::from_rgb(0xE6, 0xE1, 0xDA),
    on_accent: Color32::WHITE,
};

const DARK: Palette = Palette {
    accent: Color32::from_rgb(0xE8, 0xA9, 0x55),
    window: Color32::from_rgb(0x1D, 0x1A, 0x16),
    band: Color32::from_rgb(0x33, 0x29, 0x1C),
    card: Color32::from_rgb(0x29, 0x26, 0x21),
    text: Color32::from_rgb(0xEC, 0xE8, 0xDF),
    text_weak: Color32::from_rgb(0xB8, 0xB3, 0xA8),
    border: Color32::from_rgb(0x44, 0x3F, 0x37),
    on_accent: Color32::from_rgb(0x1A, 0x12, 0x08),
};

/// `a` moved `share` (0 to 1) of the way to `b`.
pub fn mix(a: Color32, b: Color32, share: f32) -> Color32 {
    let s = share.clamp(0.0, 1.0);
    let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * s).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(ch(a.r(), b.r()), ch(a.g(), b.g()), ch(a.b(), b.b()))
}

/// The Tinta look of `classic`: the chrome takes the Tinta palette; what is drawn on the slide
/// keeps upstream's colours.
pub fn skin(classic: Tokens) -> Tokens {
    let p = if classic.dark { DARK } else { LIGHT };
    let dark = classic.dark;
    Tokens {
        chrome: mix(p.window, p.band, 0.45),
        card: p.card,
        card_border: p.border,
        text_field: if dark { mix(p.window, Color32::BLACK, 0.45) } else { mix(p.window, p.text_weak, 0.12) },
        canvas: if dark { mix(p.window, Color32::BLACK, 0.25) } else { mix(p.window, p.text_weak, 0.06) },
        text: p.text,
        text_dim: p.text_weak,
        text_faint: mix(p.text_weak, p.card, 0.35),
        border: mix(p.border, p.text_weak, 0.2),
        separator: p.border,
        hover: mix(p.card, p.accent, if dark { 0.18 } else { 0.10 }),
        pressed: mix(p.card, p.accent, if dark { 0.28 } else { 0.20 }),
        selected: mix(p.card, p.accent, if dark { 0.24 } else { 0.14 }),
        accent: p.accent,
        accent_text: p.on_accent,
        contextual: if dark { mix(p.accent, Color32::WHITE, 0.15) } else { mix(p.accent, Color32::BLACK, 0.1) },
        field_bg: if dark { mix(p.card, p.window, 0.5) } else { p.card },
        control: if dark { mix(p.card, Color32::WHITE, 0.14) } else { mix(p.card, p.text_weak, 0.30) },
        control_hover: if dark { mix(p.card, Color32::WHITE, 0.22) } else { mix(p.card, p.text_weak, 0.38) },
        control_pressed: if dark { mix(p.card, Color32::WHITE, 0.28) } else { mix(p.card, p.text_weak, 0.46) },
        ..classic
    }
}

/// Tinta Correo's control shapes on egui's visuals for `t` ([`crate::theme::apply`] fills in the
/// colours first): rounder corners, a soft accent tint on buttons, softer shadows.
pub fn visuals(v: &mut Visuals, t: &Tokens) {
    if !on() {
        return;
    }
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(8);
    v.selection.bg_fill = mix(t.card, t.accent, if t.dark { 0.45 } else { 0.30 });
    v.selection.stroke = Stroke::new(1.0, t.accent);
    v.widgets.inactive.weak_bg_fill = mix(t.card, t.accent, 0.05);
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(6);
    }
    v.popup_shadow = egui::Shadow { offset: [0, 4], blur: 14, spread: 0, color: Color32::from_black_alpha(if t.dark { 100 } else { 32 }) };
    v.window_shadow = egui::Shadow { offset: [0, 6], blur: 24, spread: 0, color: Color32::from_black_alpha(if t.dark { 110 } else { 40 }) };
}

/// The View tab's button.
pub(crate) const BUTTON: &str = "Tinta\nlook";

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG contrast ratio between two opaque colours.
    fn contrast(a: Color32, b: Color32) -> f32 {
        let lum = |c: Color32| {
            let f = |v: u8| {
                let v = f32::from(v) / 255.0;
                if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
        };
        let (x, y) = (lum(a), lum(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    fn both() -> [Tokens; 2] {
        [skin(Tokens::light()), skin(Tokens::dark())]
    }

    #[test]
    fn the_tinta_look_keeps_text_and_controls_readable() {
        for t in both() {
            let dark = t.dark;
            for bg in [t.chrome, t.card, t.text_field, t.field_bg] {
                assert!(contrast(t.text, bg) >= 7.0, "text on {bg:?} (dark {dark})");
                assert!(contrast(t.text_dim, bg) >= 4.5, "weak text on {bg:?} (dark {dark})");
            }
            for bg in [t.hover, t.pressed, t.selected] {
                assert!(contrast(t.text, bg) >= 4.5, "text on a hovered or chosen control {bg:?} (dark {dark})");
            }
            assert!(contrast(t.accent_text, t.accent) >= 4.5, "text on the accent (dark {dark})");
            assert!(contrast(t.accent, t.card) >= 3.0, "accent marks on cards (dark {dark})");
            assert!(contrast(t.accent, t.chrome) >= 3.0, "accent marks on the window (dark {dark})");
        }
    }

    /// The same promise upstream keeps for its palette (theme.rs): unchecked boxes, slider rails
    /// and text fields have no outline, so their fill sets them apart.
    #[test]
    fn controls_without_an_outline_stand_out() {
        for t in both() {
            for bg in [t.card, t.chrome] {
                assert!(contrast(t.control, bg) >= 1.2, "control on {bg:?} (dark {})", t.dark);
                assert!(contrast(t.text_field, bg) >= 1.08, "field on {bg:?} (dark {})", t.dark);
            }
        }
    }

    #[test]
    fn the_slide_keeps_upstreams_colours() {
        for classic in [Tokens::light(), Tokens::dark()] {
            let t = skin(classic);
            assert_eq!(
                (t.selection, t.handle_fill, t.handle_stroke, t.guide, t.caret, t.text_selection),
                (classic.selection, classic.handle_fill, classic.handle_stroke, classic.guide, classic.caret, classic.text_selection)
            );
        }
    }

    #[test]
    fn the_setting_is_on_by_default() {
        assert!(crate::state::UiState::default().tinta_look);
        let saved: crate::state::UiState = serde_json::from_str(r#"{"tintaLook": false}"#).unwrap();
        assert!(!saved.tinta_look);
    }
}
