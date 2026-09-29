//! The title screen's logo: pictures made from the logo artwork by `tools/logo.py`, in the
//! game's own colours, at each size the title picks between.

use crate::palette::{CLEAR, PALETTE};

/// A picture drawn straight onto the screen, at any size (textures must be powers of two):
/// palette colours, and `CLEAR` where it's see-through.
pub struct Picture {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

impl Picture {
    /// From a PNG in the palette's colours (any other is taken as the nearest), see-through
    /// where it's mostly transparent.
    pub fn from_png(bytes: &[u8]) -> Option<Picture> {
        let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
        dec.set_transformations(png::Transformations::EXPAND);
        let mut reader = dec.read_info().ok()?;
        let mut buf = vec![0; reader.output_buffer_size()?];
        let info = reader.next_frame(&mut buf).ok()?;
        let n = match info.color_type {
            png::ColorType::Rgba => 4,
            png::ColorType::Rgb => 3,
            _ => return None,
        };
        let (w, h) = (info.width as usize, info.height as usize);
        let mut px = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                let p = &buf[y * info.line_size + x * n..][..n];
                px.push(if n == 4 && p[3] < 128 {
                    CLEAR
                } else {
                    nearest(p[0], p[1], p[2])
                });
            }
        }
        Some(Picture {
            w: w as i32,
            h: h as i32,
            px,
        })
    }
}

/// The palette colour closest to an RGB one.
fn nearest(r: u8, g: u8, b: u8) -> u8 {
    let dist = |c: u32| {
        let d = |shift: u32, v: u8| ((c >> shift) as i32 & 255) - v as i32;
        let (dr, dg, db) = (d(16, r), d(8, g), d(0, b));
        dr * dr + dg * dg + db * db
    };
    (0..PALETTE.len())
        .min_by_key(|&i| dist(PALETTE[i]))
        .unwrap_or(0) as u8
}

/// The logo, largest first.
pub fn logos() -> Vec<Picture> {
    [
        &include_bytes!("../../art/logo-150.png")[..],
        &include_bytes!("../../art/logo-116.png")[..],
        &include_bytes!("../../art/logo-96.png")[..],
    ]
    .iter()
    .filter_map(|b| Picture::from_png(b))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_logo_comes_in_three_sizes_in_the_palette() {
        let logos = logos();
        assert_eq!(
            logos.iter().map(|l| l.h).collect::<Vec<_>>(),
            [150, 116, 96]
        );
        for l in &logos {
            assert_eq!(l.px.len(), (l.w * l.h) as usize);
            // Wider than tall, see-through round the edge and solid in the middle.
            assert!(l.w > l.h * 3 / 2);
            assert_eq!(l.px[0], CLEAR);
            assert_ne!(l.px[(l.h / 2 * l.w + l.w / 2) as usize], CLEAR);
        }
        // Every colour in the files is one of the palette's, exactly.
        for b in [
            &include_bytes!("../../art/logo-150.png")[..],
            &include_bytes!("../../art/logo-116.png")[..],
            &include_bytes!("../../art/logo-96.png")[..],
        ] {
            let mut dec = png::Decoder::new(std::io::Cursor::new(b));
            dec.set_transformations(png::Transformations::EXPAND);
            let mut reader = dec.read_info().unwrap();
            let mut buf = vec![0; reader.output_buffer_size().unwrap()];
            let info = reader.next_frame(&mut buf).unwrap();
            assert_eq!(info.color_type, png::ColorType::Rgba);
            for p in buf.chunks_exact(4).filter(|p| p[3] >= 128) {
                let rgb = (p[0] as u32) << 16 | (p[1] as u32) << 8 | p[2] as u32;
                assert!(PALETTE.contains(&rgb), "#{rgb:06x} isn't in the palette");
            }
        }
    }
}
