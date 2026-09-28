//! Lighting: an ambient level, point lights accumulated on a grid of tile corners (with
//! line-of-sight against walls), and fixed per-face shading for the faceted low-poly look.

use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub pos: Vec3,
    pub radius: f32,
    /// Added light level at the centre (1.0 = full daylight).
    pub power: f32,
    /// Warmth step of the light (0 cold .. 8 fire).
    pub warmth: f32,
}

/// Light level and warmth at every tile corner of a region.
#[derive(Default)]
pub struct LightGrid {
    pub x0: i32,
    pub z0: i32,
    pub w: usize,
    pub h: usize,
    pub level: Vec<f32>,
    pub warm: Vec<f32>,
    pub ambient: f32,
    pub ambient_warm: f32,
    acc: Vec<f32>,
    wsum: Vec<f32>,
}

impl LightGrid {
    /// Rebuilds the grid for the tile rectangle `(x0, z0, x1, z1)` (inclusive).
    pub fn build(
        &mut self,
        rect: (i32, i32, i32, i32),
        ambient: f32,
        ambient_warm: f32,
        lights: &[PointLight],
        solid: &dyn Fn(i32, i32) -> bool,
    ) {
        let (x0, z0, x1, z1) = rect;
        self.x0 = x0;
        self.z0 = z0;
        self.w = (x1 - x0 + 2).max(1) as usize;
        self.h = (z1 - z0 + 2).max(1) as usize;
        self.ambient = ambient;
        self.ambient_warm = ambient_warm;
        let n = self.w * self.h;
        self.level.clear();
        self.level.resize(n, ambient);
        self.warm.clear();
        self.warm.resize(n, ambient_warm);
        let base_w = ambient.max(0.05);
        self.acc.clear();
        self.acc.resize(n, base_w * ambient_warm);
        self.wsum.clear();
        self.wsum.resize(n, base_w);

        for l in lights {
            let r = l.radius;
            let cx0 = ((l.pos.x - r).floor() as i32).max(x0);
            let cx1 = ((l.pos.x + r).ceil() as i32).min(x0 + self.w as i32 - 1);
            let cz0 = ((l.pos.z - r).floor() as i32).max(z0);
            let cz1 = ((l.pos.z + r).ceil() as i32).min(z0 + self.h as i32 - 1);
            if cx0 > cx1 || cz0 > cz1 {
                continue;
            }
            let ltx = l.pos.x.floor() as i32;
            let ltz = l.pos.z.floor() as i32;
            for cz in cz0..=cz1 {
                for cx in cx0..=cx1 {
                    let dx = cx as f32 - l.pos.x;
                    let dz = cz as f32 - l.pos.z;
                    let d2 = dx * dx + dz * dz;
                    if d2 >= r * r {
                        continue;
                    }
                    let k = 1.0 - d2 / (r * r);
                    let c = l.power * k * k;
                    if c < 0.01 {
                        continue;
                    }
                    if !line_of_sight(l.pos.x, l.pos.z, cx as f32, cz as f32, ltx, ltz, solid) {
                        continue;
                    }
                    let i = (cz - z0) as usize * self.w + (cx - x0) as usize;
                    self.level[i] += c;
                    self.acc[i] += c * l.warmth;
                    self.wsum[i] += c;
                }
            }
        }
        for i in 0..n {
            self.warm[i] = self.acc[i] / self.wsum[i];
        }
    }

    #[inline]
    fn at(&self, cx: i32, cz: i32) -> (f32, f32) {
        let x = cx - self.x0;
        let z = cz - self.z0;
        if x < 0 || z < 0 || x >= self.w as i32 || z >= self.h as i32 {
            return (self.ambient, self.ambient_warm);
        }
        let i = z as usize * self.w + x as usize;
        (self.level[i], self.warm[i])
    }

    /// Bilinear sample at a world position.
    pub fn sample(&self, x: f32, z: f32) -> (f32, f32) {
        let fx = x.floor();
        let fz = z.floor();
        let (tx, tz) = (x - fx, z - fz);
        let (ix, iz) = (fx as i32, fz as i32);
        let a = self.at(ix, iz);
        let b = self.at(ix + 1, iz);
        let c = self.at(ix, iz + 1);
        let d = self.at(ix + 1, iz + 1);
        let l = (a.0 * (1.0 - tx) + b.0 * tx) * (1.0 - tz) + (c.0 * (1.0 - tx) + d.0 * tx) * tz;
        let w = (a.1 * (1.0 - tx) + b.1 * tx) * (1.0 - tz) + (c.1 * (1.0 - tx) + d.1 * tx) * tz;
        (l, w)
    }
}

fn line_of_sight(
    lx: f32,
    lz: f32,
    cx: f32,
    cz: f32,
    ltx: i32,
    ltz: i32,
    solid: &dyn Fn(i32, i32) -> bool,
) -> bool {
    let dx = cx - lx;
    let dz = cz - lz;
    let dist = (dx * dx + dz * dz).sqrt();
    if dist < 0.5 {
        return true;
    }
    // Aim slightly short of the corner, on the light's side, so wall faces that look at the
    // light stay lit.
    let tx = cx - dx / dist * 0.3;
    let tz = cz - dz / dist * 0.3;
    let (dx, dz) = (tx - lx, tz - lz);
    let steps = ((dist / 0.33).ceil() as i32).max(1);
    for s in 1..=steps {
        let k = s as f32 / steps as f32;
        let px = (lx + dx * k).floor() as i32;
        let pz = (lz + dz * k).floor() as i32;
        if px == ltx && pz == ltz {
            continue;
        }
        if solid(px, pz) {
            return false;
        }
    }
    true
}

/// Direction towards the fixed key light: from the upper left, a little towards the camera.
pub const KEY: Vec3 = Vec3::new(-0.4205, 0.841, 0.3404);

/// Shading from a key light coming from `key` (a unit vector towards it). Top faces get
/// about 1.0 however high it stands, so the sun can move without the world dimming.
#[inline]
pub fn face_shade_from(n: Vec3, key: Vec3) -> f32 {
    let d = n.dot(key).max(0.0) / key.y.max(0.35);
    0.6 + 0.4 * d.min(1.2)
}
