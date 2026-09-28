//! A small software 3D renderer that only ever produces Resurrect 32 palette indices.

pub mod camera;
pub mod frame;
pub mod light;
pub mod mesh;
pub mod raster;
pub mod texture;

use glam::{Mat3, Mat4, Vec2, Vec3};

pub use camera::Camera;
pub use frame::Frame;
pub use light::{LightGrid, PointLight, face_shade};
pub use mesh::{BoxUv, Mesh, UvRect};
pub use raster::{CVert, Mat, Mode, draw_tri};
pub use texture::{TexBank, TexId, Texture};

use crate::palette::{FULL, LEVELS, NEUTRAL, Shading};

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
}

pub struct Renderer {
    pub fb: Frame,
    pub sh: Shading,
    pub cam: Camera,
    pub grid: LightGrid,
    /// Texture substitutions applied to meshes (animated water, lava).
    pub remap: Vec<(TexId, TexId)>,
    scratch: Vec<CVert>,
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
            scratch: Vec::new(),
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
        let nm = Mat3::from_mat4(*model);
        let fixed = match o.light {
            Light::Grid => None,
            other => Some(self.light_at(model.w_axis.truncate(), other)),
        };
        let mut scratch = std::mem::take(&mut self.scratch);
        scratch.clear();
        let max_l = LEVELS as f32 - 0.01;
        for v in &mesh.verts {
            let wp = model.transform_point3(v.pos);
            let wn = (nm * v.n).normalize_or_zero();
            let (lv, wm) = fixed.unwrap_or_else(|| self.grid.sample(wp.x, wp.z));
            let l = (lv * face_shade(wn) * v.ao).max(o.glow) * FULL;
            scratch.push(CVert {
                p: self.cam.clip(wp),
                u: v.uv.x,
                v: v.uv.y,
                l: l.clamp(0.0, max_l),
                t: wm,
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

    /// Draws a camera-facing sprite standing on `base`.
    pub fn billboard(&mut self, tex: &Texture, r: UvRect, base: Vec3, size: Vec2, o: &DrawOpts) {
        let right = self.cam.right * (size.x * 0.5);
        let up = self.cam.up * size.y;
        let (lv, wm) = self.light_at(base, o.light);
        let l = ((lv * 0.92).max(o.glow) * FULL).clamp(0.0, LEVELS as f32 - 0.01);
        let mk = |p: Vec3, u: f32, v: f32| CVert {
            p: self.cam.clip(p),
            u,
            v,
            l,
            t: wm,
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
        let (lv, wm) = self.light_at(center, o.light);
        let l = ((lv).max(o.glow) * FULL).clamp(0.0, LEVELS as f32 - 0.01);
        let mk = |x: f32, z: f32, u: f32, v: f32| CVert {
            p: self
                .cam
                .clip(Vec3::new(center.x + x, center.y, center.z + z)),
            u,
            v,
            l,
            t: wm,
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

    /// A round blob shadow under an object.
    pub fn shadow(&mut self, disk: &Texture, pos: Vec3, radius: f32) {
        let o = DrawOpts {
            mode: Mode::Darken(0),
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
        if let Some(s) = self.cam.project(p) {
            raster::draw_point(&mut self.fb, s.x, s.y, s.z, size, color);
        }
    }
}
