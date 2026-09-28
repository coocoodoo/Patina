//! Shadows from the sun (or the moon): everything that casts is drawn into a depth map as
//! seen from the light, and every pixel on screen is then checked against it, so shadows
//! fall on the ground, on walls, on crops and on whoever walks through them.

use glam::{Vec2, Vec3};

use super::texture::Texture;
use crate::palette::CLEAR;

/// The light's point of view: an orthographic depth map over the part of the world on screen.
#[derive(Default)]
pub struct ShadowMap {
    /// Nearest depth along the light at every texel (smaller is nearer the light).
    pub depth: Vec<f32>,
    pub w: usize,
    pub h: usize,
    /// Light-space axes: across, up the map, and along the light (towards the ground).
    pub lx: Vec3,
    pub ly: Vec3,
    pub lz: Vec3,
    /// Light-space coordinates of texel (0, 0), and texels per world unit on each axis.
    pub origin: Vec2,
    pub scale: Vec2,
}

impl ShadowMap {
    /// Points the map along the light `dir` (pointing down from the light) and fits it over
    /// the world rectangle `(x0, z0, x1, z1)` for things up to `max_h` tall.
    pub fn setup(&mut self, dir: Vec3, rect: (f32, f32, f32, f32), max_h: f32, density: f32) {
        let lz = dir.normalize();
        let helper = if lz.y.abs() < 0.99 { Vec3::Y } else { Vec3::Z };
        let lx = helper.cross(lz).normalize();
        let ly = lz.cross(lx);
        let (x0, z0, x1, z1) = rect;
        let mut lo = Vec2::splat(f32::MAX);
        let mut hi = Vec2::splat(f32::MIN);
        for x in [x0, x1] {
            for z in [z0, z1] {
                for y in [0.0, max_h] {
                    let p = Vec3::new(x, y, z);
                    let q = Vec2::new(p.dot(lx), p.dot(ly));
                    lo = lo.min(q);
                    hi = hi.max(q);
                }
            }
        }
        // The map is squashed along the light's slant: keep about `density` texels per unit
        // of ground either way, so dawn shadows stay as crisp as noon ones.
        let slant = (-lz.y).clamp(0.2, 1.0);
        let scale = Vec2::new(density, density / slant);
        let size = (hi - lo) * scale;
        self.w = (size.x.ceil() as usize).clamp(1, 1024);
        self.h = (size.y.ceil() as usize).clamp(1, 1024);
        self.scale = Vec2::new(
            self.w as f32 / (hi.x - lo.x).max(0.01),
            self.h as f32 / (hi.y - lo.y).max(0.01),
        );
        self.origin = lo;
        self.lx = lx;
        self.ly = ly;
        self.lz = lz;
        let n = self.w * self.h;
        self.depth.clear();
        self.depth.resize(n, f32::MAX);
    }

    /// A world point in map texels (x, y) and depth along the light.
    #[inline]
    pub fn project(&self, p: Vec3) -> Vec3 {
        Vec3::new(
            (p.dot(self.lx) - self.origin.x) * self.scale.x,
            (p.dot(self.ly) - self.origin.y) * self.scale.y,
            p.dot(self.lz),
        )
    }

    /// Is a world point in shadow (with a little slack so surfaces don't shadow themselves)?
    #[inline]
    pub fn shadowed(&self, p: Vec3, bias: f32) -> bool {
        let q = self.project(p);
        if q.x < 0.0 || q.y < 0.0 {
            return false;
        }
        let (x, y) = (q.x as usize, q.y as usize);
        if x >= self.w || y >= self.h {
            return false;
        }
        q.z > self.depth[y * self.w + x] + bias
    }

    /// Draws one triangle's depth into the map, skipping clear texels (unless the texture is
    /// `solid` all over). With `back_only`, only faces turned away from the light are kept:
    /// for solid shapes that puts the depth on their far side, so their lit faces never
    /// shadow themselves.
    pub fn triangle(
        &mut self,
        p: [Vec3; 3],
        uv: [Vec2; 3],
        tex: &Texture,
        solid: bool,
        back_only: bool,
    ) {
        if back_only {
            let n = (p[1] - p[0]).cross(p[2] - p[0]);
            if n.dot(self.lz) < 0.0 {
                return;
            }
        }
        let a = self.project(p[0]);
        let b = self.project(p[1]);
        let c = self.project(p[2]);
        let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        if area.abs() < 1e-6 {
            return;
        }
        let minx = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
        let maxx = (a.x.max(b.x).max(c.x).ceil() as i32).min(self.w as i32 - 1);
        let miny = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
        let maxy = (a.y.max(b.y).max(c.y).ceil() as i32).min(self.h as i32 - 1);
        if minx > maxx || miny > maxy {
            return;
        }
        let inv = 1.0 / area;
        // Barycentric weights step by a constant along each row.
        let (e0x, e0y) = ((b.y - c.y) * inv, (c.x - b.x) * inv);
        let (e1x, e1y) = ((c.y - a.y) * inv, (a.x - c.x) * inv);
        let tw = tex.w as f32 * 256.0;
        let th = tex.h as f32 * 256.0;
        let (wm, hm) = (tex.w - 1, tex.h - 1);
        let w = self.w;
        for y in miny..=maxy {
            let fy = y as f32 + 0.5;
            let fx = minx as f32 + 0.5;
            let mut w0 = ((b.x - fx) * (c.y - fy) - (b.y - fy) * (c.x - fx)) * inv;
            let mut w1 = ((c.x - fx) * (a.y - fy) - (c.y - fy) * (a.x - fx)) * inv;
            let row = y as usize * w;
            let mut entered = false;
            for x in minx..=maxx {
                let w2 = 1.0 - w0 - w1;
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    entered = true;
                    let d = a.z * w0 + b.z * w1 + c.z * w2;
                    let i = row + x as usize;
                    if d < self.depth[i] {
                        let keep = solid || {
                            let u = uv[0].x * w0 + uv[1].x * w1 + uv[2].x * w2;
                            let v = uv[0].y * w0 + uv[1].y * w1 + uv[2].y * w2;
                            let tu = ((u + tw) as u32 & wm) as usize;
                            let tv = ((v + th) as u32 & hm) as usize;
                            tex.data[(tv << tex.wshift) | tu] != CLEAR
                        };
                        if keep {
                            self.depth[i] = d;
                        }
                    }
                } else if entered {
                    break;
                }
                w0 += e0x;
                w1 += e1x;
            }
            let _ = (e0y, e1y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_shades_the_ground_behind_it_but_not_its_own_sunny_side() {
        let mut m = ShadowMap::default();
        // Sun from the east, fairly low.
        let dir = Vec3::new(-1.0, -0.6, 0.0).normalize();
        m.setup(dir, (-4.0, -4.0, 4.0, 4.0), 2.0, 24.0);
        let tex = Texture::new(1, 1, 3);
        let uv = [Vec2::ZERO; 3];
        // A unit post at the origin, one metre tall: its two x faces.
        let q = |x: f32| {
            [
                [
                    Vec3::new(x, 0.0, -0.1),
                    Vec3::new(x, 1.0, -0.1),
                    Vec3::new(x, 1.0, 0.1),
                ],
                [
                    Vec3::new(x, 0.0, -0.1),
                    Vec3::new(x, 1.0, 0.1),
                    Vec3::new(x, 0.0, 0.1),
                ],
            ]
        };
        for side in [q(0.1), q(-0.1)] {
            for t in side {
                m.triangle(t, uv, &tex, true, false);
            }
        }
        // West of the post, in its shadow; east of it, in the sun.
        assert!(m.shadowed(Vec3::new(-1.0, 0.0, 0.0), 0.05));
        assert!(!m.shadowed(Vec3::new(1.0, 0.0, 0.0), 0.05));
        assert!(!m.shadowed(Vec3::new(-1.0, 0.0, 2.0), 0.05));
        // The post's own sunny face is lit.
        assert!(!m.shadowed(Vec3::new(0.1, 0.5, 0.0), 0.05));
    }
}
