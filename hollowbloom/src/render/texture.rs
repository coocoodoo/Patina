//! Palette-indexed textures. Sizes are powers of two so sampling can wrap with a mask.

use crate::palette::CLEAR;

#[derive(Clone, Debug)]
pub struct Texture {
    pub w: u32,
    pub h: u32,
    pub wshift: u32,
    pub data: Vec<u8>,
}

impl Texture {
    pub fn new(w: u32, h: u32, fill: u8) -> Self {
        assert!(
            w.is_power_of_two() && h.is_power_of_two(),
            "texture sizes must be powers of two"
        );
        Texture {
            w,
            h,
            wshift: w.trailing_zeros(),
            data: vec![fill; (w * h) as usize],
        }
    }

    pub fn clear(w: u32, h: u32) -> Self {
        Self::new(w, h, CLEAR)
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> u8 {
        let xi = (x as u32) & (self.w - 1);
        let yi = (y as u32) & (self.h - 1);
        self.data[((yi << self.wshift) | xi) as usize]
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: u8) {
        if x >= 0 && y >= 0 && (x as u32) < self.w && (y as u32) < self.h {
            self.data[((y as u32) << self.wshift | x as u32) as usize] = c;
        }
    }

    /// Sets a pixel with wrap-around, for seamless tiling textures.
    #[inline]
    pub fn set_wrap(&mut self, x: i32, y: i32, c: u8) {
        let xi = (x as u32) & (self.w - 1);
        let yi = (y as u32) & (self.h - 1);
        self.data[((yi << self.wshift) | xi) as usize] = c;
    }

    pub fn map_colors(&self, f: impl Fn(u8) -> u8) -> Texture {
        let mut t = self.clone();
        for p in &mut t.data {
            if *p != CLEAR {
                *p = f(*p);
            }
        }
        t
    }

    /// Builds a texture from ASCII art rows and a legend of `(char, palette index)` pairs.
    /// Characters missing from the legend (and `.`) are transparent.
    pub fn from_art(rows: &[&str], legend: &[(char, u8)]) -> Texture {
        let h = rows.len() as u32;
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(1) as u32;
        let mut t = Texture::clear(w.next_power_of_two(), h.next_power_of_two());
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if let Some(&(_, c)) = legend.iter().find(|(k, _)| *k == ch) {
                    t.set(x as i32, y as i32, c);
                }
            }
        }
        t
    }
}

pub type TexId = u16;

#[derive(Default)]
pub struct TexBank {
    list: Vec<Texture>,
}

impl TexBank {
    pub fn add(&mut self, t: Texture) -> TexId {
        self.list.push(t);
        (self.list.len() - 1) as TexId
    }

    #[inline]
    pub fn get(&self, id: TexId) -> &Texture {
        &self.list[id as usize]
    }
}
