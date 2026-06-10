//! Single coordinate API — Godot grid (Y down) ↔ world pixels.
//! All input/render code should use these helpers instead of duplicating math.

use bevy::prelude::*;

use crate::visual_config::CELL_SIZE;
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;
use crate::world_view::WorldView;
use crate::viewport_layout::ViewportLayout;

/// Unified screen ↔ Godot-world ↔ grid conversions (see `WorldView` for zoom/pan).
pub struct CoordinateSystem;

impl CoordinateSystem {
    pub fn area_to_godot(area_pos: Vec2, layout: &ViewportLayout, view: &WorldView) -> Option<Vec2> {
        if !layout.cursor_in_world_area(area_pos) {
            return None;
        }
        Some(view.area_to_world(area_pos, layout.world_area_size()))
    }

    pub fn godot_to_area(world: Vec2, layout: &ViewportLayout, view: &WorldView) -> Vec2 {
        view.world_to_area(world, layout.world_area_size())
    }

    pub fn area_to_grid(area_pos: Vec2, layout: &ViewportLayout, view: &WorldView) -> Option<(u8, u8)> {
        world_to_grid(Self::area_to_godot(area_pos, layout, view)?)
    }

    pub fn grid_center_to_area(
        gx: u8,
        gy: u8,
        layout: &ViewportLayout,
        view: &WorldView,
    ) -> Vec2 {
        Self::godot_to_area(cell_center(gx, gy).truncate(), layout, view)
    }
}

/// Cell center in Godot-style world pixels (Y down).
pub fn cell_center(x: u8, y: u8) -> Vec3 {
    Vec3::new(
        x as f32 * CELL_SIZE + CELL_SIZE * 0.5,
        y as f32 * CELL_SIZE + CELL_SIZE * 0.5,
        0.0,
    )
}

/// Godot `visual_world_pos_for_cell` — card center matches terrain cell center.
pub fn card_world_pos(
    x: u8,
    y: u8,
    entity_id: u64,
    _world: Option<&WorldState>,
) -> Vec3 {
    let center = cell_center(x, y);
    Vec3::new(center.x, center.y, 10.0 + entity_id as f32 * 0.001)
}

pub fn grid_to_world(x: u8, y: u8) -> Vec3 {
    card_world_pos(x, y, 0, None)
}

pub fn grid_to_world_in(sim: &WorldState, x: u8, y: u8, entity_id: u64) -> Vec3 {
    card_world_pos(x, y, entity_id, Some(sim))
}

pub fn world_to_grid(world_pos: Vec2) -> Option<(u8, u8)> {
    let gx = (world_pos.x / CELL_SIZE).floor() as i32;
    let gy = (world_pos.y / CELL_SIZE).floor() as i32;
    if gx >= 0 && gx < GRID_WIDTH as i32 && gy >= 0 && gy < GRID_HEIGHT as i32 {
        Some((gx as u8, gy as u8))
    } else {
        None
    }
}

pub fn cursor_to_world(cursor: Vec2, layout: &ViewportLayout, view: &WorldView) -> Option<Vec2> {
    CoordinateSystem::area_to_godot(cursor, layout, view)
}

pub fn grid_from_cursor(
    cursor: Vec2,
    layout: &ViewportLayout,
    view: &WorldView,
) -> Option<(u8, u8)> {
    CoordinateSystem::area_to_grid(cursor, layout, view)
}

/// Round-trip: grid → world center → grid.
pub fn grid_round_trip(x: u8, y: u8) -> Option<(u8, u8)> {
    let center = cell_center(x, y).truncate();
    world_to_grid(center)
}

/// Zoom anchor invariant: world point under cursor stays fixed after zoom.
pub fn zoom_anchor_invariant(
    view: &mut WorldView,
    area_mouse: Vec2,
    area_size: Vec2,
    wheel_dir: i32,
) -> Vec2 {
    let before = view.area_to_world(area_mouse, area_size);
    view.zoom_wheel(wheel_dir, area_mouse, area_size);
    view.area_to_world(area_mouse, area_size) - before
}
