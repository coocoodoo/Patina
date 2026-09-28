//! A small software 3D renderer that only ever produces Resurrect 32 palette indices.

pub mod camera;
pub mod frame;
pub mod light;
pub mod mesh;
pub mod raster;
pub mod shadow;
pub mod texture;

use glam::{Mat3, Mat4, Vec2, Vec3};

pub use camera::Camera;
pub use frame::Frame;
pub use light::{LightGrid, PointLight};
pub use mesh::{BoxUv, Mesh, UvRect};
pub use raster::{CVert, Mat, Mode, draw_tri, opacity};
pub use shadow::ShadowMap;
pub use texture::{TexBank, TexId, Texture};

use crate::palette::{BAYER, FULL, LEVELS, NEUTRAL, SHADE, Shading};

/// Bends a mesh as it is drawn, in world space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Warp {
    #[default]
    None,
    /// Leans everything above `base` sideways, growing with the square of the height up to
    /// `h`, where it has moved by `lean` (x, z): plants in the wind, wobbling jelly. A
    /// negative `h` swings what hangs below `base` instead (a ghost's trailing sheet).
    Bend { base: f32, h: f32, lean: Vec2 },
    /// A field of grass in the breeze: every vertex leans by the `wind` where it stands,
    /// times `amp`, growing with the square of its height above the ground up to `h`; and
    /// blades near `push` are shoved aside by whoever is wading through them.
    Wind {
        t: f32,
        amp: f32,
        h: f32,
        push: Vec2,
    },
}

/// The breeze outside at a spot: which way and how hard it pushes, as a lean of about 1
/// in a strong gust. A steady drift with gusts that roll across the land in bands, so a
/// wave of bending grass runs downwind, and a flutter on top.
pub fn wind(x: f32, z: f32, t: f32) -> Vec2 {
    const DIR: Vec2 = Vec2::new(0.894, 0.447);
    let along = x * DIR.x + z * DIR.y;
    let across = z * DIR.x - x * DIR.y;
    let band = (along * 0.42 - t * 2.3 + (across * 0.23).sin() * 1.8).sin() * 0.5 + 0.5;
    let gust = band * band * band;
    let flutter = (t * 6.1 + x * 2.3 + z * 1.7).sin() * 0.16 + (t * 3.7 - z * 1.3).sin() * 0.1;
    DIR * (0.2 + gust * 0.95 + flutter) + Vec2::new(-DIR.y, DIR.x) * (flutter * 0.7)
}

impl Warp {
    #[inline]
    pub fn apply(&self, p: Vec3) -> Vec3 {
        match *self {
            Warp::None => p,
            Warp::Bend { base, h, lean } => {
                let k = ((p.y - base) / h).clamp(0.0, 1.0);
                let k2 = k * k;
                // Pull in a touch as it leans so stems keep their length.
                let sink = lean.length_squared() * 0.5 * k2 / h.abs().max(0.01);
                Vec3::new(
                    p.x + lean.x * k2,
                    p.y - sink * h.signum(),
                    p.z + lean.y * k2,
                )
            }
            Warp::Wind { t, amp, h, push } => {
                if p.y <= 0.001 {
                    return p;
                }
                let k = (p.y / h).clamp(0.0, 1.0);
                let k2 = k * k;
                let mut lean = wind(p.x, p.z, t) * amp;
                let d = Vec2::new(p.x - push.x, p.z - push.y);
                let dl = d.length();
                if dl < 0.6 && dl > 1e-4 {
                    lean += d / dl * (0.6 - dl) * 0.45;
                }
                let sink = (lean.length_squared() * 0.5 * k2 / h.max(0.01)).min(p.y * 0.7);
                Vec3::new(p.x + lean.x * k2, p.y - sink, p.z + lean.y * k2)
            }
        }
    }
}

/// Where a draw call takes its light from.
#[derive(Clone, Copy, Debug)]
pub enum Light {
    /// Per vertex from the light grid, times face shading.
    Grid,
    /// One sample from the grid at a point (whole objects light evenly), times face shading.
    At(Vec3),
    /// A fixed level and warmth, times face shading.
    Fixed(f32, f32),
}

#[derive(Clone, Copy, Debug)]
pub struct DrawOpts {
    pub mode: Mode,
    pub cull: bool,
    pub zwrite: bool,
    pub tag: u8,
    pub screen_door: bool,
    pub light: Light,
    /// Lower bound on the light level (0..2), for things that glow a little.
    pub glow: f32,
    /// Opacity (0..1) for `Mode::Glass` and `Mode::Glow`. Glass meshes thicken towards their
    /// silhouettes and catch a specular glint, like jelly.
    pub alpha: f32,
    pub warp: Warp,
}

impl Default for DrawOpts {
    fn default() -> Self {
        DrawOpts {
            mode: Mode::Lit,
            cull: true,
            zwrite: true,
            tag: 0,
            screen_door: false,
            light: Light::Grid,
            glow: 0.0,
            alpha: 1.0,
            warp: Warp::None,
        }
    }
}

impl DrawOpts {
    pub fn at(p: Vec3) -> Self {
        DrawOpts {
            light: Light::At(p),
            ..Default::default()
        }
    }
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }
    pub fn with_tag(mut self, tag: u8) -> Self {
        self.tag = tag;
        self
    }
    pub fn two_sided(mut self) -> Self {
        self.cull = false;
        self
    }
    pub fn with_glow(mut self, g: f32) -> Self {
        self.glow = g;
        self
    }
    pub fn with_warp(mut self, w: Warp) -> Self {
        self.warp = w;
        self
    }
    /// See-through at the given opacity.
    pub fn glass(mut self, alpha: f32) -> Self {
        self.mode = Mode::Glass;
        self.alpha = alpha;
        self
    }
}

pub struct Renderer {
    pub fb: Frame,
    pub sh: Shading,
    pub cam: Camera,
    pub grid: LightGrid,
    /// Texture substitutions applied to meshes (animated water, lava).
    pub remap: Vec<(TexId, TexId)>,
    /// Where the key light comes from (the sun outdoors); faces turned to it are brighter.
    pub key: Vec3,
    /// While set, meshes and sprites are drawn into `sunmap` as shadow casters instead of
    /// onto the screen, and decals, points and halos are skipped.
    pub shadow_pass: bool,
    /// The sun's (or moon's) view of the casters, for `sun_shadows`.
    pub sunmap: ShadowMap,
    /// Round blob shadows under things; off while the sun casts real ones.
    pub blobs: bool,
    /// The player's settings: real shadows outdoors, and ambient occlusion.
    pub want_shadows: bool,
    pub want_ao: bool,
    scratch: Vec<CVert>,
    cast: Vec<Vec3>,
}

impl Renderer {
    pub fn new(w: usize, h: usize) -> Self {
        let mut cam = Camera::default();
        cam.update(w, h);
        Renderer {
            fb: Frame::new(w, h),
            sh: Shading::new(),
            cam,
            grid: LightGrid::default(),
            remap: Vec::new(),
            key: light::KEY,
            shadow_pass: false,
            sunmap: ShadowMap::default(),
            blobs: true,
            want_shadows: true,
            want_ao: true,
            scratch: Vec::new(),
            cast: Vec::new(),
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.fb.resize(w, h);
        self.cam.update(w, h);
    }

    pub fn width(&self) -> usize {
        self.fb.w
    }

    pub fn height(&self) -> usize {
        self.fb.h
    }

    fn light_at(&self, p: Vec3, src: Light) -> (f32, f32) {
        match src {
            Light::Grid => self.grid.sample(p.x, p.z),
            Light::At(q) => self.grid.sample(q.x, q.z),
            Light::Fixed(l, w) => (l, w),
        }
    }

    /// Draws a mesh with a model transform.
    pub fn mesh(&mut self, bank: &TexBank, mesh: &Mesh, model: &Mat4, o: &DrawOpts) {
        if mesh.tris.is_empty() {
            return;
        }
        if self.shadow_pass {
            self.cast_mesh(bank, mesh, model, o);
            return;
        }
        let nm = Mat3::from_mat4(*model);
        let fixed = match o.light {
            Light::Grid => None,
            other => Some(self.light_at(model.w_axis.truncate(), other)),
        };
        let mut scratch = std::mem::take(&mut self.scratch);
        scratch.clear();
        let max_l = LEVELS as f32 - 0.01;
        let glass = matches!(o.mode, Mode::Glass | Mode::Glow);
        let flat = opacity(o.alpha);
        for v in &mesh.verts {
            let wp = o.warp.apply(model.transform_point3(v.pos));
            let wn = (nm * v.n).normalize_or_zero();
            let (lv, wm) = fixed.unwrap_or_else(|| self.grid.sample(wp.x, wp.z));
            let mut l = (lv * light::face_shade_from(wn, self.key) * v.ao).max(o.glow) * FULL;
            let mut a = flat;
            if glass {
                // Thicker where the surface turns away (Fresnel), and a hot glint where it
                // mirrors the key light into the eye.
                let view = (self.cam.eye - wp).normalize_or_zero();
                let facing = wn.dot(view).abs();
                let rim = (1.0 - facing) * (1.0 - facing);
                let half = (self.key + view).normalize_or_zero();
                let spec = wn.dot(half).max(0.0).powi(32);
                l += spec * 10.0 * lv.clamp(0.4, 1.2);
                a = opacity(o.alpha + (1.0 - o.alpha) * rim * 0.9 + spec * 0.25);
            }
            scratch.push(CVert {
                p: self.cam.clip(wp),
                u: v.uv.x,
                v: v.uv.y,
                l: l.clamp(0.0, max_l),
                t: wm,
                a,
            });
        }
        let remap = |t: TexId| {
            self.remap
                .iter()
                .find(|(from, _)| *from == t)
                .map_or(t, |(_, to)| *to)
        };
        let mut last_tex = u16::MAX;
        let mut mat = Mat {
            tex: bank.get(remap(mesh.tris[0].tex)),
            mode: o.mode,
            cull: o.cull,
            zwrite: o.zwrite,
            tag: o.tag,
            screen_door: o.screen_door,
        };
        for t in &mesh.tris {
            if t.tex != last_tex {
                last_tex = t.tex;
                mat.tex = bank.get(remap(t.tex));
            }
            let tri = [
                scratch[t.i[0] as usize],
                scratch[t.i[1] as usize],
                scratch[t.i[2] as usize],
            ];
            draw_tri(&mut self.fb, &self.sh, &tri, &mat);
        }
        self.scratch = scratch;
    }

    /// Draws a mesh into the shadow map as a caster.
    fn cast_mesh(&mut self, bank: &TexBank, mesh: &Mesh, model: &Mat4, o: &DrawOpts) {
        if !casts(o.mode) {
            return;
        }
        let mut pts = std::mem::take(&mut self.cast);
        pts.clear();
        pts.extend(
            mesh.verts
                .iter()
                .map(|v| o.warp.apply(model.transform_point3(v.pos))),
        );
        // Solid shapes put their far side in the map; open ones (leaves, blades) both sides.
        let back_only = o.cull;
        for t in &mesh.tris {
            let [i0, i1, i2] = t.i.map(|i| i as usize);
            let uv = [mesh.verts[i0].uv, mesh.verts[i1].uv, mesh.verts[i2].uv];
            self.sunmap.triangle(
                [pts[i0], pts[i1], pts[i2]],
                uv,
                bank.get(t.tex),
                bank.solid(t.tex),
                back_only,
            );
        }
        self.cast = pts;
    }

    /// Draws a camera-facing sprite standing on `base`.
    pub fn billboard(&mut self, tex: &Texture, r: UvRect, base: Vec3, size: Vec2, o: &DrawOpts) {
        if self.shadow_pass {
            // Glowing sprites (flames, sparkles) cast nothing; cut-outs cast their shape,
            // turned to face the light like a paper doll.
            if !casts(o.mode) || o.mode == Mode::Unlit {
                return;
            }
            let lz = self.sunmap.lz;
            let side = Vec3::new(-lz.z, 0.0, lz.x).normalize_or(Vec3::X) * (size.x * 0.5);
            let up = Vec3::Y * size.y;
            let w = o.warp;
            let bl = w.apply(base - side);
            let br = w.apply(base + side);
            let tr = w.apply(base + side + up);
            let tl = w.apply(base - side + up);
            let (u0, v0, u1, v1) = (r.u0, r.v0, r.u1, r.v1);
            self.sunmap.triangle(
                [bl, br, tr],
                [Vec2::new(u0, v1), Vec2::new(u1, v1), Vec2::new(u1, v0)],
                tex,
                false,
                false,
            );
            self.sunmap.triangle(
                [bl, tr, tl],
                [Vec2::new(u0, v1), Vec2::new(u1, v0), Vec2::new(u0, v0)],
                tex,
                false,
                false,
            );
            return;
        }
        let right = self.cam.right * (size.x * 0.5);
        let up = self.cam.up * size.y;
        let (lv, wm) = self.light_at(base, o.light);
        let l = ((lv * 0.92).max(o.glow) * FULL).clamp(0.0, LEVELS as f32 - 0.01);
        let a = opacity(o.alpha);
        let warp = o.warp;
        let mk = |p: Vec3, u: f32, v: f32| CVert {
            p: self.cam.clip(warp.apply(p)),
            u,
            v,
            l,
            t: wm,
            a,
        };
        let e = 0.02;
        let bl = mk(base - right, r.u0 + e, r.v1 - e);
        let br = mk(base + right, r.u1 - e, r.v1 - e);
        let tr = mk(base + right + up, r.u1 - e, r.v0 + e);
        let tl = mk(base - right + up, r.u0 + e, r.v0 + e);
        let mat = Mat {
            tex,
            mode: o.mode,
            cull: false,
            zwrite: o.zwrite,
            tag: o.tag,
            screen_door: o.screen_door,
        };
        draw_tri(&mut self.fb, &self.sh, &[bl, br, tr], &mat);
        draw_tri(&mut self.fb, &self.sh, &[bl, tr, tl], &mat);
    }

    /// Draws a flat textured quad lying on the ground (decals, shadows, rugs).
    pub fn decal(&mut self, tex: &Texture, r: UvRect, center: Vec3, half: Vec2, o: &DrawOpts) {
        if self.shadow_pass {
            return;
        }
        let (lv, wm) = self.light_at(center, o.light);
        let l = ((lv).max(o.glow) * FULL).clamp(0.0, LEVELS as f32 - 0.01);
        let a = opacity(o.alpha);
        let mk = |x: f32, z: f32, u: f32, v: f32| CVert {
            p: self
                .cam
                .clip(Vec3::new(center.x + x, center.y, center.z + z)),
            u,
            v,
            l,
            t: wm,
            a,
        };
        let e = 0.02;
        let a = mk(-half.x, half.y, r.u0 + e, r.v1 - e);
        let b = mk(half.x, half.y, r.u1 - e, r.v1 - e);
        let c = mk(half.x, -half.y, r.u1 - e, r.v0 + e);
        let d = mk(-half.x, -half.y, r.u0 + e, r.v0 + e);
        let mat = Mat {
            tex,
            mode: o.mode,
            cull: false,
            zwrite: o.zwrite,
            tag: o.tag,
            screen_door: o.screen_door,
        };
        draw_tri(&mut self.fb, &self.sh, &[a, b, c], &mat);
        draw_tri(&mut self.fb, &self.sh, &[a, c, d], &mat);
    }

    /// A round blob shadow under an object (where the sun isn't casting real ones).
    pub fn shadow(&mut self, disk: &Texture, pos: Vec3, radius: f32) {
        if !self.blobs || self.shadow_pass {
            return;
        }
        let o = DrawOpts {
            mode: Mode::Darken(3),
            zwrite: false,
            light: Light::Fixed(1.0, NEUTRAL),
            ..Default::default()
        };
        let r = UvRect::new(0.0, 0.0, disk.w as f32, disk.h as f32);
        self.decal(
            disk,
            r,
            Vec3::new(pos.x, pos.y + 0.02, pos.z),
            Vec2::splat(radius),
            &o,
        );
    }

    /// A small screen-aligned square at a world position (particles, sparks).
    pub fn point(&mut self, p: Vec3, size: i32, color: u8) {
        if self.shadow_pass {
            return;
        }
        if let Some(s) = self.cam.project(p) {
            raster::draw_point(&mut self.fb, s.x, s.y, s.z, size, color);
        }
    }

    /// A twinkle at a world position: a little four-pointed star, `arm` pixels out from a
    /// bright `core`, its tips in `edge`.
    pub fn sparkle(&mut self, p: Vec3, arm: i32, core: u8, edge: u8) {
        if self.shadow_pass {
            return;
        }
        if let Some(s) = self.cam.project(p) {
            raster::draw_star(&mut self.fb, s.x, s.y, s.z, arm, core, edge);
        }
    }

    /// A soft glow of added light, `radius` world units across, centred on `p`.
    pub fn halo(&mut self, p: Vec3, radius: f32, color: u8, strength: f32) {
        if self.shadow_pass {
            return;
        }
        let Some(s) = self.cam.project(p) else { return };
        let Some(e) = self.cam.project(p + self.cam.right * radius) else {
            return;
        };
        let px = (e.x - s.x).abs().max(1.0);
        raster::draw_halo(&mut self.fb, &self.sh, s.x, s.y, s.z, px, color, strength);
    }

    /// Points the sun (or moon) along `dir` (the way its light travels) and gets the shadow
    /// map ready for casters over the tile rectangle `rect`.
    pub fn begin_shadows(&mut self, dir: Vec3, rect: (i32, i32, i32, i32)) {
        let (x0, z0, x1, z1) = rect;
        self.sunmap.setup(
            dir,
            (x0 as f32, z0 as f32, x1 as f32 + 1.0, z1 as f32 + 1.0),
            3.0,
            24.0,
        );
        self.shadow_pass = true;
    }

    /// Darkens every pixel on screen the sun can't see, one step down its colour ramp,
    /// dithered by `strength` (dawn, dusk and moonlight cast fainter shadows).
    pub fn sun_shadows(&mut self, strength: f32) {
        self.shadow_pass = false;
        if strength <= 0.0 || self.sunmap.depth.is_empty() {
            return;
        }
        let cam = &self.cam;
        let map = &self.sunmap;
        let fb = &mut self.fb;
        let (w, h) = (fb.w, fb.h);
        let ty = (cam.fov_y * 0.5).tan();
        let tx = ty * w as f32 / h as f32;
        let (depth, glow) = (&fb.depth, &fb.glow);
        frame::par_rows(&mut fb.color, w, |py, row| {
            let ny = 1.0 - (py as f32 + 0.5) / h as f32 * 2.0;
            let ray = cam.fwd + cam.up * (ny * ty);
            for px in 0..w {
                let i = py * w + px;
                let d = depth[i];
                let c = row[px];
                if d <= 0.0 || c >= 32 || glow[i] {
                    continue;
                }
                if strength < 1.0 && strength <= BAYER[((py & 3) << 2) | (px & 3)] {
                    continue;
                }
                let nx = (px as f32 + 0.5) / w as f32 * 2.0 - 1.0;
                let p = cam.eye + (ray + cam.right * (nx * tx)) / d;
                if map.shadowed(p, 0.06) {
                    row[px] = SHADE[c as usize];
                }
            }
        });
    }

    /// Ambient occlusion over everything drawn so far (see `Frame::ambient_occlusion`).
    pub fn ambient_occlusion(&mut self, strength: f32) {
        self.fb.ambient_occlusion(&SHADE, strength);
    }
}

/// Whether a draw mode leaves a shadow: glass, glows and screen tricks don't.
fn casts(mode: Mode) -> bool {
    !matches!(
        mode,
        Mode::Glass | Mode::Glow | Mode::Hidden(_) | Mode::Darken(_)
    )
}
