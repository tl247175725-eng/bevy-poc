//! Procedural vegetation meshes — pure vertex colors, no textures.
//! Each function generates a 3D Y-up mesh for one plant species.
//!
//! Colors follow the handoff spec: bark/leaf colors encoded as vertex
//! colors so no material or texture is required.

use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::mesh::RenderAssetUsages;

// ═══════════════════════════════════════════════════════════════
//  Color palette
// ═══════════════════════════════════════════════════════════════

/// Bark — nanmu (light grey-brown)
const BARK_NANMU: [f32; 3] = [0.40, 0.35, 0.28];
/// Bark — camphor (dark blackish-brown)
const BARK_CAMPHOR: [f32; 3] = [0.22, 0.18, 0.14];
/// Bark — pine (reddish-brown)
const BARK_PINE: [f32; 3] = [0.35, 0.20, 0.15];
/// Bamboo stem (yellow-green)
const BAMBOO_COLOR: [f32; 3] = [0.55, 0.65, 0.35];

/// Broadleaf (deep green)
const LEAF_BROADLEAF: [f32; 3] = [0.18, 0.35, 0.12];
/// Pine needle (dark green)
const LEAF_PINE: [f32; 3] = [0.10, 0.25, 0.10];
/// Bamboo leaf (yellow-green)
const LEAF_BAMBOO: [f32; 3] = [0.35, 0.50, 0.20];
/// Azalea flower (pink)
const LEAF_FLOWER: [f32; 3] = [0.75, 0.40, 0.35];

/// Reed spike / grass colour
const REED_COLOR: [f32; 3] = [0.50, 0.55, 0.25];
const MISCANTHUS_COLOR: [f32; 3] = [0.45, 0.60, 0.20];
const LOTUS_PAD: [f32; 3] = [0.15, 0.40, 0.15];
const LOTUS_CENTER: [f32; 3] = [0.80, 0.70, 0.20];
const WATERWEED_COLOR: [f32; 3] = [0.10, 0.35, 0.15];

fn col4(c: [f32; 3]) -> [f32; 4] {
    [c[0], c[1], c[2], 1.0]
}

// ═══════════════════════════════════════════════════════════════
//  Mesh builder — small helper to assemble a Mesh
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
//  Trees
// ═══════════════════════════════════════════════════════════════

/// 楠木 — 半球冠 + 直干。冠半径 2.5，干高 6。~40 面。
pub fn generate_nanmu_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;

    // ── Trunk (6-sided cylinder, 2 segments) ──
    let trunk_segs = 6;
    let trunk_radius = 0.4;
    let trunk_height = 6.0;
    let bark = c4(BARK_NANMU);

    // Bottom ring
    let r0 = b.vertex_offset();
    b.push_ring(-trunk_height, trunk_radius, trunk_segs, bark);
    // Mid ring
    let r1 = b.vertex_offset();
    b.push_ring(-trunk_height * 0.4, trunk_radius * 0.9, trunk_segs, bark);
    // Top ring
    let r2 = b.vertex_offset();
    b.push_ring(0.0, trunk_radius, trunk_segs, bark);
    b.link_rings(r0, r1, trunk_segs);
    b.link_rings(r1, r2, trunk_segs);

    // ── Hemisphere crown — 4 lat × 6 lon ──
    let crown_radius = 2.5;
    let lat_segs = 4;
    let lon_segs = 6;
    let leaf = c4(LEAF_BROADLEAF);

    // Apex vertex
    let apex = b.vertex_offset();
    b.push_vert([0.0, crown_radius, 0.0], leaf);

    for lat in 1..=lat_segs {
        let theta = lat as f32 / lat_segs as f32 * std::f32::consts::FRAC_PI_2;
        let r = crown_radius * theta.cos();
        let y = crown_radius * theta.sin();
        let ring_start = b.vertex_offset();
        b.push_ring(y, r, lon_segs, leaf);

        if lat == 1 {
            // Connect apex to first ring (fan)
            for i in 0..lon_segs {
                let a = apex;
                let bv = ring_start + i;
                let cv = ring_start + (i + 1) % lon_segs;
                b.indices.extend_from_slice(&[a, bv, cv]);
            }
        } else {
            let prev_ring = ring_start - lon_segs;
            b.link_rings(prev_ring, ring_start, lon_segs);
        }
    }

    b.build()
}

/// 樟树 — 扁椭球冠 + 粗干 + 外展枝。冠宽 7×高 4。~50 面。
pub fn generate_camphor_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;

    let bark = c4(BARK_CAMPHOR);
    let leaf = c4(LEAF_BROADLEAF);

    // ── Trunk (6-sided cylinder) ──
    let trunk_segs = 6;
    let trunk_r = 0.6;
    let trunk_h = 3.5;
    let r0 = b.vertex_offset();
    b.push_ring(-trunk_h, trunk_r, trunk_segs, bark);
    let r1 = b.vertex_offset();
    b.push_ring(0.0, trunk_r * 0.9, trunk_segs, bark);
    b.link_rings(r0, r1, trunk_segs);

    // ── Ellipsoid crown — 4 lat × 8 lon ──
    let rx = 3.5;
    let ry = 2.0;
    let rz = 3.5;
    let lat_segs = 4;
    let lon_segs = 8;

    // Apex
    let apex = b.vertex_offset();
    b.push_vert([0.0, ry, 0.0], leaf);

    for lat in 1..=lat_segs {
        let theta = lat as f32 / lat_segs as f32 * std::f32::consts::FRAC_PI_2;
        let y = ry * theta.cos(); // ry at top, 0 at equator
        // Scale horizontal radius by sin(theta) so it's 0 at apex, 1 at equator
        let hscale = theta.sin();
        let ring_start = b.vertex_offset();
        for i in 0..lon_segs {
            let a = i as f32 / lon_segs as f32 * std::f32::consts::TAU;
            b.push_vert([a.cos() * rx * hscale, y, a.sin() * rz * hscale], leaf);
        }

        if lat == 1 {
            for i in 0..lon_segs {
                let a = apex;
                let bv = ring_start + i;
                let cv = ring_start + (i + 1) % lon_segs;
                b.indices.extend_from_slice(&[a, bv, cv]);
            }
        } else {
            let prev = ring_start - lon_segs;
            b.link_rings(prev, ring_start, lon_segs);
        }
    }

    // Equator ring to complete bottom half (mirror top rings)
    // lat from lat_segs down to 1
    for lat in (0..lat_segs).rev() {
        let theta = lat as f32 / lat_segs as f32 * std::f32::consts::FRAC_PI_2;
        let hscale = theta.sin();
        let y = -ry * theta.cos();
        let ring_start = b.vertex_offset();
        for i in 0..lon_segs {
            let a = i as f32 / lon_segs as f32 * std::f32::consts::TAU;
            b.push_vert([a.cos() * rx * hscale, y, a.sin() * rz * hscale], leaf);
        }

        if lat == 0 {
            // Bottom point
            // Just close with a fan to center bottom
            let bottom = b.vertex_offset();
            b.push_vert([0.0, -ry, 0.0], leaf);
            for i in 0..lon_segs {
                let a = ring_start + i;
                let bv = ring_start + (i + 1) % lon_segs;
                b.indices.extend_from_slice(&[a, bv, bottom]);
            }
        } else {
            let prev = ring_start - lon_segs;
            b.link_rings(prev, ring_start, lon_segs);
        }
    }

    // ── 4 short branches extending outward ──
    let branch_count = 4;
    let branch_len = 2.0;
    let branch_r = 0.2;
    for i in 0..branch_count {
        let angle = i as f32 / branch_count as f32 * std::f32::consts::TAU;
        let tip_x = (angle + 0.3).cos() * (rx + branch_len);
        let tip_z = (angle + 0.3).sin() * (rz + branch_len);

        // Simple 4-sided branch: a short cylinder segment
        let segs = 4;
        let boff = b.vertex_offset();
        b.push_ring(0.3, branch_r, segs, bark);
        let b1 = b.vertex_offset();
        // Position the tip ring at the end of the branch
        for j in 0..segs {
            let a = j as f32 / segs as f32 * std::f32::consts::TAU;
            b.push_vert(
                [tip_x + a.cos() * branch_r * 0.8, 0.6, tip_z + a.sin() * branch_r * 0.8],
                bark,
            );
        }
        b.link_rings(boff, b1, segs);
    }

    b.build()
}

/// 马尾松 — 尖锥冠 + 直干。锥底 4，高 8。~30 面。
pub fn generate_pine_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;

    let bark = c4(BARK_PINE);
    let leaf = c4(LEAF_PINE);

    // ── Trunk (6-sided cylinder) ──
    let segs = 6;
    let trunk_r = 0.4;
    let trunk_h = 5.0;
    let r0 = b.vertex_offset();
    b.push_ring(-trunk_h, trunk_r, segs, bark);
    let r1 = b.vertex_offset();
    b.push_ring(0.0, trunk_r, segs, bark);
    b.link_rings(r0, r1, segs);

    // ── Cone crown — 4 lat rings ──
    let base_r = 2.0;
    let cone_h = 8.0;
    let lon_segs = 6;

    // Apex
    let apex = b.vertex_offset();
    b.push_vert([0.0, cone_h, 0.0], leaf);

    for lat in 1..=4 {
        let t = lat as f32 / 4.0; // 0.25, 0.5, 0.75, 1.0
        let y = (1.0 - t) * cone_h; // top down
        let r = base_r * t; // 0.25*base_r ... base_r
        let ring = b.vertex_offset();
        b.push_ring(y, r, lon_segs, leaf);

        if lat == 1 {
            for i in 0..lon_segs {
                let a = apex;
                let bv = ring + i;
                let cv = ring + (i + 1) % lon_segs;
                b.indices.extend_from_slice(&[a, bv, cv]);
            }
        } else {
            let prev = ring - lon_segs;
            b.link_rings(prev, ring, lon_segs);
        }
    }

    b.build()
}

/// 毛竹 — 细柱节段 + 顶弯垂带小叶。高 10。~20 面。
pub fn generate_bamboo_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;

    let stem = c4(BAMBOO_COLOR);
    let leaf = c4(LEAF_BAMBOO);
    let segs = 5; // pentagon cross-section
    let seg_count = 5; // 5 segments

    // ── Stem segments ──
    let total_h = 10.0;
    let seg_h = total_h / seg_count as f32;
    let radius = 0.3;

    let mut prev_ring = b.vertex_offset();
    b.push_ring(0.0, radius, segs, stem);

    for s in 1..=seg_count {
        let y = s as f32 * seg_h;
        // Slight taper
        let r = radius * (1.0 - s as f32 / seg_count as f32 * 0.3);
        let ring = b.vertex_offset();
        b.push_ring(y, r, segs, stem);
        b.link_rings(prev_ring, ring, segs);

        // Node bulge
        if s < seg_count {
            let bulge = b.vertex_offset();
            for i in 0..segs {
                let a = i as f32 / segs as f32 * std::f32::consts::TAU;
                b.push_vert(
                    [a.cos() * r * 1.2, y, a.sin() * r * 1.2],
                    stem,
                );
            }
            b.link_rings(ring, bulge, segs);
            let next = b.vertex_offset();
            b.push_ring(y + 0.05, r, segs, stem);
            b.link_rings(bulge, next, segs);
            prev_ring = next;
        } else {
            prev_ring = ring;
        }
    }

    // ── Curved tip ──
    let tip_start = prev_ring;
    let tip_len = 1.5;
    let tip_segs = 3;
    let mut prev = tip_start;
    for s in 1..=tip_segs {
        let t = s as f32 / tip_segs as f32;
        let y = total_h + t * tip_len * 0.7;
        let bend = t * 0.8;
        let r = radius * (1.0 - t * 0.6);
        let ring = b.vertex_offset();
        for i in 0..segs {
            let a = i as f32 / segs as f32 * std::f32::consts::TAU;
            b.push_vert(
                [bend + a.cos() * r, y, a.sin() * r],
                stem,
            );
        }
        b.link_rings(prev, ring, segs);
        prev = ring;
    }

    // ── Leaflets — small triangles at tip ──
    let tip_pos = [
        0.8,
        total_h + tip_len * 0.7 + 0.3,
        0.0,
    ];
    for i in 0..3 {
        let angle = i as f32 / 3.0 * std::f32::consts::TAU + 0.5;
        let lx = tip_pos[0] + angle.cos() * 0.8;
        let lz = angle.sin() * 0.5;
        let lx2 = tip_pos[0] + (angle + 0.5).cos() * 0.4;
        let lz2 = (angle + 0.5).sin() * 0.3;
        let base = b.vertex_offset();
        b.push_vert([tip_pos[0], tip_pos[1] + 0.1, tip_pos[2]], leaf);
        b.push_vert([lx, tip_pos[1] - 0.1, lz], leaf);
        b.push_vert([lx2, tip_pos[1], lz2], leaf);
        b.indices.extend_from_slice(&[base, base + 1, base + 2]);
    }

    b.build()
}

// ═══════════════════════════════════════════════════════════════
//  Shrubs
// ═══════════════════════════════════════════════════════════════

/// 杜鹃 — 3–5 个扁球叠堆。~20 面。
pub fn generate_azalea_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;

    let leaf = c4(LEAF_BROADLEAF);
    let flower_c = c4(LEAF_FLOWER);
    let clusters = 4;
    let lon_segs = 6;
    let lat_segs = 3;

    for ci in 0..clusters {
        let cx = (ci as f32 / clusters as f32 - 0.5) * 0.8;
        let cy = ci as f32 * 0.5;
        let cz = ((ci as i32 * 37) % 7 - 3) as f32 * 0.15;
        let r = 0.5 - ci as f32 * 0.08;
        let color = if ci == 2 { flower_c } else { leaf };

        // Apex
        let apex = b.vertex_offset();
        b.push_vert([cx, cy + r * 0.7, cz], color);

        for lat in 1..=lat_segs {
            let theta = lat as f32 / lat_segs as f32 * std::f32::consts::FRAC_PI_2;
            let hr = r * theta.cos();
            let y = cy + r * theta.sin() * 0.7;
            let ring = b.vertex_offset();
            for i in 0..lon_segs {
                let a = i as f32 / lon_segs as f32 * std::f32::consts::TAU;
                b.push_vert([cx + a.cos() * hr, y, cz + a.sin() * hr], color);
            }
            if lat == 1 {
                for i in 0..lon_segs {
                    let a = apex;
                    let bv = ring + i;
                    let cv = ring + (i + 1) % lon_segs;
                    b.indices.extend_from_slice(&[a, bv, cv]);
                }
            } else {
                let prev = ring - lon_segs;
                b.link_rings(prev, ring, lon_segs);
            }
        }
    }

    b.build()
}

// ═══════════════════════════════════════════════════════════════
//  Grasses / reeds
// ═══════════════════════════════════════════════════════════════

/// 芦苇 — 细柱(高3) + 顶椭球穗。~12 面。
pub fn generate_reed_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;

    let stem_c = c4(REED_COLOR);
    let spike_c = col4([0.55, 0.50, 0.20]);

    // ── Thin stem (4-sided cylinder) ──
    let segs = 4;
    let r = 0.08;
    let h = 3.0;
    let r0 = b.vertex_offset();
    b.push_ring(0.0, r, segs, stem_c);
    let r1 = b.vertex_offset();
    b.push_ring(h, r * 0.7, segs, stem_c);
    b.link_rings(r0, r1, segs);

    // ── Spike — small ellipsoid at top ──
    let spike_rx = 0.2;
    let spike_ry = 0.35;
    let spike_rz = 0.2;
    let lon_segs = 6;
    let lat_segs = 3;

    // Apex
    let apex = b.vertex_offset();
    b.push_vert([0.0, h + spike_ry, 0.0], spike_c);

    for lat in 1..=lat_segs {
        let t = (lat as f32) / (lat_segs as f32) * std::f32::consts::FRAC_PI_2;
        let y = h + spike_ry * t.sin();
        let hr = spike_rx * t.cos();
        let ring = b.vertex_offset();
        for i in 0..lon_segs {
            let a = i as f32 / lon_segs as f32 * std::f32::consts::TAU;
            b.push_vert([a.cos() * hr, y, a.sin() * hr * spike_rz / spike_rx], spike_c);
        }
        if lat == 1 {
            for i in 0..lon_segs {
                let a = apex;
                let bv = ring + i;
                let cv = ring + (i + 1) % lon_segs;
                b.indices.extend_from_slice(&[a, bv, cv]);
            }
        } else {
            let prev = ring - lon_segs;
            b.link_rings(prev, ring, lon_segs);
        }
    }

    // Bottom half of spike
    let equator_ring = b.vertex_offset() - lon_segs;
    let bottom = b.vertex_offset();
    b.push_vert([0.0, h, 0.0], spike_c);
    for i in 0..lon_segs {
        let a = equator_ring + i;
        let bv = equator_ring + (i + 1) % lon_segs;
        b.indices.extend_from_slice(&[a, bottom, bv]);
    }

    b.build()
}

/// 芒草 — 细三角柱集群(高5)，每株 6 面。
pub fn generate_miscanthus_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;

    let col = c4(MISCANTHUS_COLOR);
    let blades = 5;

    for bi in 0..blades {
        let angle = bi as f32 / blades as f32 * std::f32::consts::TAU;
        let bend_x = angle.cos() * 0.3;
        let bend_z = angle.sin() * 0.3;
        let lean = (bi as f32 * 0.7).cos() * 0.3;

        let base = b.vertex_offset();
        b.push_vert([0.0, 0.0, 0.0], col);
        b.push_vert(
            [0.08 + bend_x + lean, 5.0, bend_z + lean * 0.3],
            col,
        );
        b.push_vert(
            [-0.08 + bend_x + lean, 5.0, bend_z + lean * 0.3],
            col,
        );
        b.indices.extend_from_slice(&[base, base + 1, base + 2]);
    }

    b.build()
}

// ═══════════════════════════════════════════════════════════════
//  Aquatic plants
// ═══════════════════════════════════════════════════════════════

/// 莲花 — 扁圆盘 + 中心小凸起。~10 面。
pub fn generate_lotus_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;

    let pad = c4(LOTUS_PAD);
    let center = c4(LOTUS_CENTER);
    let n = 8; // octagon
    let r = 1.0;

    // ── Flat octagon disk ──

    // ── Flat octagon disk ──
    let cv = b.vertex_offset();
    b.push_vert([0.0, 0.0, 0.0], pad);

    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        b.push_vert([a.cos() * r, 0.0, a.sin() * r], pad);
    }

    for i in 0..n {
        let next = (i + 1) % n;
        b.indices.extend_from_slice(&[cv, cv + 1 + i, cv + 1 + next]);
    }

    // ── Center bump — small hemisphere ──
    let bump_r = 0.2;
    let bump_h = 0.15;
    let lon_segs = 6;
    let lat_segs = 2;

    let apex = b.vertex_offset();
    b.push_vert([0.0, bump_h, 0.0], center);

    for lat in 1..=lat_segs {
        let theta = lat as f32 / lat_segs as f32 * std::f32::consts::FRAC_PI_2;
        let r = bump_r * theta.cos();
        let y = bump_h * theta.sin();
        let ring = b.vertex_offset();
        for i in 0..lon_segs {
            let a = i as f32 / lon_segs as f32 * std::f32::consts::TAU;
            b.push_vert([a.cos() * r, y, a.sin() * r], center);
        }
        if lat == 1 {
            for i in 0..lon_segs {
                let a = apex;
                let bv = ring + i;
                let cv = ring + (i + 1) % lon_segs;
                b.indices.extend_from_slice(&[a, bv, cv]);
            }
        } else {
            let prev = ring - lon_segs;
            b.link_rings(prev, ring, lon_segs);
        }
    }

    b.build()
}

/// 水草 — 弯曲线条带。~8 面。
pub fn generate_waterweed_mesh() -> Mesh {
    let mut b = MeshBuilder::new();
    let c4 = col4;

    let col = c4(WATERWEED_COLOR);
    let strips = 3;
    let segs = 4;
    let w = 0.15;
    let h = 2.0;

    for si in 0..strips {
        let x_off = (si as f32 - 1.0) * 0.5;

        // Build a ribbon: 2 vertices per segment side
        let mut prev_a = None;
        let mut prev_b = None;

        for s in 0..=segs {
            let t = s as f32 / segs as f32;
            let y = t * h;
            let wave = (t * std::f32::consts::TAU * 1.5).sin() * 0.3;
            let x = x_off + wave;
            let z = (t * std::f32::consts::TAU * 2.0).sin() * 0.2;

            let a = b.vertex_offset();
            b.push_vert([x - w, y, z], col);
            let bv = b.vertex_offset();
            b.push_vert([x + w, y, z], col);

            if let (Some(pa), Some(pb)) = (prev_a, prev_b) {
                b.indices
                    .extend_from_slice(&[pa, a, bv, pa, bv, pb]);
            }
            prev_a = Some(a);
            prev_b = Some(bv);
        }
    }

    b.build()
}

// ═══════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure every plant mesh is non-empty and has valid attributes.
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

    macro_rules! test_plant {
        ($name:ident, $func:ident) => {
            #[test]
            fn $name() {
                let mesh = $func();
                check_mesh(&mesh, stringify!($func));
            }
        };
    }

    test_plant!(mesh_nanmu, generate_nanmu_mesh);
    test_plant!(mesh_camphor, generate_camphor_mesh);
    test_plant!(mesh_pine, generate_pine_mesh);
    test_plant!(mesh_bamboo, generate_bamboo_mesh);
    test_plant!(mesh_azalea, generate_azalea_mesh);
    test_plant!(mesh_reed, generate_reed_mesh);
    test_plant!(mesh_miscanthus, generate_miscanthus_mesh);
    test_plant!(mesh_lotus, generate_lotus_mesh);
    test_plant!(mesh_waterweed, generate_waterweed_mesh);
}
