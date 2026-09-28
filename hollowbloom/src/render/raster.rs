//! Triangle rasterizer: clipping, perspective-correct texturing, depth testing and
//! palette lighting with ordered dithering. Everything it writes is a palette index.

use glam::Vec4;

use super::frame::Frame;
use super::texture::Texture;
use crate::palette::{BAYER, BLENDS, CLEAR, LEVELS, Shading, WARMTHS};

/// A vertex in clip space with its attributes.
#[derive(Clone, Copy, Debug, Default)]
pub struct CVert {
    pub p: Vec4,
    /// Texture coordinates in texels.
    pub u: f32,
    pub v: f32,
    /// Light level (0..LEVELS, 16 = texture colour).
    pub l: f32,
    /// Warmth step (0..WARMTHS-1).
    pub t: f32,
    /// Opacity for glass and glow (0 clear .. `BLENDS + 1` solid).
    pub a: f32,
}

impl CVert {
    fn lerp(&self, o: &CVert, k: f32) -> CVert {
        CVert {
            p: self.p + (o.p - self.p) * k,
            u: self.u + (o.u - self.u) * k,
            v: self.v + (o.v - self.v) * k,
            l: self.l + (o.l - self.l) * k,
            t: self.t + (o.t - self.t) * k,
            a: self.a + (o.a - self.a) * k,
        }
    }
}

/// Converts an opacity in 0..1 to the vertex `a` scale.
pub fn opacity(k: f32) -> f32 {
    k.clamp(0.0, 1.0) * (BLENDS + 1) as f32
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Texture colours through the light maps.
    Lit,
    /// Texture colours as they are (flames, glows).
    Unlit,
    /// Every opaque texel becomes one colour (hit flashes).
    Solid(u8),
    /// Darkens what is already in the framebuffer (blob shadows), using `darken[k]`.
    Darken(u8),
    /// Only where the triangle is hidden, as a checkerboard of one colour (see-through
    /// silhouettes).
    Hidden(u8),
    /// Lit texture colours laid over what is already drawn, like tinted jelly or glass,
    /// with the vertex opacity dithered across the surface.
    Glass,
    /// Texture colours added to what is already drawn as light (glows, sparks, halos).
    Glow,
}

#[derive(Clone, Copy)]
pub struct Mat<'a> {
    pub tex: &'a Texture,
    pub mode: Mode,
    pub cull: bool,
    pub zwrite: bool,
    pub tag: u8,
    /// Draws only every other pixel (checkerboard transparency).
    pub screen_door: bool,
}

#[cfg(test)]
impl<'a> Mat<'a> {
    pub fn lit(tex: &'a Texture) -> Self {
        Mat {
            tex,
            mode: Mode::Lit,
            cull: true,
            zwrite: true,
            tag: 0,
            screen_door: false,
        }
    }
}

const GUARD: f32 = 4.0;

fn outcode(c: &CVert) -> u8 {
    let mut o = 0;
    if c.p.z < 0.0 {
        o |= 1;
    }
    if c.p.x < -GUARD * c.p.w {
        o |= 2;
    }
    if c.p.x > GUARD * c.p.w {
        o |= 4;
    }
    if c.p.y < -GUARD * c.p.w {
        o |= 8;
    }
    if c.p.y > GUARD * c.p.w {
        o |= 16;
    }
    o
}

fn plane_dist(c: &CVert, plane: usize) -> f32 {
    match plane {
        0 => c.p.z,
        1 => c.p.x + GUARD * c.p.w,
        2 => GUARD * c.p.w - c.p.x,
        3 => c.p.y + GUARD * c.p.w,
        _ => GUARD * c.p.w - c.p.y,
    }
}

/// Clips and rasterizes one triangle.
pub fn draw_tri(fb: &mut Frame, sh: &Shading, v: &[CVert; 3], m: &Mat) {
    let (a, b, c) = (outcode(&v[0]), outcode(&v[1]), outcode(&v[2]));
    if a & b & c != 0 {
        return;
    }
    if a | b | c == 0 {
        raster(fb, sh, &v[0], &v[1], &v[2], m);
        return;
    }
    let mut buf_a = [CVert::default(); 10];
    let mut buf_b = [CVert::default(); 10];
    buf_a[..3].copy_from_slice(v);
    let mut n = 3;
    let all = a | b | c;
    for plane in 0..5 {
        if all & (1 << plane) == 0 {
            continue;
        }
        let mut out = 0;
        for i in 0..n {
            let cur = &buf_a[i];
            let nxt = &buf_a[(i + 1) % n];
            let dc = plane_dist(cur, plane);
            let dn = plane_dist(nxt, plane);
            if dc >= 0.0 {
                buf_b[out] = *cur;
                out += 1;
            }
            if (dc >= 0.0) != (dn >= 0.0) {
                let k = dc / (dc - dn);
                buf_b[out] = cur.lerp(nxt, k);
                out += 1;
            }
        }
        n = out;
        if n < 3 {
            return;
        }
        buf_a[..n].copy_from_slice(&buf_b[..n]);
    }
    for i in 1..n - 1 {
        raster(fb, sh, &buf_a[0], &buf_a[i], &buf_a[i + 1], m);
    }
}

#[derive(Clone, Copy)]
struct SVert {
    x: f64,
    y: f64,
    iw: f32,
    uw: f32,
    vw: f32,
    l: f32,
    t: f32,
    a: f32,
}

#[inline]
fn to_screen(c: &CVert, w: f32, h: f32) -> SVert {
    let iw = 1.0 / c.p.w;
    let x = (c.p.x * iw * 0.5 + 0.5) * w;
    let y = (0.5 - c.p.y * iw * 0.5) * h;
    SVert {
        // Snap to a 1/16 pixel grid so edge functions are exact in f64.
        x: ((x as f64) * 16.0).round() / 16.0,
        y: ((y as f64) * 16.0).round() / 16.0,
        iw,
        uw: c.u * iw,
        vw: c.v * iw,
        l: c.l,
        t: c.t,
        a: c.a,
    }
}

const LIT: u8 = 0;
const UNLIT: u8 = 1;
const SOLID: u8 = 2;
const DARKEN: u8 = 3;
const HIDDEN: u8 = 4;
const GLASS: u8 = 5;
const GLOW: u8 = 6;

fn raster(fb: &mut Frame, sh: &Shading, v0: &CVert, v1: &CVert, v2: &CVert, m: &Mat) {
    match m.mode {
        Mode::Lit => raster_mode::<LIT>(fb, sh, v0, v1, v2, m, 0),
        Mode::Unlit => raster_mode::<UNLIT>(fb, sh, v0, v1, v2, m, 0),
        Mode::Solid(c) => raster_mode::<SOLID>(fb, sh, v0, v1, v2, m, c),
        Mode::Darken(k) => raster_mode::<DARKEN>(fb, sh, v0, v1, v2, m, k.min(2)),
        Mode::Hidden(c) => raster_mode::<HIDDEN>(fb, sh, v0, v1, v2, m, c),
        Mode::Glass => raster_mode::<GLASS>(fb, sh, v0, v1, v2, m, 0),
        Mode::Glow => raster_mode::<GLOW>(fb, sh, v0, v1, v2, m, 0),
    }
}

/// Looks up `src` over `dst` in a blend table at a dithered opacity `a` (vertex scale).
/// `None` leaves the pixel as it is.
#[inline]
fn blend(table: &[u8], a: f32, th: f32, src: u8, dst: u8) -> Option<u8> {
    let k = (a + th) as usize;
    if k == 0 {
        None
    } else if k > BLENDS || dst >= 32 {
        Some(src)
    } else {
        Some(table[((k - 1) * 32 + src as usize) * 32 + dst as usize])
    }
}

#[inline]
fn is_top_left(p: &SVert, q: &SVert) -> bool {
    let dx = q.x - p.x;
    let dy = q.y - p.y;
    dy < 0.0 || (dy == 0.0 && dx > 0.0)
}

#[allow(clippy::too_many_arguments)]
fn raster_mode<const MODE: u8>(
    fb: &mut Frame,
    sh: &Shading,
    v0: &CVert,
    v1: &CVert,
    v2: &CVert,
    m: &Mat,
    param: u8,
) {
    let (fw, fh) = (fb.w as f32, fb.h as f32);
    let s0 = to_screen(v0, fw, fh);
    let mut s1 = to_screen(v1, fw, fh);
    let mut s2 = to_screen(v2, fw, fh);
    let mut area = (s1.x - s0.x) * (s2.y - s0.y) - (s1.y - s0.y) * (s2.x - s0.x);
    if area == 0.0 {
        return;
    }
    // In y-down screen space a front face (counter-clockwise on screen) has negative area.
    if area > 0.0 {
        if m.cull {
            return;
        }
    } else {
        std::mem::swap(&mut s1, &mut s2);
        area = -area;
    }

    let minx = (s0.x.min(s1.x).min(s2.x).floor().max(0.0)) as i32;
    let maxx = (s0.x.max(s1.x).max(s2.x).ceil() as i32).min(fb.w as i32 - 1);
    let miny = (s0.y.min(s1.y).min(s2.y).floor().max(0.0)) as i32;
    let maxy = (s0.y.max(s1.y).max(s2.y).ceil() as i32).min(fb.h as i32 - 1);
    if minx > maxx || miny > maxy {
        return;
    }

    // Edge p->q: E(x, y) = (q.x - p.x) * (y - p.y) - (q.y - p.y) * (x - p.x), inside >= 0.
    let edge =
        |p: &SVert, q: &SVert, x: f64, y: f64| (q.x - p.x) * (y - p.y) - (q.y - p.y) * (x - p.x);
    const BIAS: f64 = 1.0 / 512.0;
    let b0 = if is_top_left(&s1, &s2) { 0.0 } else { -BIAS };
    let b1 = if is_top_left(&s2, &s0) { 0.0 } else { -BIAS };
    let b2 = if is_top_left(&s0, &s1) { 0.0 } else { -BIAS };
    let (sx0, sx1, sx2) = (-(s2.y - s1.y), -(s0.y - s2.y), -(s1.y - s0.y));

    // Plane gradients of the interpolated attributes.
    let inv_area = 1.0 / area;
    let (dx1, dy1) = (s1.x - s0.x, s1.y - s0.y);
    let (dx2, dy2) = (s2.x - s0.x, s2.y - s0.y);
    let grad = |a0: f32, a1: f32, a2: f32| -> (f32, f32) {
        let d1 = (a1 - a0) as f64;
        let d2 = (a2 - a0) as f64;
        (
            ((d1 * dy2 - d2 * dy1) * inv_area) as f32,
            ((d2 * dx1 - d1 * dx2) * inv_area) as f32,
        )
    };
    let giw = grad(s0.iw, s1.iw, s2.iw);
    let gu = grad(s0.uw, s1.uw, s2.uw);
    let gv = grad(s0.vw, s1.vw, s2.vw);
    let gl = grad(s0.l, s1.l, s2.l);
    let gt = grad(s0.t, s1.t, s2.t);
    let ga = grad(s0.a, s1.a, s2.a);

    let tex = m.tex;
    let tdata = &tex.data[..];
    let wmask = tex.w - 1;
    let hmask = tex.h - 1;
    let wshift = tex.wshift;
    let map = &sh.map[..];
    let darken = &sh.darken[param.min(2) as usize];
    let table = if MODE == GLOW {
        &sh.glow[..]
    } else {
        &sh.glass[..]
    };
    let fbw = fb.w;
    let zwrite = m.zwrite;
    let tag = m.tag;
    let door = m.screen_door;

    for py in miny..=maxy {
        let fy = py as f64 + 0.5;
        let fx = minx as f64 + 0.5;
        let mut w0 = edge(&s1, &s2, fx, fy) + b0;
        let mut w1 = edge(&s2, &s0, fx, fy) + b1;
        let mut w2 = edge(&s0, &s1, fx, fy) + b2;
        let ox = (fx - s0.x) as f32;
        let oy = (fy - s0.y) as f32;
        let mut iw = s0.iw + giw.0 * ox + giw.1 * oy;
        let mut uw = s0.uw + gu.0 * ox + gu.1 * oy;
        let mut vw = s0.vw + gv.0 * ox + gv.1 * oy;
        let mut l = s0.l + gl.0 * ox + gl.1 * oy;
        let mut t = s0.t + gt.0 * ox + gt.1 * oy;
        let mut a = s0.a + ga.0 * ox + ga.1 * oy;
        let row = py as usize * fbw;
        let brow = ((py & 3) << 2) as usize;
        let brow2 = (((py + 2) & 3) << 2) as usize;
        let mut entered = false;
        for px in minx..=maxx {
            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                entered = true;
                let idx = row + px as usize;
                let visible = !door || (px + py) & 1 == 0;
                if visible {
                    let front = iw > fb.depth[idx];
                    let wanted = if MODE == HIDDEN {
                        !front && fb.tag[idx] != tag && (px + py) & 1 == 0
                    } else {
                        front
                    };
                    if wanted {
                        let z = 1.0 / iw;
                        let tu = ((uw * z + 4096.0) as u32 & wmask) as usize;
                        let tv = ((vw * z + 4096.0) as u32 & hmask) as usize;
                        let texel = tdata[(tv << wshift) | tu];
                        if texel != CLEAR {
                            let mut drawn = true;
                            match MODE {
                                LIT | GLASS => {
                                    let th = BAYER[brow | (px & 3) as usize];
                                    let th2 = BAYER[brow2 | ((px + 1) & 3) as usize];
                                    let li = ((l + th) as usize).min(LEVELS - 1);
                                    let wi = ((t + th2) as usize).min(WARMTHS - 1);
                                    let c = map[(wi * LEVELS + li) * 32 + texel as usize];
                                    if MODE == LIT {
                                        fb.color[idx] = c;
                                    } else if let Some(c) = blend(table, a, th, c, fb.color[idx]) {
                                        fb.color[idx] = c;
                                    } else {
                                        drawn = false;
                                    }
                                }
                                GLOW => {
                                    let th = BAYER[brow | (px & 3) as usize];
                                    match blend(table, a, th, texel, fb.color[idx]) {
                                        Some(c) => fb.color[idx] = c,
                                        None => drawn = false,
                                    }
                                }
                                UNLIT => fb.color[idx] = texel,
                                SOLID | HIDDEN => fb.color[idx] = param,
                                _ => {
                                    let cur = fb.color[idx];
                                    if cur < 32 {
                                        fb.color[idx] = darken[cur as usize];
                                    }
                                }
                            }
                            if drawn && zwrite && MODE != DARKEN && MODE != HIDDEN {
                                fb.depth[idx] = iw;
                                fb.tag[idx] = tag;
                            }
                        }
                    }
                }
            } else if entered {
                break;
            }
            w0 += sx0;
            w1 += sx1;
            w2 += sx2;
            iw += giw.0;
            uw += gu.0;
            vw += gv.0;
            l += gl.0;
            t += gt.0;
            a += ga.0;
        }
    }
}

/// Draws a screen-aligned square (particles) with a depth test against `iw`.
pub fn draw_point(fb: &mut Frame, x: f32, y: f32, iw: f32, size: i32, color: u8) {
    let x0 = (x - size as f32 * 0.5).round() as i32;
    let y0 = (y - size as f32 * 0.5).round() as i32;
    for yy in y0.max(0)..(y0 + size).min(fb.h as i32) {
        for xx in x0.max(0)..(x0 + size).min(fb.w as i32) {
            let i = yy as usize * fb.w + xx as usize;
            if iw >= fb.depth[i] {
                fb.color[i] = color;
            }
        }
    }
}

/// A soft disc of light added around a screen point (the glow round a spark or ember),
/// strongest in the middle and dithered out to nothing at `radius` pixels. Anything in front
/// of the point hides its glow.
#[allow(clippy::too_many_arguments)]
pub fn draw_halo(
    fb: &mut Frame,
    sh: &Shading,
    x: f32,
    y: f32,
    iw: f32,
    radius: f32,
    color: u8,
    strength: f32,
) {
    let r = radius.max(0.5);
    let x0 = ((x - r).floor() as i32).max(0);
    let x1 = ((x + r).ceil() as i32).min(fb.w as i32 - 1);
    let y0 = ((y - r).floor() as i32).max(0);
    let y1 = ((y + r).ceil() as i32).min(fb.h as i32 - 1);
    let c = color.min(31);
    for yy in y0..=y1 {
        for xx in x0..=x1 {
            let dx = xx as f32 + 0.5 - x;
            let dy = yy as f32 + 0.5 - y;
            let d = (dx * dx + dy * dy).sqrt() / r;
            if d >= 1.0 {
                continue;
            }
            let i = yy as usize * fb.w + xx as usize;
            // A little slack so the glow still washes over the surface a spark sits on.
            if iw * 1.02 < fb.depth[i] {
                continue;
            }
            let k = (1.0 - d) * (1.0 - d) * strength;
            let th = BAYER[((yy as usize & 3) << 2) | (xx as usize & 3)];
            if let Some(out) = blend(&sh.glow, opacity(k), th, c, fb.color[i]) {
                fb.color[i] = out;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{NEUTRAL, WHITE};

    fn v(x: f32, y: f32) -> CVert {
        CVert {
            p: Vec4::new(x, y, 0.5, 1.0),
            u: 0.0,
            v: 0.0,
            l: 16.0,
            t: NEUTRAL,
            a: 0.0,
        }
    }

    #[test]
    fn glass_tints_what_is_behind_and_clear_glass_leaves_it() {
        let sh = Shading::new();
        let tex = Texture::new(1, 1, crate::palette::GREEN);
        let mut fb = Frame::new(32, 32);
        fb.clear(WHITE);
        let mut m = Mat::lit(&tex);
        m.mode = Mode::Glass;
        let tri = |a: f32| {
            let mut t = [v(-0.9, -0.9), v(0.9, -0.9), v(0.0, 0.9)];
            for c in &mut t {
                c.a = a;
            }
            t
        };
        // The left half of what's behind is dark, the right half light.
        for y in 0..32 {
            for x in 0..16 {
                fb.color[y * 32 + x] = crate::palette::INK;
            }
        }
        let before = fb.color.clone();
        draw_tri(&mut fb, &sh, &tri(0.0), &m);
        assert_eq!(fb.color, before, "clear glass drew");
        draw_tri(&mut fb, &sh, &tri(opacity(0.5)), &m);
        let (left, right) = (fb.color[16 * 32 + 12], fb.color[16 * 32 + 19]);
        assert_ne!(left, crate::palette::INK, "glass left no tint");
        assert_ne!(left, right, "glass hid what was behind it");
    }

    #[test]
    fn shared_edges_cover_each_pixel_once() {
        // Two triangles forming a quad, drawn in darken mode: a pixel covered twice would be
        // darkened twice.
        let sh = Shading::new();
        let tex = Texture::new(1, 1, WHITE);
        let mut fb = Frame::new(64, 64);
        fb.clear(WHITE);
        let m = Mat {
            tex: &tex,
            mode: Mode::Darken(0),
            cull: false,
            zwrite: false,
            tag: 0,
            screen_door: false,
        };
        let a = v(-0.8, -0.7);
        let b = v(0.9, -0.8);
        let c = v(0.7, 0.9);
        let d = v(-0.9, 0.6);
        draw_tri(&mut fb, &sh, &[a, b, c], &m);
        draw_tri(&mut fb, &sh, &[a, c, d], &m);
        let once = sh.darken[0][WHITE as usize];
        let twice = sh.darken[0][once as usize];
        assert_ne!(once, WHITE);
        if twice != once {
            assert!(fb.color.iter().all(|&p| p != twice));
        }
        assert!(fb.color.contains(&once));
    }

    #[test]
    fn front_faces_survive_culling() {
        let sh = Shading::new();
        let tex = Texture::new(1, 1, WHITE);
        let mut fb = Frame::new(32, 32);
        fb.clear(0);
        let m = Mat::lit(&tex);
        // Counter-clockwise in NDC (y up) is front facing.
        draw_tri(
            &mut fb,
            &sh,
            &[v(-0.5, -0.5), v(0.5, -0.5), v(0.0, 0.5)],
            &m,
        );
        assert!(fb.depth.iter().any(|&d| d > 0.0));
        let mut fb2 = Frame::new(32, 32);
        draw_tri(
            &mut fb2,
            &sh,
            &[v(-0.5, -0.5), v(0.0, 0.5), v(0.5, -0.5)],
            &m,
        );
        assert!(fb2.depth.iter().all(|&d| d == 0.0));
    }
}
