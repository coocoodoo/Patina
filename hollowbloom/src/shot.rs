//! Indexed PNG output for screenshots (F12) and the headless `--shots` mode.

use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

use crate::palette::PALETTE;
use crate::render::Frame;

pub fn save_png(path: &Path, fb: &Frame, scale: usize) -> io::Result<()> {
    let scale = scale.max(1);
    let (w, h) = (fb.w * scale, fb.h * scale);
    let file = File::create(path)?;
    let mut enc = png::Encoder::new(BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Indexed);
    enc.set_depth(png::BitDepth::Eight);
    let mut pal = Vec::with_capacity(PALETTE.len() * 3);
    for c in PALETTE {
        pal.extend([(c >> 16) as u8, (c >> 8) as u8, c as u8]);
    }
    enc.set_palette(pal);
    let mut writer = enc.write_header().map_err(io::Error::other)?;
    let mut data = vec![0u8; w * h];
    for y in 0..h {
        let src = &fb.color[(y / scale) * fb.w..(y / scale + 1) * fb.w];
        let dst = &mut data[y * w..(y + 1) * w];
        for (x, d) in dst.iter_mut().enumerate() {
            *d = src[x / scale].min(31);
        }
    }
    writer.write_image_data(&data).map_err(io::Error::other)?;
    writer.finish().map_err(io::Error::other)
}
