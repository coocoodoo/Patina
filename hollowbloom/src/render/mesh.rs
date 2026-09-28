//! Low-poly meshes: boxes with per-face pixel-art UVs, lathes (revolved profiles) and quads.
//! Faces are flat shaded; every face owns its vertices.

use glam::{Mat3, Mat4, Vec2, Vec3};

use super::texture::TexId;

#[derive(Clone, Copy, Debug)]
pub struct MVert {
    pub pos: Vec3,
    pub n: Vec3,
    pub uv: Vec2,
    /// Baked ambient occlusion / light multiplier.
    pub ao: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct MTri {
    pub i: [u32; 3],
    pub tex: TexId,
}

/// A rectangle in texel space (u0, v0) - (u1, v1).
#[derive(Clone, Copy, Debug, Default)]
pub struct UvRect {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

impl UvRect {
    pub const fn new(u0: f32, v0: f32, u1: f32, v1: f32) -> Self {
        UvRect { u0, v0, u1, v1 }
    }
    pub const fn px(x: i32, y: i32, w: i32, h: i32) -> Self {
        UvRect {
            u0: x as f32,
            v0: y as f32,
            u1: (x + w) as f32,
            v1: (y + h) as f32,
        }
    }
    /// Pulls the rectangle in slightly so interpolation never samples a neighbouring texel.
    fn inset(self) -> Self {
        const E: f32 = 0.02;
        let du = if self.u1 >= self.u0 { E } else { -E };
        let dv = if self.v1 >= self.v0 { E } else { -E };
        UvRect {
            u0: self.u0 + du,
            v0: self.v0 + dv,
            u1: self.u1 - du,
            v1: self.v1 - dv,
        }
    }
}

/// Per-face UVs for a box, in the order east (+x), west (-x), top (+y), bottom (-y),
/// south/front (+z), north/back (-z).
#[derive(Clone, Copy, Debug)]
pub struct BoxUv(pub [UvRect; 6]);

impl BoxUv {
    pub fn all(r: UvRect) -> Self {
        BoxUv([r; 6])
    }
}

pub const TOP: usize = 2;
pub const FRONT: usize = 4;

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub verts: Vec<MVert>,
    pub tris: Vec<MTri>,
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a quad given counter-clockwise corners (seen from the front) in the order
    /// bottom-left, bottom-right, top-right, top-left.
    pub fn quad(&mut self, p: [Vec3; 4], r: UvRect, tex: TexId) {
        self.quad_ao(p, r, tex, [1.0; 4]);
    }

    /// A quad with per-corner light multipliers (baked ambient occlusion).
    pub fn quad_ao(&mut self, p: [Vec3; 4], r: UvRect, tex: TexId, ao: [f32; 4]) {
        let r = r.inset();
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let uv = [
            Vec2::new(r.u0, r.v1),
            Vec2::new(r.u1, r.v1),
            Vec2::new(r.u1, r.v0),
            Vec2::new(r.u0, r.v0),
        ];
        let base = self.verts.len() as u32;
        for k in 0..4 {
            self.verts.push(MVert {
                pos: p[k],
                n,
                uv: uv[k],
                ao: ao[k],
            });
        }
        self.tris.push(MTri {
            i: [base, base + 1, base + 2],
            tex,
        });
        self.tris.push(MTri {
            i: [base, base + 2, base + 3],
            tex,
        });
    }

    /// Adds a triangle with explicit UVs (counter-clockwise from the front).
    pub fn tri(&mut self, p: [Vec3; 3], uv: [Vec2; 3], tex: TexId) {
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let base = self.verts.len() as u32;
        for k in 0..3 {
            self.verts.push(MVert {
                pos: p[k],
                n,
                uv: uv[k],
                ao: 1.0,
            });
        }
        self.tris.push(MTri {
            i: [base, base + 1, base + 2],
            tex,
        });
    }

    /// Adds an axis-aligned box. Faces whose bit is set in `skip` (1 << face) are omitted.
    pub fn cube(&mut self, min: Vec3, max: Vec3, uv: &BoxUv, tex: TexId, skip: u8) {
        let (x0, y0, z0) = (min.x, min.y, min.z);
        let (x1, y1, z1) = (max.x, max.y, max.z);
        let v = Vec3::new;
        let faces: [[Vec3; 4]; 6] = [
            [v(x1, y0, z1), v(x1, y0, z0), v(x1, y1, z0), v(x1, y1, z1)],
            [v(x0, y0, z0), v(x0, y0, z1), v(x0, y1, z1), v(x0, y1, z0)],
            [v(x0, y1, z1), v(x1, y1, z1), v(x1, y1, z0), v(x0, y1, z0)],
            [v(x0, y0, z0), v(x1, y0, z0), v(x1, y0, z1), v(x0, y0, z1)],
            [v(x0, y0, z1), v(x1, y0, z1), v(x1, y1, z1), v(x0, y1, z1)],
            [v(x1, y0, z0), v(x0, y0, z0), v(x0, y1, z0), v(x1, y1, z0)],
        ];
        for (f, corners) in faces.iter().enumerate() {
            if skip & (1 << f) == 0 {
                self.quad(*corners, uv.0[f], tex);
            }
        }
    }

    /// Revolves a profile of `(radius, y, v)` points (bottom to top) around the Y axis.
    /// `u_span` is the texel width wrapped once around; caps are added for non-zero radii.
    #[allow(clippy::too_many_arguments)]
    pub fn lathe(
        &mut self,
        center: Vec3,
        profile: &[(f32, f32, f32)],
        segments: usize,
        u_span: f32,
        phase: f32,
        tex: TexId,
        caps: (Option<UvRect>, Option<UvRect>),
    ) {
        let seg = segments.max(3);
        let ring = |i: usize, j: usize| {
            let (r, y, _) = profile[i];
            let a = phase + j as f32 / seg as f32 * std::f32::consts::TAU;
            center + Vec3::new(r * a.cos(), y, -r * a.sin())
        };
        for i in 0..profile.len().saturating_sub(1) {
            let (va, vb) = (profile[i].2, profile[i + 1].2);
            for j in 0..seg {
                let u0 = j as f32 / seg as f32 * u_span;
                let u1 = (j + 1) as f32 / seg as f32 * u_span;
                let p = [
                    ring(i, j),
                    ring(i, j + 1),
                    ring(i + 1, j + 1),
                    ring(i + 1, j),
                ];
                if profile[i].0 <= 1e-4 {
                    // Pointed bottom: a single triangle.
                    self.tri(
                        [p[0], p[2], p[3]],
                        [
                            Vec2::new((u0 + u1) * 0.5, va),
                            Vec2::new(u1, vb),
                            Vec2::new(u0, vb),
                        ],
                        tex,
                    );
                } else if profile[i + 1].0 <= 1e-4 {
                    self.tri(
                        [p[0], p[1], p[2]],
                        [
                            Vec2::new(u0, va),
                            Vec2::new(u1, va),
                            Vec2::new((u0 + u1) * 0.5, vb),
                        ],
                        tex,
                    );
                } else {
                    self.quad(p, UvRect::new(u0, vb, u1, va), tex);
                }
            }
        }
        let cap = |m: &mut Mesh, i: usize, r: UvRect, up: bool| {
            let (rad, y, _) = profile[i];
            if rad <= 1e-4 {
                return;
            }
            let c = center + Vec3::Y * y;
            let (cu, cv) = ((r.u0 + r.u1) * 0.5, (r.v0 + r.v1) * 0.5);
            let (hu, hv) = ((r.u1 - r.u0) * 0.49, (r.v1 - r.v0) * 0.49);
            for j in 0..seg {
                let a0 = phase + j as f32 / seg as f32 * std::f32::consts::TAU;
                let a1 = phase + (j + 1) as f32 / seg as f32 * std::f32::consts::TAU;
                let p0 = ring(i, j);
                let p1 = ring(i, j + 1);
                let uv0 = Vec2::new(cu + a0.cos() * hu, cv + a0.sin() * hv);
                let uv1 = Vec2::new(cu + a1.cos() * hu, cv + a1.sin() * hv);
                let uc = Vec2::new(cu, cv);
                if up {
                    m.tri([c, p0, p1], [uc, uv0, uv1], tex);
                } else {
                    m.tri([c, p1, p0], [uc, uv1, uv0], tex);
                }
            }
        };
        if let Some(r) = caps.0 {
            cap(self, 0, r, false);
        }
        if let Some(r) = caps.1 {
            cap(self, profile.len() - 1, r, true);
        }
    }

    pub fn append(&mut self, other: &Mesh, m: Mat4) {
        let nm = Mat3::from_mat4(m).inverse().transpose();
        let base = self.verts.len() as u32;
        self.verts.extend(other.verts.iter().map(|v| MVert {
            pos: m.transform_point3(v.pos),
            n: (nm * v.n).normalize_or_zero(),
            uv: v.uv,
            ao: v.ao,
        }));
        self.tris.extend(other.tris.iter().map(|t| MTri {
            i: [t.i[0] + base, t.i[1] + base, t.i[2] + base],
            tex: t.tex,
        }));
    }

    /// Re-targets every triangle to another texture (palette-swapped variants).
    pub fn retexture(&self, from: TexId, to: TexId) -> Mesh {
        let mut m = self.clone();
        for t in &mut m.tris {
            if t.tex == from {
                t.tex = to;
            }
        }
        m
    }
}
