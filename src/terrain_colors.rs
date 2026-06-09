/// Terrain cell colors from Godot `terrain_visual_palette.gd`.

use crate::terrain::terrain_at;
use crate::world_state::WorldState;

fn hex_rgb(hex: &str) -> (u8, u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return (234, 215, 171, 255);
    }
    let parse = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0);
    (
        parse(&hex[0..2]),
        parse(&hex[2..4]),
        parse(&hex[4..6]),
        255,
    )
}

pub fn terrain_color(terrain_type: &str) -> (u8, u8, u8, u8) {
    match terrain_type {
        "barren" => hex_rgb("8b7d6b"),
        "pool" | "dark_river_pool" => hex_rgb("1a3040"),
        "river" => hex_rgb("7eb8c8"),
        "ford" => hex_rgb("9ec8d4"),
        "bank" | "wetland" | "riverbank" => hex_rgb("a8c9a0"),
        "riparian" | "grassland" => hex_rgb("c5dbb8"),
        "wasteland" | "land" => hex_rgb("ead7ab"),
        _ => hex_rgb("ead7ab"),
    }
}

fn river_color_stressed(cell_type: &str, stress: f32) -> (u8, u8, u8, u8) {
    if cell_type == "ford" {
        return hex_rgb("9ec8d4");
    }
    if stress >= 70.0 {
        hex_rgb("6a9098")
    } else if stress >= 35.0 {
        hex_rgb("7aa8b0")
    } else {
        hex_rgb("7eb8c8")
    }
}

/// Godot `TerrainVisualPalette.cell_color` / `pool_color_at`.
pub fn cell_color(world: &WorldState, x: u8, y: u8) -> (u8, u8, u8, u8) {
    let cell_type = terrain_at(world, x, y);
    match cell_type {
        "pool" => pool_color_at(world, x, y),
        "bank" | "wetland" => hex_rgb("a8c9a0"),
        "river" | "ford" => river_color(world, x, y, cell_type),
        "land" => {
            if world.ecology.ready && world.ecology.is_riparian_grass_cell(x, y) {
                hex_rgb("c5dbb8")
            } else {
                hex_rgb("ead7ab")
            }
        }
        "barren" => hex_rgb("8b7d6b"),
        _ => hex_rgb("ead7ab"),
    }
}

fn pool_color_at(world: &WorldState, x: u8, y: u8) -> (u8, u8, u8, u8) {
    if world.ecology.ready {
        if world.ecology.underground_river_role(x, y) == "dark_river_pool" {
            return hex_rgb("1a3040");
        }
        match world.ecology.pool_manhattan_dist(x, y) {
            0 => hex_rgb("1a3040"),
            1 => hex_rgb("2a6a82"),
            2 => hex_rgb("3a8aa4"),
            3 => hex_rgb("4aa8c4"),
            4 => hex_rgb("6ec0dc"),
            5 => hex_rgb("9ec8d4"),
            _ => hex_rgb("6ec0dc"),
        }
    } else {
        terrain_color("pool")
    }
}

fn river_color(world: &WorldState, x: u8, y: u8, cell_type: &str) -> (u8, u8, u8, u8) {
    if world.ecology.ready {
        let role = world.ecology.underground_river_role(x, y);
        if role == "underground_river_entrance" || role == "dark_river_pool" {
            return hex_rgb("1a3040");
        }
    }
    if cell_type == "ford" {
        hex_rgb("9ec8d4")
    } else {
        hex_rgb("7eb8c8")
    }
}

pub const SELECTION_BORDER: (u8, u8, u8, u8) = (22, 129, 58, 255);

/// River stress tint for pool/bank cells — Godot `terrain_manager.update_pond_cells`.
pub fn cell_color_with_stress(
    world: &WorldState,
    x: u8,
    y: u8,
    river_stress: f32,
) -> (u8, u8, u8, u8) {
    let base = cell_color(world, x, y);
    let terrain = terrain_at(world, x, y);
    if terrain == "river" {
        return river_color_stressed("river", river_stress);
    }
    if !matches!(terrain, "pool" | "bank" | "wetland" | "riverbank" | "ford") {
        return base;
    }
    let stress = river_stress.clamp(0.0, 100.0) / 100.0;
    if stress < 0.05 {
        return base;
    }
    let (r, g, b, a) = base;
    let tint = (stress * 40.0) as i32;
    (
        r.saturating_sub(tint as u8),
        g.saturating_sub((tint / 2) as u8),
        b.saturating_sub((tint / 3) as u8),
        a,
    )
}

/// Godot `update_pond_cells` / `pond_state` — 稳 / 低 / 紧
pub fn pond_state_label(stress: f32) -> &'static str {
    if stress >= 70.0 {
        "紧"
    } else if stress >= 35.0 {
        "低"
    } else {
        "稳"
    }
}

pub fn river_stress_label(stress: f32) -> &'static str {
    pond_state_label(stress)
}

pub fn rgba_to_f32(c: (u8, u8, u8, u8)) -> (f32, f32, f32, f32) {
    (
        c.0 as f32 / 255.0,
        c.1 as f32 / 255.0,
        c.2 as f32 / 255.0,
        c.3 as f32 / 255.0,
    )
}
