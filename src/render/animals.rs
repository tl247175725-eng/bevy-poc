//! Procedural animal meshes — pure vertex colors, Perlin noise stripes.
//! 13 species + black cylindrical base.
//!
//! Each function returns a single merged Y-up mesh. Colors are encoded
//! as vertex colors so no material or texture is required.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

// ═══════════════════════════════════════════════════════════════
//  Color palette
// ═══════════════════════════════════════════════════════════════

const ORANGE: [f32; 3] = [0.85, 0.50, 0.15];
const YELLOW: [f32; 3] = [0.80, 0.70, 0.20];
const RED_BROWN: [f32; 3] = [0.55, 0.20, 0.10];
const BROWN: [f32; 3] = [0.40, 0.25, 0.12];
const DARK_BROWN: [f32; 3] = [0.25, 0.15, 0.08];
const GREY: [f32; 3] = [0.50, 0.50, 0.50];
const GREY_BROWN: [f32; 3] = [0.45, 0.40, 0.35];
const DARK_GREY: [f32; 3] = [0.20, 0.20, 0.20];
const BLACK: [f32; 3] = [0.08, 0.08, 0.08];
const WHITE_ISH: [f32; 3] = [0.85, 0.82, 0.78];
const GREEN_GREY: [f32; 3] = [0.35, 0.45, 0.35];
const BLUE_GREEN: [f32; 3] = [0.20, 0.50, 0.45];
const RABBIT_FUR: [f32; 3] = [0.55, 0.45, 0.35];
const BASE_TOP: [f32; 3] = [0.25, 0.25, 0.25];
const BASE_SIDE: [f32; 3] = [0.10, 0.10, 0.10];

fn col4(c: [f32; 3]) -> [f32; 4] {
    [c[0], c[1], c[2], 1.0]
}

// ═══════════════════════════════════════════════════════════════
//  Perlin noise
// ═══════════════════════════════════════════════════════════════

/// Simple 3D value noise using a hash function and smoothstep
/// interpolation. Deterministic for a given position.
fn hash3(p: [f32; 3]) -> f32 {
    let bits = (p[0] * 127.1 + p[1] * 311.7 + p[2] * 74.7) as i64;
    let n = bits.wrapping_mul(269).wrapping_add(137);
    let h = (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_shr(32);
    (h as f32) * (1.0 / std::u32::MAX as f32)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn simple_perlin(x: f32, y: f32, z: f32) -> f32 {
    let ix = x.floor();
    let iy = y.floor();
    let iz = z.floor();
    let fx = x - ix;
    let fy = y - iy;
    let fz = z - iz;

    let sx = smoothstep(fx);
    let sy = smoothstep(fy);
    let sz = smoothstep(fz);

    let c000 = hash3([ix, iy, iz]);
    let c100 = hash3([ix + 1.0, iy, iz]);
    let c010 = hash3([ix, iy + 1.0, iz]);
    let c110 = hash3([ix + 1.0, iy + 1.0, iz]);
    let c001 = hash3([ix, iy, iz + 1.0]);
    let c101 = hash3([ix + 1.0, iy, iz + 1.0]);
    let c011 = hash3([ix, iy + 1.0, iz + 1.0]);
    let c111 = hash3([ix + 1.0, iy + 1.0, iz + 1.0]);

    let x00 = lerp(c000, c100, sx);
    let x10 = lerp(c010, c110, sx);
    let x01 = lerp(c001, c101, sx);
    let x11 = lerp(c011, c111, sx);
    let y0 = lerp(x00, x10, sy);
    let y1 = lerp(x01, x11, sy);
    lerp(y0, y1, sz)
}

/// Apply Perlin-noise stripes/spots to vertex colors in-place.
/// Vertices where noise > threshold are darkened by `darken` factor.
fn noise_stripe(
    positions: &[[f32; 3]],
    colors: &mut [[f32; 4]],
    scale: f32,
    threshold: f32,
    darken: f32,
    seed_x: f32,
) {
    for (pos, col) in positions.iter().zip(colors.iter_mut()) {
        let n = simple_perlin(
            pos[0] * scale + seed_x,
            pos[1] * scale,
            pos[2] * scale,
        );
        if n > threshold {
            col[0] *= darken;
            col[1] *= darken;
            col[2] *= darken;
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  Mesh builder
// ═══════════════════════════════════════════════════════════════

struct MeshBuilder {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn vertex_offset(&self) -> u32 {
        self.positions.len() as u32
    }

    fn push_vert(&mut self, pos: [f32; 3], col: [f32; 4]) {
        self.positions.push(pos);
        self.colors.push(col);
    }

    /// Push a ring of `n` vertices at Y = `y` around a circle of `radius`.
    fn push_ring(&mut self, y: f32, radius: f32, n: u32, col: [f32; 4]) {
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            self.push_vert([a.cos() * radius, y, a.sin() * radius], col);
        }
    }

    /// Connect two rings of equal vertex count `n` as a strip of quads.
    fn link_rings(&mut self, start_a: u32, start_b: u32, n: u32) {
        for i in 0..n {
            let a = start_a + i;
            let b = start_a + (i + 1) % n;
            let c = start_b + i;
            let d = start_b + (i + 1) % n;
            self.indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }

    /// Ellipsoid centered at `center` with radii `(rx, ry, rz)`.
    /// `lon_segs` = longitudinal divisions, `lat_segs` = latitudinal divisions.
    /// Full sphere (both poles).
    fn push_ellipsoid(
        &mut self,
        center: [f32; 3],
        rx: f32,
        ry: f32,
        rz: f32,
        lon_segs: u32,
        lat_segs: u32,
        col: [f32; 4],
    ) {
        // North pole
        let north = self.vertex_offset();
        self.push_vert([center[0], center[1] + ry, center[2]], col);

        for lat in 1..=lat_segs {
            let theta = lat as f32 / lat_segs as f32 * std::f32::consts::PI;
            let y = center[1] + ry * theta.cos();
            let r_x = rx * theta.sin();
            let r_z = rz * theta.sin();
            let ring = self.vertex_offset();
            for i in 0..lon_segs {
                let a = i as f32 / lon_segs as f32 * std::f32::consts::TAU;
                self.push_vert(
                    [
                        center[0] + a.cos() * r_x,
                        y,
                        center[2] + a.sin() * r_z,
                    ],
                    col,
                );
            }
            if lat == 1 {
                // Fan from north pole
                for i in 0..lon_segs {
                    let a = north;
                    let bv = ring + i;
                    let cv = ring + (i + 1) % lon_segs;
                    self.indices.extend_from_slice(&[a, bv, cv]);
                }
            } else {
                let prev = ring - lon_segs;
                self.link_rings(prev, ring, lon_segs);
            }
        }

        // South pole
        let south = self.vertex_offset();
        self.push_vert([center[0], center[1] - ry, center[2]], col);
        let last_ring = self.vertex_offset() - 1 - lon_segs;
        for i in 0..lon_segs {
            let a = last_ring + i;
            let bv = last_ring + (i + 1) % lon_segs;
            self.indices.extend_from_slice(&[a, bv, south]);
        }
    }

    /// Hemisphere (northern half of ellipsoid).
    fn push_hemisphere(
        &mut self,
        center: [f32; 3],
        rx: f32,
        ry: f32,
        rz: f32,
        lon_segs: u32,
        lat_segs: u32,
        col: [f32; 4],
    ) {
        let apex = self.vertex_offset();
        self.push_vert([center[0], center[1] + ry, center[2]], col);

        for lat in 1..=lat_segs {
            let theta = lat as f32 / lat_segs as f32 * std::f32::consts::FRAC_PI_2;
            let y = center[1] + ry * theta.sin();
            let r_x = rx * theta.cos();
            let r_z = rz * theta.cos();
            let ring = self.vertex_offset();
            for i in 0..lon_segs {
                let a = i as f32 / lon_segs as f32 * std::f32::consts::TAU;
                self.push_vert(
                    [
                        center[0] + a.cos() * r_x,
                        y,
                        center[2] + a.sin() * r_z,
                    ],
                    col,
                );
            }
            if lat == 1 {
                for i in 0..lon_segs {
                    let a = apex;
                    let bv = ring + i;
                    let cv = ring + (i + 1) % lon_segs;
                    self.indices.extend_from_slice(&[a, bv, cv]);
                }
            } else {
                let prev = ring - lon_segs;
                self.link_rings(prev, ring, lon_segs);
            }
        }
    }

    /// Cylinder from `y0` to `y1` with `r0` (bottom radius) and `r1` (top radius).
    /// Open-ended (no caps).
    fn push_cylinder(
        &mut self,
        y0: f32,
        y1: f32,
        r0: f32,
        r1: f32,
        segs: u32,
        col: [f32; 4],
    ) {
        let bottom = self.vertex_offset();
        self.push_ring(y0, r0, segs, col);
        let top = self.vertex_offset();
        self.push_ring(y1, r1, segs, col);
        self.link_rings(bottom, top, segs);
    }

    /// Close the bottom of a cylinder with a fan to center.
    fn close_bottom(&mut self, y: f32, r: f32, segs: u32, col: [f32; 4]) {
        let ring_start = self.vertex_offset();
        self.push_ring(y, r, segs, col);
        let center = self.vertex_offset();
        self.push_vert([0.0, y, 0.0], col);
        for i in 0..segs {
            let a = ring_start + i;
            let bv = ring_start + (i + 1) % segs;
            self.indices.extend_from_slice(&[a, center, bv]);
        }
    }

    fn build(self) -> Mesh {
        let mut mesh =
            Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

// ═══════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════

/// Build 4 legs at approximate corners of a rectangular footprint.
fn add_quad_legs(
    b: &mut MeshBuilder,
    body_len: f32,
    body_width: f32,
    leg_height: f32,
    leg_radius: f32,
    leg_col: [f32; 4],
    segs: u32,
) {
    let hw = body_width * 0.4;
    let hl = body_len * 0.35;
    for (sx, sz) in &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let cx = hl * sx;
        let cz = hw * sz;
        let off = b.vertex_offset();
        for i in 0..segs {
            let a = i as f32 / segs as f32 * std::f32::consts::TAU;
            b.push_vert(
                [cx + a.cos() * leg_radius, -leg_height, cz + a.sin() * leg_radius],
                leg_col,
            );
        }
        let top = b.vertex_offset();
        for i in 0..segs {
            let a = i as f32 / segs as f32 * std::f32::consts::TAU;
            b.push_vert(
                [cx + a.cos() * leg_radius, 0.0, cz + a.sin() * leg_radius],
                leg_col,
            );
        }
        b.link_rings(off, top, segs);
    }
}

/// A simple tail — a slightly tapered cylinder angled backward.
fn add_tail(
    b: &mut MeshBuilder,
    base: [f32; 3],
    length: f32,
    radius: f32,
    segs: u32,
    col: [f32; 4],
) {
    let seg_count = 3;
    let mut prev = b.vertex_offset();
    b.push_ring(base[1], radius, segs, col);
    for s in 1..=seg_count {
        let t = s as f32 / seg_count as f32;
        let r = radius * (1.0 - t * 0.6);
        let y = base[1] - t * length * 0.3;
        let bx = base[0] - t * length * 0.6;
        let bz = base[2];
        let ring = b.vertex_offset();
        for i in 0..segs {
            let a = i as f32 / segs as f32 * std::f32::consts::TAU;
            b.push_vert([bx + a.cos() * r, y, bz + a.sin() * r], col);
        }
        b.link_rings(prev, ring, segs);
        prev = ring;
    }
}

// ═══════════════════════════════════════════════════════════════
//  Animal mesh generators
// ═══════════════════════════════════════════════════════════════

/// 虎 — orange body + Perlin noise dark stripes. ~1000 faces.
pub fn generate_tiger_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let body_col = c4(ORANGE);

    // Body ellipsoid
    b.push_ellipsoid([0.0, 0.6, 0.0], 1.0, 0.4, 0.5, 10, 5, body_col);

    // Head
    b.push_ellipsoid([1.1, 0.8, 0.0], 0.3, 0.3, 0.25, 8, 4, body_col);

    // Legs
    add_quad_legs(&mut b, 2.0, 0.8, 0.5, 0.1, body_col, 6);

    // Tail
    add_tail(&mut b, [-0.9, 0.5, 0.0], 1.0, 0.08, 6, body_col);

    // Ears — 2 small hemispheres
    for (ex, ez) in [(0.25, -0.15), (0.25, 0.15)] {
        b.push_hemisphere([1.15 + ex, 1.1, ez], 0.08, 0.08, 0.08, 6, 2, c4(WHITE_ISH));
    }

    noise_stripe(&b.positions, &mut b.colors, 3.5, 0.55, 0.25, 0.0);
    b.build()
}

/// 豹 — yellow body + Perlin noise dark spots. ~800 faces.
pub fn generate_leopard_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let body_col = c4(YELLOW);

    b.push_ellipsoid([0.0, 0.55, 0.0], 0.9, 0.38, 0.45, 10, 5, body_col);
    b.push_ellipsoid([0.95, 0.75, 0.0], 0.28, 0.28, 0.22, 8, 4, body_col);
    add_quad_legs(&mut b, 1.8, 0.7, 0.45, 0.09, body_col, 6);
    add_tail(&mut b, [-0.85, 0.45, 0.0], 0.9, 0.07, 6, body_col);
    for (ex, ez) in [(0.2, -0.12), (0.2, 0.12)] {
        b.push_hemisphere([1.0 + ex, 1.0, ez], 0.07, 0.07, 0.07, 6, 2, c4(WHITE_ISH));
    }

    noise_stripe(&b.positions, &mut b.colors, 6.0, 0.65, 0.20, 3.7);
    b.build()
}

/// 豺 — reddish-brown, no stripe. ~600 faces.
pub fn generate_dhole_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let col = c4(RED_BROWN);

    b.push_ellipsoid([0.0, 0.5, 0.0], 0.7, 0.35, 0.4, 8, 4, col);
    b.push_ellipsoid([0.75, 0.65, 0.0], 0.25, 0.22, 0.2, 6, 3, col);
    add_quad_legs(&mut b, 1.4, 0.6, 0.4, 0.08, col, 5);
    add_tail(&mut b, [-0.65, 0.4, 0.0], 0.7, 0.06, 5, col);
    for (ex, ez) in [(0.18, -0.1), (0.18, 0.1)] {
        b.push_hemisphere([0.8 + ex, 0.85, ez], 0.06, 0.06, 0.06, 5, 2, col);
    }
    b.build()
}

/// 犀牛 — bulky grey body + horn. ~700 faces.
pub fn generate_rhino_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let col = c4(GREY);

    b.push_ellipsoid([0.0, 0.55, 0.0], 1.2, 0.5, 0.7, 10, 5, col);
    // Head
    b.push_ellipsoid([1.1, 0.45, 0.0], 0.4, 0.35, 0.35, 8, 4, col);
    // Horn — cone
    let horn_segs = 6;
    let horn_base = b.vertex_offset();
    b.push_ring(0.6, 0.12, horn_segs, c4(DARK_GREY));
    let horn_tip = b.vertex_offset();
    b.push_ring(1.0, 0.02, horn_segs, c4(DARK_GREY));
    b.link_rings(horn_base, horn_tip, horn_segs);
    // Legs (short + thick)
    add_quad_legs(&mut b, 2.0, 1.0, 0.35, 0.18, col, 6);
    // Small tail
    add_tail(&mut b, [-1.1, 0.3, 0.0], 0.5, 0.08, 5, col);
    b.build()
}

/// 剑齿象(史前) — large grey-brown body + curved tusks. ~1200 faces.
pub fn generate_stegodon_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let col = c4(GREY_BROWN);

    b.push_ellipsoid([0.0, 0.7, 0.0], 1.5, 0.7, 0.9, 12, 6, col);
    b.push_ellipsoid([1.3, 0.9, 0.0], 0.5, 0.45, 0.4, 8, 4, col);
    // Trunk — curved forward
    let trunk_segs = 5;
    let mut prev_trunk = b.vertex_offset();
    b.push_ring(0.8, 0.15, trunk_segs, col);
    for s in 1..=4 {
        let t = s as f32 / 4.0;
        let r = 0.15 * (1.0 - t * 0.5);
        let ex = t * 1.2;
        let ey = 0.8 - t * 0.6;
        let ring = b.vertex_offset();
        for i in 0..trunk_segs {
            let a = i as f32 / trunk_segs as f32 * std::f32::consts::TAU;
            b.push_vert([1.3 + ex + a.cos() * r, ey, a.sin() * r], col);
        }
        b.link_rings(prev_trunk, ring, trunk_segs);
        prev_trunk = ring;
    }
    // Tusks (2 curved cones)
    for side in &[-1.0, 1.0] {
        let mut prev = b.vertex_offset();
        b.push_ring(0.7, 0.1, 5, c4(WHITE_ISH));
        for s in 1..=4 {
            let t = s as f32 / 4.0;
            let r = 0.1 * (1.0 - t * 0.7);
            let ex = t * 0.8;
            let ey = 0.7 - t * 0.5 - t * t * 0.3;
            let ring = b.vertex_offset();
            for i in 0..5 {
                let a = i as f32 / 5.0 * std::f32::consts::TAU;
                b.push_vert(
                    [1.2 + ex + a.cos() * r, ey, side * (0.25 + t * 0.15) + a.sin() * r],
                    c4(WHITE_ISH),
                );
            }
            b.link_rings(prev, ring, 5);
            prev = ring;
        }
    }
    // Thick legs
    add_quad_legs(&mut b, 2.5, 1.4, 0.55, 0.22, col, 6);
    // Tail
    add_tail(&mut b, [-1.4, 0.4, 0.0], 0.6, 0.1, 5, col);
    b.build()
}

/// 貘 — black front + white rear, short trunk. ~600 faces.
pub fn generate_tapir_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let col_black = c4(BLACK);
    let col_white = c4(WHITE_ISH);

    // Body — black forward half, white rear half
    // We build as front and back ellipsoids merged
    b.push_ellipsoid([-0.4, 0.5, 0.0], 0.6, 0.4, 0.45, 8, 4, col_white);
    b.push_ellipsoid([0.5, 0.5, 0.0], 0.6, 0.4, 0.45, 8, 4, col_black);
    // Head
    b.push_ellipsoid([1.0, 0.6, 0.0], 0.3, 0.25, 0.22, 6, 3, col_black);
    // Short trunk
    b.push_cylinder(0.55, 0.75, 0.08, 0.06, 5, c4(DARK_GREY));
    add_quad_legs(&mut b, 1.6, 0.7, 0.4, 0.1, c4(BLACK), 5);
    add_tail(&mut b, [-1.0, 0.3, 0.0], 0.3, 0.05, 5, c4(BLACK));
    b.build()
}

/// 鹿 — slender body + antlers + white belly. ~900 faces.
pub fn generate_deer_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let col = c4(BROWN);

    b.push_ellipsoid([0.0, 0.6, 0.0], 0.8, 0.45, 0.4, 10, 5, col);
    b.push_ellipsoid([0.85, 0.8, 0.0], 0.25, 0.25, 0.2, 8, 4, col);
    // Long legs
    add_quad_legs(&mut b, 1.6, 0.6, 0.6, 0.07, col, 5);
    // Tail (small white)
    add_tail(&mut b, [-0.75, 0.35, 0.0], 0.2, 0.04, 5, c4(WHITE_ISH));
    // Antlers (branching)
    for side in &[-1.0, 1.0] {
        let base_x = 0.9;
        let base_z = side * 0.12;
        // Main antler beam
        let mut prev = b.vertex_offset();
        b.push_ring(0.9, 0.03, 4, c4(WHITE_ISH));
        for s in 1..=4 {
            let t = s as f32 / 4.0;
            let r = 0.03 * (1.0 - t * 0.5);
            let ex = t * 0.3;
            let ey = 0.9 + t * 0.5;
            let ez = base_z * (1.0 - t * 0.3);
            let ring = b.vertex_offset();
            for i in 0..4 {
                let a = i as f32 / 4.0 * std::f32::consts::TAU;
                b.push_vert(
                    [base_x + ex + a.cos() * r, ey, ez + a.sin() * r],
                    c4(WHITE_ISH),
                );
            }
            b.link_rings(prev, ring, 4);
            prev = ring;
        }
        // Side tine
        let _tine_base = prev - 4; // one ring back
        let tine_start = b.vertex_offset();
        b.push_ring(1.15, 0.025, 4, c4(WHITE_ISH));
        let tine_end = b.vertex_offset();
        for i in 0..4 {
            let a = i as f32 / 4.0 * std::f32::consts::TAU;
            b.push_vert(
                [base_x + 0.35 + a.cos() * 0.02, 1.15 + 0.25, base_z * 0.7 + a.sin() * 0.02],
                c4(WHITE_ISH),
            );
        }
        b.link_rings(tine_start, tine_end, 4);
    }
    // White belly — apply Perlin noise to make belly white
    noise_stripe(&b.positions, &mut b.colors, 2.0, 0.6, 1.8, 0.0);
    b.build()
}

/// 野猪 — stocky dark brown + tusks. ~600 faces.
pub fn generate_boar_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let col = c4(DARK_BROWN);

    b.push_ellipsoid([0.0, 0.5, 0.0], 0.9, 0.4, 0.5, 8, 4, col);
    b.push_ellipsoid([0.8, 0.55, 0.0], 0.35, 0.3, 0.28, 6, 3, col);
    add_quad_legs(&mut b, 1.6, 0.7, 0.35, 0.1, col, 5);
    // Tusks (small cones)
    for side in &[-1.0, 1.0] {
        let tusk_base = b.vertex_offset();
        b.push_ring(0.4, 0.04, 4, c4(WHITE_ISH));
        let tusk_tip = b.vertex_offset();
        for i in 0..4 {
            let a = i as f32 / 4.0 * std::f32::consts::TAU;
            b.push_vert(
                [0.95 + a.cos() * 0.01, 0.25, side * 0.16 + a.sin() * 0.01],
                c4(WHITE_ISH),
            );
        }
        b.link_rings(tusk_base, tusk_tip, 4);
    }
    add_tail(&mut b, [-0.85, 0.3, 0.0], 0.3, 0.06, 5, col);
    b.build()
}

/// 熊 — bulky black body. ~800 faces.
pub fn generate_bear_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let col = c4(BLACK);

    b.push_ellipsoid([0.0, 0.65, 0.0], 1.0, 0.55, 0.6, 10, 5, col);
    b.push_ellipsoid([0.9, 0.85, 0.0], 0.35, 0.35, 0.3, 8, 4, col);
    // Thick legs
    add_quad_legs(&mut b, 1.8, 0.9, 0.5, 0.15, col, 6);
    // Ears
    for (ex, ez) in [(0.22, -0.15), (0.22, 0.15)] {
        b.push_hemisphere([0.95 + ex, 1.15, ez], 0.08, 0.08, 0.08, 6, 2, col);
    }
    add_tail(&mut b, [-0.95, 0.4, 0.0], 0.15, 0.06, 5, col);
    b.build()
}

/// 兔 — small round body + long ears. ~300 faces.
pub fn generate_rabbit_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let col = c4(RABBIT_FUR);

    b.push_ellipsoid([0.0, 0.25, 0.0], 0.35, 0.25, 0.28, 8, 4, col);
    b.push_ellipsoid([0.3, 0.35, 0.0], 0.18, 0.16, 0.15, 6, 3, col);
    // Long ears (2 thin ellipsoids)
    for side in &[-1.0, 1.0] {
        b.push_ellipsoid(
            [0.28, 0.6, side * 0.1],
            0.05,
            0.22,
            0.05,
            5,
            3,
            c4(WHITE_ISH),
        );
    }
    // Legs (small)
    add_quad_legs(&mut b, 0.6, 0.4, 0.15, 0.04, col, 4);
    // Tail (tiny white ball)
    b.push_ellipsoid([-0.3, 0.2, 0.0], 0.06, 0.05, 0.06, 5, 3, c4(WHITE_ISH));
    b.build()
}

/// 鳄 — long flat body + short legs + long tail. ~600 faces.
pub fn generate_crocodile_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let col = c4(GREEN_GREY);

    // Body: elongated flattened ellipsoid
    b.push_ellipsoid([0.0, 0.12, 0.0], 0.8, 0.12, 0.25, 10, 4, col);
    // Head / snout
    b.push_ellipsoid([0.8, 0.1, 0.0], 0.4, 0.08, 0.15, 8, 3, col);
    // Tail (long, tapering)
    let tail_segs = 5;
    let mut prev = b.vertex_offset();
    b.push_ring(0.08, 0.15, 6, col);
    for s in 1..=tail_segs {
        let t = s as f32 / tail_segs as f32;
        let r = 0.15 * (1.0 - t * 0.8);
        let ex = -t * 0.7;
        let ey = 0.08 - t * 0.05;
        let ring = b.vertex_offset();
        for i in 0..6 {
            let a = i as f32 / 6.0 * std::f32::consts::TAU;
            b.push_vert([ex + a.cos() * r, ey, a.sin() * r], col);
        }
        b.link_rings(prev, ring, 6);
        prev = ring;
    }
    // Short legs (splayed)
    for (sx, sz) in &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let cx = 0.4 * sx;
        let cz = 0.2 * sz;
        for i in 0..5 {
            let a = i as f32 / 5.0 * std::f32::consts::TAU;
            b.push_vert(
                [cx + a.cos() * 0.05, -0.08, cz + a.sin() * 0.05],
                col,
            );
        }
        let top = b.vertex_offset();
        for i in 0..5 {
            let a = i as f32 / 5.0 * std::f32::consts::TAU;
            b.push_vert(
                [cx + a.cos() * 0.05, 0.0, cz + a.sin() * 0.05],
                col,
            );
        }
        b.link_rings(top - 5, top, 5);
    }
    // Jaw ridge — dark line
    noise_stripe(&b.positions, &mut b.colors, 8.0, 0.7, 0.5, 2.0);
    b.build()
}

/// 猴 — small body + long arms + tail. ~400 faces.
pub fn generate_monkey_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let col = c4(BROWN);
    let skin = c4(WHITE_ISH);

    b.push_ellipsoid([0.0, 0.3, 0.0], 0.3, 0.25, 0.25, 8, 4, col);
    b.push_ellipsoid([0.35, 0.4, 0.0], 0.18, 0.18, 0.16, 6, 3, skin);
    // Arms (long cylinders)
    for side in &[-1.0, 1.0] {
        let arm_segs = 3;
        let mut prev = b.vertex_offset();
        b.push_ring(0.3, 0.04, 5, col);
        for s in 1..=arm_segs {
            let t = s as f32 / arm_segs as f32;
            let r = 0.04 * (1.0 - t * 0.3);
            let ex = t * 0.4;
            let ey = 0.3 - t * 0.15;
            let ring = b.vertex_offset();
            for i in 0..5 {
                let a = i as f32 / 5.0 * std::f32::consts::TAU;
                b.push_vert(
                    [ex + a.cos() * r, ey, side * (0.2 + t * 0.1) + a.sin() * r],
                    col,
                );
            }
            b.link_rings(prev, ring, 5);
            prev = ring;
        }
    }
    // Legs
    add_quad_legs(&mut b, 0.5, 0.35, 0.25, 0.04, col, 4);
    // Tail (long)
    let tail_segs = 4;
    let mut prev_tail = b.vertex_offset();
    b.push_ring(0.2, 0.04, 5, col);
    for s in 1..=tail_segs {
        let t = s as f32 / tail_segs as f32;
        let r = 0.04 * (1.0 - t * 0.4);
        let ex = -t * 0.35;
        let ey = 0.2 - t * 0.15 + (t * 0.5).sin() * 0.1;
        let ring = b.vertex_offset();
        for i in 0..5 {
            let a = i as f32 / 5.0 * std::f32::consts::TAU;
            b.push_vert([ex + a.cos() * r, ey, a.sin() * r], col);
        }
        b.link_rings(prev_tail, ring, 5);
        prev_tail = ring;
    }
    b.build()
}

/// 孔雀 — bird body + fan tail. ~500 faces.
pub fn generate_peacock_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;
    let body_col = c4(BLUE_GREEN);
    let fan_col = c4([0.15, 0.55, 0.40]);

    // Body
    b.push_ellipsoid([0.0, 0.3, 0.0], 0.35, 0.25, 0.2, 8, 4, body_col);
    // Head (small)
    b.push_ellipsoid([0.4, 0.5, 0.0], 0.1, 0.1, 0.09, 6, 3, c4([0.15, 0.30, 0.25]));
    // Beak
    let beak_segs = 4;
    let beak_base = b.vertex_offset();
    b.push_ring(0.5, 0.03, beak_segs, c4(YELLOW));
    let beak_tip = b.vertex_offset();
    for i in 0..beak_segs {
        let a = i as f32 / beak_segs as f32 * std::f32::consts::TAU;
        b.push_vert([0.52 + a.cos() * 0.01, 0.48, a.sin() * 0.01], c4(YELLOW));
    }
    b.link_rings(beak_base, beak_tip, beak_segs);
    // Fan tail — large half-ellipsoid behind
    let _fan_segs = 8;
    let lon_segs = 6;
    // Back ring
    let fan_back = b.vertex_offset();
    for i in 0..lon_segs {
        let a = i as f32 / lon_segs as f32 * std::f32::consts::TAU;
        let angle = a - std::f32::consts::FRAC_PI_2; // centered behind
        let r = 0.5;
        let ex = -0.2 + angle.sin() * r * 0.6;
        let ey = 0.3 + (angle.cos() * r) * 0.5;
        b.push_vert([ex, ey, angle.sin() * 0.3], fan_col);
    }
    // Outer fan ring
    let fan_outer = b.vertex_offset();
    for i in 0..lon_segs {
        let a = i as f32 / lon_segs as f32 * std::f32::consts::TAU;
        let angle = a - std::f32::consts::FRAC_PI_2;
        let r = 1.0;
        let ex = -0.3 + angle.sin() * r * 0.8;
        let ey = 0.3 + (angle.cos() * r) * 0.8;
        b.push_vert(
            [ex, ey, angle.sin() * 0.5 + (a.cos() * 0.2)],
            col4([0.20, 0.60, 0.50]),
        );
    }
    // Link fan rings with triangles
    for i in 0..lon_segs {
        let next = (i + 1) % lon_segs;
        b.indices.extend_from_slice(&[
            fan_back + i,
            fan_outer + i,
            fan_outer + next,
            fan_back + i,
            fan_outer + next,
            fan_back + next,
        ]);
    }
    // Legs (thin)
    add_quad_legs(&mut b, 0.5, 0.25, 0.3, 0.03, c4(YELLOW), 4);
    // Tail fan "eyes" — small circles with color
    for fi in 0..5 {
        let t = fi as f32 / 5.0;
        let angle = t * std::f32::consts::PI - std::f32::consts::FRAC_PI_2 * 0.5;
        let ex = -0.3 + angle.sin() * 1.2 * 0.7;
        let ey = 0.3 + (angle.cos() * 1.2) * 0.7;
        b.push_ellipsoid([ex, ey, 0.05], 0.04, 0.04, 0.02, 5, 2, c4([0.80, 0.50, 0.20]));
    }
    b.build()
}

/// Black cylindrical base (pedestal). Height 0.15, radius 0.25.
pub fn generate_base_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;

    // Side
    b.push_cylinder(-0.075, 0.075, 0.25, 0.25, 12, c4(BASE_SIDE));
    // Top
    b.close_bottom(0.075, 0.25, 12, c4(BASE_TOP));
    // Bottom (hidden, but build for completeness)
    let bottom_ring = b.vertex_offset();
    b.push_ring(-0.075, 0.25, 12, c4(BASE_SIDE));
    let bottom_c = b.vertex_offset();
    b.push_vert([0.0, -0.075, 0.0], c4(BASE_SIDE));
    for i in 0..12 {
        let a = bottom_ring + i;
        let bv = bottom_ring + (i + 1) % 12;
        b.indices.extend_from_slice(&[a, bottom_c, bv]);
    }

    b.build()
}

// ═══════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn check_mesh(mesh: &Mesh, name: &str) {
        assert!(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some(),
            "{name}: missing POSITION"
        );
        assert!(
            mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some(),
            "{name}: missing COLOR"
        );
        assert!(mesh.indices().is_some(), "{name}: missing INDICES");

        let verts = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().len();
        let idx = mesh.indices().unwrap().len();
        assert!(verts >= 3, "{name}: need ≥3 vertices, got {verts}");
        assert!(idx >= 3, "{name}: need ≥3 indices, got {idx}");
        assert_eq!(idx % 3, 0, "{name}: indices must be multiple of 3");
    }

    macro_rules! test_animal {
        ($name:ident, $func:ident) => {
            #[test]
            fn $name() {
                let mesh = $func();
                check_mesh(&mesh, stringify!($func));
            }
        };
    }

    test_animal!(mesh_tiger, generate_tiger_mesh);
    test_animal!(mesh_leopard, generate_leopard_mesh);
    test_animal!(mesh_dhole, generate_dhole_mesh);
    test_animal!(mesh_rhino, generate_rhino_mesh);
    test_animal!(mesh_stegodon, generate_stegodon_mesh);
    test_animal!(mesh_tapir, generate_tapir_mesh);
    test_animal!(mesh_deer, generate_deer_mesh);
    test_animal!(mesh_boar, generate_boar_mesh);
    test_animal!(mesh_bear, generate_bear_mesh);
    test_animal!(mesh_rabbit, generate_rabbit_mesh);
    test_animal!(mesh_crocodile, generate_crocodile_mesh);
    test_animal!(mesh_monkey, generate_monkey_mesh);
    test_animal!(mesh_peacock, generate_peacock_mesh);
    test_animal!(mesh_base, generate_base_mesh);

    /// Verify every mesh has at least ~200 triangles (meaningful geometry).
    #[test]
    fn all_meshes_have_minimum_triangles() {
        let meshes: Vec<(&str, Mesh)> = vec![
            ("tiger", generate_tiger_mesh()),
            ("leopard", generate_leopard_mesh()),
            ("dhole", generate_dhole_mesh()),
            ("rhino", generate_rhino_mesh()),
            ("stegodon", generate_stegodon_mesh()),
            ("tapir", generate_tapir_mesh()),
            ("deer", generate_deer_mesh()),
            ("boar", generate_boar_mesh()),
            ("bear", generate_bear_mesh()),
            ("rabbit", generate_rabbit_mesh()),
            ("crocodile", generate_crocodile_mesh()),
            ("monkey", generate_monkey_mesh()),
            ("peacock", generate_peacock_mesh()),
            ("base", generate_base_mesh()),
        ];
        for (name, mesh) in &meshes {
            let idx = mesh.indices().unwrap().len();
            let tris = idx / 3;
            assert!(
                tris >= 20,
                "{name}: only {tris} triangles, need ≥20"
            );
        }
    }

    /// Verify Perlin noise produces varied values.
    #[test]
    fn perlin_noise_range() {
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    let v = simple_perlin(x as f32, y as f32, z as f32);
                    min_val = min_val.min(v);
                    max_val = max_val.max(v);
                }
            }
        }
        assert!(min_val >= 0.0, "perlin below 0: {min_val}");
        assert!(max_val <= 1.0, "perlin above 1: {max_val}");
        // Should cover most of [0,1] range
        assert!(max_val - min_val > 0.5, "perlin range too narrow: {min_val}..{max_val}");
    }
}
