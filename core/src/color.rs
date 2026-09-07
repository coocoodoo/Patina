//! Colors: straight-alpha RGBA in 0..1 plus the theme-token indirection used by the ABI.

use crate::props::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const TRANSPARENT: Rgba = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const WHITE: Rgba = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Rgba = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Rgba { r, g, b, a }
    }

    /// `0xRRGGBB`, opaque.
    pub const fn hex(rgb: u32) -> Rgba {
        Rgba {
            r: ((rgb >> 16) & 0xff) as f32 / 255.0,
            g: ((rgb >> 8) & 0xff) as f32 / 255.0,
            b: (rgb & 0xff) as f32 / 255.0,
            a: 1.0,
        }
    }

    /// `0xRRGGBB` with an explicit alpha.
    pub const fn hexa(rgb: u32, a: f32) -> Rgba {
        let mut c = Rgba::hex(rgb);
        c.a = a;
        c
    }

    /// `0xRRGGBBAA` as used by the ABI.
    pub fn from_rgba_u32(v: u32) -> Rgba {
        Rgba {
            r: ((v >> 24) & 0xff) as f32 / 255.0,
            g: ((v >> 16) & 0xff) as f32 / 255.0,
            b: ((v >> 8) & 0xff) as f32 / 255.0,
            a: (v & 0xff) as f32 / 255.0,
        }
    }

    pub fn to_rgba_u32(self) -> u32 {
        let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
        (c(self.r) << 24) | (c(self.g) << 16) | (c(self.b) << 8) | c(self.a)
    }

    pub fn with_alpha(mut self, a: f32) -> Rgba {
        self.a = a;
        self
    }

    pub fn mul_alpha(mut self, f: f32) -> Rgba {
        self.a *= f;
        self
    }

    pub fn mix(self, o: Rgba, t: f32) -> Rgba {
        let t = t.clamp(0.0, 1.0);
        Rgba {
            r: self.r + (o.r - self.r) * t,
            g: self.g + (o.g - self.g) * t,
            b: self.b + (o.b - self.b) * t,
            a: self.a + (o.a - self.a) * t,
        }
    }

    pub fn lighten(self, t: f32) -> Rgba {
        let a = self.a;
        self.mix(Rgba::WHITE, t).with_alpha(a)
    }

    pub fn darken(self, t: f32) -> Rgba {
        let a = self.a;
        self.mix(Rgba::BLACK, t).with_alpha(a)
    }

    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    pub fn is_visible(&self) -> bool {
        self.a > 0.001
    }

    pub fn to_skia(self) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
            self.a.clamp(0.0, 1.0),
        )
        .unwrap_or(tiny_skia::Color::TRANSPARENT)
    }
}

/// A color as stored on a node: either "use the widget default", a literal, or a theme token.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ColorRef {
    #[default]
    Default,
    Literal(Rgba),
    Token(i64),
}

impl ColorRef {
    /// ABI encoding: negative values are tokens, non-negative values are `0xRRGGBBAA`.
    pub fn from_i64(v: i64) -> ColorRef {
        if v == COLOR_DEFAULT {
            ColorRef::Default
        } else if v < 0 {
            ColorRef::Token(v)
        } else {
            ColorRef::Literal(Rgba::from_rgba_u32((v as u64 & 0xffff_ffff) as u32))
        }
    }

    pub fn to_i64(self) -> i64 {
        match self {
            ColorRef::Default => COLOR_DEFAULT,
            ColorRef::Token(t) => t,
            ColorRef::Literal(c) => c.to_rgba_u32() as i64,
        }
    }

    pub fn is_default(&self) -> bool {
        matches!(self, ColorRef::Default)
    }
}
