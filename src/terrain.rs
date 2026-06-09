use crate::terrain_colors::pond_state_label;
use crate::terrain_ecology::MapEcology;
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;

pub fn terrain_at(world: &WorldState, x: u8, y: u8) -> &'static str {
    if world.fire_cells.contains(&(x, y)) {
        return "wasteland";
    }
    base_terrain_at(world, x, y)
}

/// Underlying cell type without fire/wasteland overlay — Godot `TerrainManager.get_cell_type`.
pub fn base_terrain_at(world: &WorldState, x: u8, y: u8) -> &'static str {
    if world.ecology.ready {
        return world.ecology.base_cell_type(x, y);
    }
    legacy_terrain_at(world, x, y)
}

fn legacy_terrain_at(world: &WorldState, x: u8, y: u8) -> &'static str {
    if world.pool_cells.contains(&(x, y)) {
        return "pool";
    }
    if world.river_cells.contains(&(x, y)) {
        return "river";
    }
    if x == 0 || x == GRID_WIDTH - 1 || y == GRID_HEIGHT - 1 {
        return "barren";
    }
    "land"
}

pub fn terrain_label(terrain: &str) -> &'static str {
    match terrain {
        "grassland" => "草地",
        "riverbank" => "河岸",
        "bank" => "河岸",
        "river" => "河沟",
        "ford" => "浅滩",
        "pool" => "水潭",
        "dark_river_pool" => "暗河水潭",
        "wetland" => "湿地",
        "land" => "荒地",
        "wasteland" => "焦土",
        "barren" => "边界",
        _ => "未知地形",
    }
}

pub const ELEVATION_VISUAL_PX_PER_LEVEL: f32 = 2.0;

pub fn elevation_visual_offset_y(world: &WorldState, x: u8, y: u8) -> f32 {
    -(cell_elevation(world, x, y) as f32 * ELEVATION_VISUAL_PX_PER_LEVEL)
}

/// Godot `TerrainManager.surface_label_at` + `world_rules_ui.gd`.
pub fn surface_label_with_stress(
    world: &WorldState,
    x: u8,
    y: u8,
    river_stress: f32,
) -> Option<String> {
    if x == 0 || y == 0 || x >= GRID_WIDTH - 1 || y >= GRID_HEIGHT - 1 {
        return None;
    }
    if let Some(overlay) = cell_overlay_label(world, x, y) {
        return Some(overlay);
    }
    let base = terrain_at(world, x, y);
    let pond = pond_state_label(river_stress);
    let label = match base {
        "pool" => "水潭".to_string(),
        "ford" => format!("浅滩·{pond}"),
        "river" => format!("河沟·{pond}"),
        "bank" | "wetland" | "riverbank" => "湿润土地".into(),
        "grassland" | "riparian" => "湿润土地".into(),
        "land" => {
            if world.ecology.ready && world.ecology.is_riparian_grass_cell(x, y) {
                "湿润土地".into()
            } else {
                "荒地".into()
            }
        }
        "wasteland" => "焦土".into(),
        "barren" => return None,
        _ => return None,
    };
    Some(label)
}

/// Dynamic overlay labels — Godot `world_rules_ui.gd` / den overlays.
pub fn surface_label(world: &WorldState, x: u8, y: u8) -> Option<String> {
    surface_label_with_stress(world, x, y, 0.0)
}

fn cell_overlay_label(world: &WorldState, x: u8, y: u8) -> Option<String> {
    for entity in world.entities.values() {
        if entity.x != x || entity.y != y {
            continue;
        }
        match entity.type_name.as_str() {
            "wolfDen" => return Some("狼穴".into()),
            "foxDen" => return Some("狐窝".into()),
            "birdNest" => return Some("鸟巢".into()),
            "humus" => return Some("腐殖".into()),
            _ => {}
        }
    }
    None
}

pub fn is_blocked_terrain(terrain: &str) -> bool {
    matches!(terrain, "pool" | "river" | "barren" | "dark_river_pool")
}

pub fn cell_elevation(world: &WorldState, x: u8, y: u8) -> i32 {
    if world.ecology.ready {
        world.ecology.elevation_at(x, y)
    } else {
        0
    }
}

pub fn ecology(world: &WorldState) -> &MapEcology {
    &world.ecology
}
