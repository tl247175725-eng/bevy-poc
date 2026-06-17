//! Vegetation scatter — scan WorldState entities and spawn 3D mesh
//! instances with per-cell random offset. All plants share meshes via
//! Bevy 0.15 GPU instancing (same Handle<Mesh> + Handle<StandardMaterial>
//! → single draw call per species).
//!
//! ## Startup flow
//! 1. `setup_vegetation_meshes` → generate all 9 plant meshes.
//! 2. `scatter_vegetation` → iterate entities, spawn instances at grid (x,y).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::coords::cell_center;
use crate::grid_render::SimWorld;
use crate::visual_config::CELL_SIZE;
use crate::world_view::WorldRootEntity;

use super::vegetation::{
    generate_azalea_mesh, generate_bamboo_mesh, generate_camphor_mesh, generate_lotus_mesh,
    generate_miscanthus_mesh, generate_nanmu_mesh, generate_pine_mesh, generate_reed_mesh,
    generate_waterweed_mesh,
};

// ═══════════════════════════════════════════════════════════════
//  Constants
// ═══════════════════════════════════════════════════════════════

/// Jitter amplitude as fraction of CELL_SIZE.
const JITTER_FRAC: f32 = 0.35;

// ═══════════════════════════════════════════════════════════════
//  Components & Resources
// ═══════════════════════════════════════════════════════════════

/// Marker on each spawned vegetation mesh instance.
#[derive(Component)]
pub struct VegetationTag;

/// Pre-generated mesh handles, keyed by short name (e.g. "nanmu").
#[derive(Resource)]
pub struct VegetationMeshes(pub HashMap<String, Handle<Mesh>>);

/// Shared white material with vertex colors enabled.
#[derive(Resource)]
pub struct VegetationMaterial(pub Handle<StandardMaterial>);

// ═══════════════════════════════════════════════════════════════
//  Seed helpers
// ═══════════════════════════════════════════════════════════════

/// Simple splitmix-style hash for deterministic jitter from entity ID.
fn hash64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// Deterministic offset within cell derived from entity ID.
fn jitter_xy(id: u64) -> Vec2 {
    let h = hash64(id);
    let ox = ((h >> 32) as f32 / u32::MAX as f32 - 0.5) * 2.0 * CELL_SIZE * JITTER_FRAC;
    let oy = ((h & 0xffffffff) as f32 / u32::MAX as f32 - 0.5) * 2.0 * CELL_SIZE * JITTER_FRAC;
    Vec2::new(ox, oy)
}

/// Deterministic scale in [0.8, 1.2] from entity ID.
fn scale_for(id: u64) -> f32 {
    0.8 + (hash64(id.wrapping_mul(3)) >> 32) as f32 / u32::MAX as f32 * 0.4
}

// ═══════════════════════════════════════════════════════════════
//  Type mapping
// ═══════════════════════════════════════════════════════════════

/// World entity type_name → short mesh key.
fn entity_to_mesh_key(type_name: &str) -> Option<&'static str> {
    match type_name {
        "nanmu_tree" => Some("nanmu"),
        "camphor_tree" => Some("camphor"),
        "pine_forest" => Some("pine"),
        "bamboo" => Some("bamboo"),
        "reed" => Some("reed"),
        "miscanthus" => Some("miscanthus"),
        "lotus" => Some("lotus"),
        "azalea" => Some("azalea"),
        "waterweed" => Some("waterweed"),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════
//  Startup — mesh generation
// ═══════════════════════════════════════════════════════════════

/// Generate all 9 plant meshes and store handles.
pub fn setup_vegetation_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut map = HashMap::new();

    macro_rules! insert_mesh {
        ($name:expr, $func:ident) => {
            map.insert($name.to_string(), meshes.add($func()));
        };
    }

    insert_mesh!("nanmu", generate_nanmu_mesh);
    insert_mesh!("camphor", generate_camphor_mesh);
    insert_mesh!("pine", generate_pine_mesh);
    insert_mesh!("bamboo", generate_bamboo_mesh);
    insert_mesh!("reed", generate_reed_mesh);
    insert_mesh!("miscanthus", generate_miscanthus_mesh);
    insert_mesh!("lotus", generate_lotus_mesh);
    insert_mesh!("azalea", generate_azalea_mesh);
    insert_mesh!("waterweed", generate_waterweed_mesh);

    commands.insert_resource(VegetationMeshes(map));

    // White StandardMaterial — vertex colours from mesh are used
    // automatically when ATTRIBUTE_COLOR is present.
    let mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    commands.insert_resource(VegetationMaterial(mat));
}

// ═══════════════════════════════════════════════════════════════
//  Startup — scatter instances
// ═══════════════════════════════════════════════════════════════

/// Iterate all non-corpse entities, match vegetation types, and spawn
/// one Mesh3d instance per entity with cell-internal jitter.
pub fn scatter_vegetation(
    mut commands: Commands,
    sim: Res<SimWorld>,
    vegetation_meshes: Res<VegetationMeshes>,
    vegetation_material: Res<VegetationMaterial>,
    root: Res<WorldRootEntity>,
) {
    let mesh_map = &vegetation_meshes.0;
    let material = &vegetation_material.0;

    let mut count: usize = 0;

    for entity in sim.0.entities.values() {
        if entity.is_corpse {
            continue;
        }
        let Some(mesh_key) = entity_to_mesh_key(&entity.type_name) else {
            continue;
        };
        let Some(mesh_handle) = mesh_map.get(mesh_key) else {
            continue;
        };

        let base = cell_center(entity.x, entity.y);
        let j = jitter_xy(entity.id.0);
        let scale = scale_for(entity.id.0);

        commands.entity(root.0).with_children(|parent| {
            parent.spawn((
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(base.x + j.x, base.y + j.y, base.z + 0.5)
                    .with_scale(Vec3::splat(scale)),
                VegetationTag,
            ));
        });

        count += 1;
    }

    info!("scattered {count} vegetation instances");
}

// ═══════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_to_mesh_key_known() {
        assert_eq!(entity_to_mesh_key("nanmu_tree"), Some("nanmu"));
        assert_eq!(entity_to_mesh_key("camphor_tree"), Some("camphor"));
        assert_eq!(entity_to_mesh_key("pine_forest"), Some("pine"));
        assert_eq!(entity_to_mesh_key("bamboo"), Some("bamboo"));
        assert_eq!(entity_to_mesh_key("reed"), Some("reed"));
        assert_eq!(entity_to_mesh_key("miscanthus"), Some("miscanthus"));
        assert_eq!(entity_to_mesh_key("lotus"), Some("lotus"));
        assert_eq!(entity_to_mesh_key("azalea"), Some("azalea"));
        assert_eq!(entity_to_mesh_key("waterweed"), Some("waterweed"));
    }

    #[test]
    fn entity_to_mesh_key_unknown() {
        assert_eq!(entity_to_mesh_key("player"), None);
        assert_eq!(entity_to_mesh_key("fox"), None);
        assert_eq!(entity_to_mesh_key(""), None);
    }

    #[test]
    fn hash64_deterministic() {
        assert_eq!(hash64(42), hash64(42));
        assert_ne!(hash64(42), hash64(43));
    }

    #[test]
    fn jitter_within_cell() {
        let j = jitter_xy(12345);
        let half = CELL_SIZE * JITTER_FRAC;
        assert!(j.x >= -half && j.x <= half, "jitter.x out of range: {}", j.x);
        assert!(j.y >= -half && j.y <= half, "jitter.y out of range: {}", j.y);
    }

    #[test]
    fn scale_in_range() {
        for id in [0u64, 1, 100, u64::MAX] {
            let s = scale_for(id);
            assert!(s >= 0.8 && s <= 1.2, "scale {s} out of [0.8, 1.2] for id {id}");
        }
    }
}
