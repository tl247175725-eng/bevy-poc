//! Terrain chunk mesh — merge 32×32 grid cells into ~50–80 plane meshes
//! with pure vertex colors. Each chunk groups same-row same-terrain
//! contiguous cells into a rectangle, rendered as a single Mesh2d.

use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::mesh::RenderAssetUsages;

use crate::grid_render::SimWorld;
use crate::terrain::terrain_at;
use crate::terrain_colors::{cell_color, rgba_to_f32};
use crate::visual_config::CELL_SIZE;
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::world_view::WorldRootEntity;

// ── Constants ──────────────────────────────────────────────────

/// Each tile in the mesh = CELL_SIZE × CELL_SIZE (56 px), matching
/// the existing sprite-based terrain grid.
const TILE_SIZE: f32 = CELL_SIZE;

/// Map center in world pixels (grid 16,16).
const CENTER_X: f32 = GRID_WIDTH as f32 * TILE_SIZE * 0.5;
const CENTER_Y: f32 = GRID_HEIGHT as f32 * TILE_SIZE * 0.5;

/// Ring radii (in grid-cells) → Z elevation offset.
/// Chebyshev distance from cell center to map center.
const RING_TABLE: &[(f32, f32)] = &[
    (1.5, -0.5),  // deep pool
    (3.5, -0.3),
    (7.0, -0.1),
    (12.0, 0.0),  // grassland baseline
    (19.0, 0.1),
    (25.0, 0.3),
    (99.0, 0.5),  // mountain wall
];

// ── Components ─────────────────────────────────────────────────

/// Marker on each spawned terrain-chunk entity.
#[derive(Component)]
pub struct TerrainChunkTag;

// ── Chunk data ─────────────────────────────────────────────────

#[derive(Debug)]
struct TerrainChunk {
    x_start: u8,          // inclusive
    x_end: u8,            // exclusive
    y_start: u8,          // inclusive
    y_end: u8,            // exclusive
}

/// Return Chebyshev distance (in grid-cells) from map center.
fn ring_distance(gx: u8, gy: u8) -> f32 {
    let cx = GRID_WIDTH as f32 * 0.5;
    let cy = GRID_HEIGHT as f32 * 0.5;
    let dx = gx as f32 - cx;
    let dy = gy as f32 - cy;
    dx.abs().max(dy.abs())
}

/// Z offset for a cell based on its ring distance from center.
fn ring_height(gx: u8, gy: u8) -> f32 {
    let dist = ring_distance(gx, gy);
    for &(radius, height) in RING_TABLE {
        if dist <= radius {
            return height;
        }
    }
    RING_TABLE.last().map(|&(_, h)| h).unwrap_or(0.0)
}

// ── Chunk merging ──────────────────────────────────────────────

/// Greedy scan of the 32×32 grid. Same-row contiguous same-terrain
/// cells are merged into a row-run chunk (no vertical merging for
/// simplicity and correctness).
fn build_chunks(sim: &SimWorld) -> Vec<TerrainChunk> {
    let mut visited = [[false; GRID_WIDTH as usize]; GRID_HEIGHT as usize];
    let mut chunks = Vec::new();

    for gy in 0..GRID_HEIGHT {
        let mut gx = 0u8;
        while gx < GRID_WIDTH {
            if visited[gy as usize][gx as usize] {
                gx += 1;
                continue;
            }

            let terrain = terrain_at(&sim.0, gx, gy);

            // ── expand right ──
            let mut x_end = gx;
            while x_end < GRID_WIDTH
                && terrain_at(&sim.0, x_end, gy) == terrain
                && !visited[gy as usize][x_end as usize]
            {
                x_end += 1;
            }
            if x_end <= gx {
                gx += 1;
                continue;
            }

            // ── expand down ──
            let mut y_end = gy + 1;
            while y_end < GRID_HEIGHT {
                let mut all_match = true;
                for col in gx..x_end {
                    if visited[y_end as usize][col as usize]
                        || terrain_at(&sim.0, col, y_end) != terrain
                    {
                        all_match = false;
                        break;
                    }
                }
                if !all_match {
                    break;
                }
                y_end += 1;
            }

            // ── mark visited ──
            for row in gy..y_end {
                for col in gx..x_end {
                    visited[row as usize][col as usize] = true;
                }
            }

            chunks.push(TerrainChunk {
                x_start: gx,
                x_end,
                y_start: gy,
                y_end,
            });

            gx = x_end;
        }
    }

    chunks
}

// ── Mesh generation ────────────────────────────────────────────

/// Build a single plane mesh from a chunk rectangle.
///
/// Each grid cell is a quad (2 tris). Vertex position in world
/// pixels (matching `cell_center`), Z from ring height. Vertex
/// color from `cell_color`.
fn build_chunk_mesh(
    chunk: &TerrainChunk,
    sim: &SimWorld,
) -> Mesh {
    let w = (chunk.x_end - chunk.x_start) as f32 * TILE_SIZE;
    let h = (chunk.y_end - chunk.y_start) as f32 * TILE_SIZE;
    let base_x = chunk.x_start as f32 * TILE_SIZE;
    let base_y = chunk.y_start as f32 * TILE_SIZE;

    // Use the average ring height across the chunk for consistent Z.
    // Sample center cell(s) for Z.
    let center_gx = (chunk.x_start + chunk.x_end) / 2;
    let center_gy = (chunk.y_start + chunk.y_end) / 2;
    let z_offset = ring_height(center_gx, center_gy);

    let cols = (chunk.x_end - chunk.x_start) as usize;
    let rows = (chunk.y_end - chunk.y_start) as usize;
    let verts_wide = cols + 1;
    let verts_high = rows + 1;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(verts_wide * verts_high);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(verts_wide * verts_high);
    let mut indices: Vec<u32> = Vec::with_capacity(cols * rows * 6);

    // Pre-compute per-cell vertex colors (4 corners each).
    // For a clean look, each quad gets the color of its cell.
    // Corner vertices shared by 4 quads → pick the top-left cell's
    // colour for each corner.
    for row in 0..=rows {
        let gy = chunk.y_start + row as u8;
        for col in 0..=cols {
            let gx = chunk.x_start + col as u8;
            let px = base_x + col as f32 * TILE_SIZE;
            let py = base_y + row as f32 * TILE_SIZE;

            positions.push([px, py, z_offset]);

            // Pick colour from the cell at the corner's top-left
            // (clamped to valid range).
            let cell_x = gx.min(GRID_WIDTH - 1);
            let cell_y = gy.min(GRID_HEIGHT - 1);
            let (r, g, b, a) = rgba_to_f32(cell_color(&sim.0, cell_x, cell_y));
            colors.push([r, g, b, a]);
        }
    }

    // Build triangle strip-style indices.
    for row in 0..rows {
        for col in 0..cols {
            let a = (row * verts_wide + col) as u32;
            let b = a + 1;
            let c = a + verts_wide as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

// ── Startup system ─────────────────────────────────────────────

/// Generate terrain chunk meshes on startup.
pub fn spawn_terrain_chunks(
    mut commands: Commands,
    sim: Res<SimWorld>,
    root: Res<WorldRootEntity>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let chunks = build_chunks(&sim);
    let white_material = materials.add(ColorMaterial::from(Color::WHITE));

    for chunk in &chunks {
        let mesh = build_chunk_mesh(chunk, &sim);
        let mesh_handle = meshes.add(mesh);

        commands.entity(root.0).with_children(|parent| {
            parent.spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(white_material.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, -5.0)),
                Visibility::default(),
                TerrainChunkTag,
            ));
        });
    }

    info!(
        "terrain chunks: {} total ({}×{} grid)",
        chunks.len(),
        GRID_WIDTH,
        GRID_HEIGHT,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal WorldState with just the grid dimensions and
    /// terrain info needed for chunking.
    fn make_test_state() -> crate::world_state::WorldState {
        let mut state = crate::world_state::WorldState::new(
            crate::card_def::card_defs_map(
                &crate::card_def::load_card_defs("assets/card_defs.ron"),
            ),
        );
        // Ensure ecology is ready so terrain_at uses it.
        state.ecology.ready = true;
        state
    }

    #[test]
    fn chunk_count_within_bounds() {
        let state = make_test_state();
        let sim = SimWorld(state);
        let chunks = build_chunks(&sim);
        assert!(!chunks.is_empty(), "should have at least one chunk");
        assert!(
            chunks.len() <= 80,
            "chunk count {} should be ≤ 80",
            chunks.len()
        );
    }

    #[test]
    fn chunks_cover_all_cells() {
        let state = make_test_state();
        let sim = SimWorld(state);
        let chunks = build_chunks(&sim);

        let mut covered = [[false; GRID_WIDTH as usize]; GRID_HEIGHT as usize];
        for chunk in &chunks {
            for y in chunk.y_start..chunk.y_end {
                for x in chunk.x_start..chunk.x_end {
                    covered[y as usize][x as usize] = true;
                }
            }
        }

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                assert!(
                    covered[y as usize][x as usize],
                    "cell ({},{}) not covered by any chunk",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn chunk_no_overlap() {
        let state = make_test_state();
        let sim = SimWorld(state);
        let chunks = build_chunks(&sim);

        let mut seen = [[false; GRID_WIDTH as usize]; GRID_HEIGHT as usize];
        for chunk in &chunks {
            for y in chunk.y_start..chunk.y_end {
                for x in chunk.x_start..chunk.x_end {
                    assert!(
                        !seen[y as usize][x as usize],
                        "cell ({},{}) appears in multiple chunks",
                        x,
                        y
                    );
                    seen[y as usize][x as usize] = true;
                }
            }
        }
    }

    #[test]
    fn same_terrain_in_chunk() {
        let state = make_test_state();
        let sim = SimWorld(state);
        let chunks = build_chunks(&sim);

        for chunk in &chunks {
            // All cells in this chunk must have the same terrain type.
            let first_terrain = terrain_at(
                &sim.0,
                chunk.x_start,
                chunk.y_start,
            );
            for y in chunk.y_start..chunk.y_end {
                for x in chunk.x_start..chunk.x_end {
                    assert_eq!(
                        terrain_at(&sim.0, x, y),
                        first_terrain,
                        "chunk ({}-{},{}-{}) contains mixed terrain at ({},{})",
                        chunk.x_start, chunk.x_end,
                        chunk.y_start, chunk.y_end,
                        x, y,
                    );
                }
            }
        }
    }

    #[test]
    fn mesh_has_valid_attributes() {
        let state = make_test_state();
        let sim = SimWorld(state);
        let chunks = build_chunks(&sim);

        for chunk in &chunks {
            let mesh = build_chunk_mesh(chunk, &sim);
            assert!(
                mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some(),
                "missing POSITION"
            );
            assert!(
                mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some(),
                "missing COLOR"
            );
            assert!(
                mesh.indices().is_some(),
                "missing INDICES"
            );

            // Each quad = 2 tris = 6 indices; 4 vertices per quad
            // (shared corners reduce total verts).
            let vert_count = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().len();
            let idx_count = mesh.indices().unwrap().len();
            assert!(vert_count >= 4, "need at least 4 vertices, got {vert_count}");
            assert!(idx_count >= 6, "need at least 6 indices, got {idx_count}");
            assert_eq!(idx_count % 3, 0, "indices must be multiple of 3");
        }
    }
}
