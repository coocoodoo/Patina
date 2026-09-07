//! Design tokens: named palettes, the active light/dark pair, accent overrides and animated
//! transitions between palettes.

use crate::anim::ease_in_out;
use crate::color::{ColorRef, Rgba};
use crate::palettes;
use crate::props::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Light,
    Dark,
    System,
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub is_dark: bool,
    pub background: Rgba,
    pub surface: Rgba,
    pub surface_variant: Rgba,
    pub text: Rgba,
    pub text_secondary: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
    pub on_accent: Rgba,
    pub outline: Rgba,
    pub outline_strong: Rgba,
    pub danger: Rgba,
    pub success: Rgba,
    pub warning: Rgba,
    pub shadow: Rgba,
}

impl Palette {
    /// Order of the colors in the ABI's `patina_theme_define` array.
    pub const FIELD_COUNT: usize = 14;

    pub fn from_colors(is_dark: bool, c: &[Rgba]) -> Option<Palette> {
        if c.len() < Self::FIELD_COUNT {
            return None;
        }
        Some(Palette {
            is_dark,
            background: c[0],
            surface: c[1],
            surface_variant: c[2],
            text: c[3],
            text_secondary: c[4],
            text_muted: c[5],
            accent: c[6],
            on_accent: c[7],
            outline: c[8],
            outline_strong: c[9],
            danger: c[10],
            success: c[11],
            warning: c[12],
            shadow: c[13],
        })
    }

    /// Resolve a node color against this palette.
    pub fn resolve(&self, c: ColorRef, default: Rgba) -> Rgba {
        match c {
            ColorRef::Default => default,
            ColorRef::Literal(v) => v,
            ColorRef::Token(t) => self.token(t).unwrap_or(default),
        }
    }

    pub fn token(&self, t: i64) -> Option<Rgba> {
        Some(match t {
            COLOR_BACKGROUND => self.background,
            COLOR_SURFACE => self.surface,
            COLOR_SURFACE_VARIANT => self.surface_variant,
            COLOR_TEXT => self.text,
            COLOR_TEXT_SECONDARY => self.text_secondary,
            COLOR_TEXT_MUTED => self.text_muted,
            COLOR_ACCENT => self.accent,
            COLOR_ON_ACCENT => self.on_accent,
            COLOR_OUTLINE => self.outline,
            COLOR_DANGER => self.danger,
            COLOR_SUCCESS => self.success,
            COLOR_WARNING => self.warning,
            _ => return None,
        })
    }

    /// Shadow color, blur radius and vertical offset (logical px) for elevation `level` (1..=3).
    pub fn shadow(&self, level: i64) -> Option<(Rgba, f32, f32)> {
        let boost = if self.is_dark { 2.2 } else { 1.0 };
        Some(match level {
            1 => (self.shadow.with_alpha(0.07 * boost), 10.0, 2.0),
            2 => (self.shadow.with_alpha(0.11 * boost), 20.0, 6.0),
            3 => (self.shadow.with_alpha(0.16 * boost), 34.0, 12.0),
            _ => return None,
        })
    }

    /// Blend every color towards `o` (0 = self, 1 = o).
    pub fn mix(&self, o: &Palette, t: f32) -> Palette {
        Palette {
            is_dark: if t < 0.5 { self.is_dark } else { o.is_dark },
            background: self.background.mix(o.background, t),
            surface: self.surface.mix(o.surface, t),
            surface_variant: self.surface_variant.mix(o.surface_variant, t),
            text: self.text.mix(o.text, t),
            text_secondary: self.text_secondary.mix(o.text_secondary, t),
            text_muted: self.text_muted.mix(o.text_muted, t),
            accent: self.accent.mix(o.accent, t),
            on_accent: self.on_accent.mix(o.on_accent, t),
            outline: self.outline.mix(o.outline, t),
            outline_strong: self.outline_strong.mix(o.outline_strong, t),
            danger: self.danger.mix(o.danger, t),
            success: self.success.mix(o.success, t),
            warning: self.warning.mix(o.warning, t),
            shadow: self.shadow.mix(o.shadow, t),
        }
    }

    pub fn with_accent(mut self, a: Rgba) -> Palette {
        // Keep the accent readable on dark surfaces.
        self.accent = if self.is_dark && a.luminance() < 0.35 {
            a.lighten(0.18)
        } else {
            a
        };
        self.on_accent = if self.accent.luminance() > 0.62 {
            Rgba::hex(0x14161C)
        } else {
            Rgba::WHITE
        };
        self
    }
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub mode: Mode,
    pub accent: Option<Rgba>,
    /// Whether the OS reports a dark appearance (used when `mode == System`).
    pub system_dark: bool,
    pub light: Palette,
    pub light_name: String,
    pub dark: Palette,
    pub dark_name: String,
    /// Palettes registered by the host.
    pub custom: Vec<(String, Palette)>,
    /// Seconds a palette cross-fade takes.
    pub transition_secs: f64,
    /// The palette being faded out and the clock time the fade started.
    transition: Option<(Palette, f64)>,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            mode: Mode::Light,
            accent: None,
            system_dark: false,
            light: palettes::INDIGO,
            light_name: "Indigo".to_string(),
            dark: palettes::MIDNIGHT,
            dark_name: "Midnight".to_string(),
            custom: Vec::new(),
            transition_secs: 0.28,
            transition: None,
        }
    }
}

impl Theme {
    pub fn is_dark(&self) -> bool {
        match self.mode {
            Mode::Light => false,
            Mode::Dark => true,
            Mode::System => self.system_dark,
        }
    }

    /// Name of the palette currently in effect.
    pub fn current_name(&self) -> &str {
        if self.is_dark() {
            &self.dark_name
        } else {
            &self.light_name
        }
    }

    fn target(&self) -> Palette {
        let p = if self.is_dark() {
            self.dark
        } else {
            self.light
        };
        match self.accent {
            Some(a) => p.with_accent(a),
            None => p,
        }
    }

    /// The palette to use right now, ignoring any running transition.
    pub fn palette(&self) -> Palette {
        self.target()
    }

    /// The palette at animation-clock time `clock`, blending a running transition.
    pub fn palette_at(&self, clock: f64) -> Palette {
        let target = self.target();
        match &self.transition {
            Some((from, start)) => {
                let t = if self.transition_secs <= 0.0 {
                    1.0
                } else {
                    ((clock - start) / self.transition_secs).clamp(0.0, 1.0) as f32
                };
                if t >= 1.0 {
                    target
                } else {
                    from.mix(&target, ease_in_out(t))
                }
            }
            None => target,
        }
    }

    /// Finish any running cross-fade immediately.
    pub fn settle(&mut self) {
        self.transition = None;
    }

    pub fn transitioning(&self, clock: f64) -> bool {
        match &self.transition {
            Some((_, start)) => clock - start < self.transition_secs,
            None => false,
        }
    }

    /// Start cross-fading from whatever is on screen at `clock` to the (new) target.
    pub fn begin_transition(&mut self, clock: f64) {
        let from = self.palette_at(clock);
        self.transition = Some((from, clock));
    }

    pub fn set_mode(&mut self, mode: Mode, clock: f64) {
        self.begin_transition(clock);
        self.mode = mode;
    }

    pub fn set_system_dark(&mut self, dark: bool, clock: f64) {
        if self.system_dark != dark {
            self.begin_transition(clock);
            self.system_dark = dark;
        }
    }

    pub fn set_accent(&mut self, accent: Option<Rgba>, clock: f64) {
        self.begin_transition(clock);
        self.accent = accent;
    }

    fn find(&self, name: &str) -> Option<(String, Palette)> {
        let want = name.trim().to_ascii_lowercase();
        if let Some((n, p)) = self
            .custom
            .iter()
            .find(|(n, _)| n.to_ascii_lowercase() == want)
        {
            return Some((n.clone(), *p));
        }
        palettes::BUILTIN
            .iter()
            .find(|(n, _)| n.to_ascii_lowercase() == want)
            .map(|(n, p)| (n.to_string(), *p))
    }

    /// Activate a palette by name: it becomes the light or dark palette according to its
    /// kind, and the mode switches to show it. Returns false for unknown names.
    pub fn use_palette(&mut self, name: &str, clock: f64) -> bool {
        let Some((name, p)) = self.find(name) else {
            return false;
        };
        self.begin_transition(clock);
        if p.is_dark {
            self.dark = p;
            self.dark_name = name;
            self.mode = Mode::Dark;
        } else {
            self.light = p;
            self.light_name = name;
            self.mode = Mode::Light;
        }
        true
    }

    /// Register (or replace) a host-defined palette.
    pub fn define(&mut self, name: &str, p: Palette) {
        let key = name.trim().to_string();
        if key.is_empty() {
            return;
        }
        match self
            .custom
            .iter_mut()
            .find(|(n, _)| n.eq_ignore_ascii_case(&key))
        {
            Some(slot) => slot.1 = p,
            None => self.custom.push((key, p)),
        }
    }

    /// Every known palette: built-ins first, then custom ones.
    pub fn list(&self) -> Vec<(String, Palette)> {
        let mut out: Vec<(String, Palette)> = palettes::BUILTIN
            .iter()
            .map(|(n, p)| (n.to_string(), *p))
            .collect();
        out.extend(self.custom.iter().cloned());
        out
    }
}
