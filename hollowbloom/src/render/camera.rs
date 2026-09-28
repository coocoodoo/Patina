//! The top-down diorama camera.

use glam::{Mat4, Vec3, Vec4};

#[derive(Clone, Debug)]
pub struct Camera {
    pub target: Vec3,
    /// Angle above the horizon, radians.
    pub pitch: f32,
    pub dist: f32,
    pub fov_y: f32,
    pub w: f32,
    pub h: f32,
    pub eye: Vec3,
    pub vp: Mat4,
    pub inv_vp: Mat4,
    pub right: Vec3,
    pub up: Vec3,
    pub fwd: Vec3,
}

impl Default for Camera {
    fn default() -> Self {
        let mut c = Camera {
            target: Vec3::ZERO,
            pitch: 47f32.to_radians(),
            dist: 15.5,
            fov_y: 30f32.to_radians(),
            w: 480.0,
            h: 270.0,
            eye: Vec3::ZERO,
            vp: Mat4::IDENTITY,
            inv_vp: Mat4::IDENTITY,
            right: Vec3::X,
            up: Vec3::Y,
            fwd: -Vec3::Z,
        };
        c.update(480, 270);
        c
    }
}

impl Camera {
    pub fn update(&mut self, w: usize, h: usize) {
        self.w = w as f32;
        self.h = h as f32;
        let off = Vec3::new(0.0, self.pitch.sin(), self.pitch.cos()) * self.dist;
        self.eye = self.target + off;
        let view = glam::camera::rh::view::look_at_mat4(self.eye, self.target, Vec3::Y);
        let proj =
            glam::camera::rh::proj::directx::perspective(self.fov_y, self.w / self.h, 0.5, 200.0);
        self.vp = proj * view;
        self.inv_vp = self.vp.inverse();
        self.fwd = (self.target - self.eye).normalize();
        self.right = self.fwd.cross(Vec3::Y).normalize();
        self.up = self.right.cross(self.fwd).normalize();
    }

    #[inline]
    pub fn clip(&self, p: Vec3) -> Vec4 {
        self.vp * p.extend(1.0)
    }

    /// World point to (screen x, screen y, 1/w). `None` when behind the near plane.
    pub fn project(&self, p: Vec3) -> Option<Vec3> {
        let c = self.clip(p);
        if c.z < 0.0 || c.w <= 0.0 {
            return None;
        }
        let iw = 1.0 / c.w;
        Some(Vec3::new(
            (c.x * iw * 0.5 + 0.5) * self.w,
            (0.5 - c.y * iw * 0.5) * self.h,
            iw,
        ))
    }

    /// Screen pixel to a world ray (origin, direction).
    pub fn ray(&self, sx: f32, sy: f32) -> (Vec3, Vec3) {
        let nx = sx / self.w * 2.0 - 1.0;
        let ny = 1.0 - sy / self.h * 2.0;
        let a = self.inv_vp.project_point3(Vec3::new(nx, ny, 0.0));
        let b = self.inv_vp.project_point3(Vec3::new(nx, ny, 1.0));
        (a, (b - a).normalize())
    }

    /// Where a screen pixel meets the horizontal plane at height `y`.
    pub fn ground(&self, sx: f32, sy: f32, y: f32) -> Option<Vec3> {
        let (o, d) = self.ray(sx, sy);
        if d.y.abs() < 1e-5 {
            return None;
        }
        let t = (y - o.y) / d.y;
        (t > 0.0).then(|| o + d * t)
    }

    /// Tile rectangle (x0, z0, x1, z1), inclusive, that can be on screen for geometry up to
    /// `max_h` tall.
    pub fn visible_tiles(&self, max_h: f32) -> (i32, i32, i32, i32) {
        let mut lo = Vec3::splat(f32::MAX);
        let mut hi = Vec3::splat(f32::MIN);
        for (sx, sy) in [(0.0, 0.0), (self.w, 0.0), (0.0, self.h), (self.w, self.h)] {
            for y in [0.0, max_h] {
                if let Some(p) = self.ground(sx, sy, y) {
                    lo = lo.min(p);
                    hi = hi.max(p);
                }
            }
        }
        if lo.x > hi.x {
            let t = self.target;
            return (
                t.x as i32 - 20,
                t.z as i32 - 20,
                t.x as i32 + 20,
                t.z as i32 + 20,
            );
        }
        (
            lo.x.floor() as i32 - 1,
            lo.z.floor() as i32 - 1,
            hi.x.ceil() as i32 + 1,
            hi.z.ceil() as i32 + 2,
        )
    }
}
