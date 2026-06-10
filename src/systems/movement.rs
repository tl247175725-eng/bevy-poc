use crate::pathfinding::find_path;
use crate::spatial_index::EntityId;
use crate::world_rules::{chebyshev_distance, GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::{MoveResult, WorldState};

pub fn move_toward(world: &mut WorldState, id: EntityId, x: u8, y: u8, tx: u8, ty: u8) {
    if x == tx && y == ty {
        return;
    }
    let dx = (tx as i16 - x as i16).signum();
    let dy = (ty as i16 - y as i16).signum();
    let gx = (x as i16 + dx).clamp(0, GRID_WIDTH as i16 - 1) as u8;
    let gy = (y as i16 + dy).clamp(0, GRID_HEIGHT as i16 - 1) as u8;
    if (gx != x || gy != y) && !crate::pathfinding::is_blocked_for(world, gx, gy, Some(id.0)) {
        if world.move_entity(id, gx, gy) == crate::world_state::MoveResult::Moved {
            return;
        }
    }
    if let Some((nx, ny)) = find_path(world, x, y, tx, ty, Some(id.0)).first().copied() {
        if world.move_entity(id, nx, ny) == crate::world_state::MoveResult::Moved {
            return;
        }
    }
    move_toward_greedy(world, id, x, y, tx, ty);
}

fn move_toward_greedy(world: &mut WorldState, id: EntityId, x: u8, y: u8, tx: u8, ty: u8) {
    let dx = (tx as i16 - x as i16).signum();
    let dy = (ty as i16 - y as i16).signum();
    let nx = (x as i16 + dx).clamp(0, GRID_WIDTH as i16 - 1) as u8;
    let ny = (y as i16 + dy).clamp(0, GRID_HEIGHT as i16 - 1) as u8;
    if world.move_entity(id, nx, ny) == crate::world_state::MoveResult::Moved {
        return;
    }
}

pub fn flee_from(world: &mut WorldState, id: EntityId, x: u8, y: u8, tx: u8, ty: u8) {
    let mut dx = (x as i16 - tx as i16).signum();
    let mut dy = (y as i16 - ty as i16).signum();
    if dx == 0 && dy == 0 {
        if x > 0 {
            dx = -1;
        } else if (x as u16 + 1) < GRID_WIDTH as u16 {
            dx = 1;
        } else if y > 0 {
            dy = -1;
        } else if (y as u16 + 1) < GRID_HEIGHT as u16 {
            dy = 1;
        }
    }
    let nx = (x as i16 + dx).clamp(0, GRID_WIDTH as i16 - 1) as u8;
    let ny = (y as i16 + dy).clamp(0, GRID_HEIGHT as i16 - 1) as u8;
    if world.move_entity(id, nx, ny) == MoveResult::Moved {
        return;
    }
    let alt_nx = (x as i16 + dy).clamp(0, GRID_WIDTH as i16 - 1) as u8;
    let alt_ny = (y as i16 + dx).clamp(0, GRID_HEIGHT as i16 - 1) as u8;
    if (alt_nx != x || alt_ny != y) && (alt_nx != nx || alt_ny != ny) {
        world.move_entity(id, alt_nx, alt_ny);
    }
}

pub fn nearest_of(world: &WorldState, x: u8, y: u8, ids: &[EntityId]) -> Option<(EntityId, u8)> {
    let mut best: Option<(EntityId, u8)> = None;
    for &id in ids {
        if let Some((ex, ey)) = world.spatial_index.position(id) {
            let d = chebyshev_distance(x, y, ex, ey);
            if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                best = Some((id, d));
            }
        }
    }
    best
}

pub fn wander(world: &mut WorldState, id: EntityId, x: u8, y: u8, tick: u64) {
    let nx = if (tick + id.0) % 2 == 0 && x + 1 < GRID_WIDTH {
        x + 1
    } else if x > 0 {
        x - 1
    } else {
        x
    };
    let _ = world.move_entity(id, nx, y);
}

/// Godot `_find_safe_land_near` — ring search for walkable empty cell.
pub fn find_safe_land_near(world: &WorldState, x: u8, y: u8) -> Option<(u8, u8)> {
    for r in 1..7u8 {
        for dx in -(r as i16)..=(r as i16) {
            for dy in -(r as i16)..=(r as i16) {
                let adx = dx.unsigned_abs() as u8;
                let ady = dy.unsigned_abs() as u8;
                if adx.max(ady) != r {
                    continue;
                }
                let nx = x as i16 + dx;
                let ny = y as i16 + dy;
                if nx < 0
                    || ny < 0
                    || nx >= GRID_WIDTH as i16
                    || ny >= GRID_HEIGHT as i16
                {
                    continue;
                }
                let ux = nx as u8;
                let uy = ny as u8;
                let terrain = crate::terrain::terrain_at(world, ux, uy);
                if matches!(terrain, "river" | "ford" | "barren" | "pool") {
                    continue;
                }
                if !world.entities_at(ux, uy).is_empty() {
                    continue;
                }
                return Some((ux, uy));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn path_moves_around_obstacle() {
        let mut w = empty_world();
        w.spawn("mountain", 6, 5);
        let id = w.spawn("sheep", 5, 5);
        move_toward(&mut w, id, 5, 5, 7, 5);
        let e = &w.entities[&id];
        assert!(e.x == 5 || e.x == 6 || e.x == 7);
    }
}
