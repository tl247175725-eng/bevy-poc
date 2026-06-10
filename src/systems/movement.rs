use crate::pathfinding::{find_path, is_blocked_for};
use crate::spatial_index::EntityId;
use crate::world_rules::{chebyshev_distance, GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::{EcologyState, MoveResult, WorldState};

fn manhattan_step(dx: i16, dy: i16, pick_x: bool) -> (i16, i16) {
    if dx != 0 && dy != 0 {
        if pick_x {
            (dx.signum(), 0)
        } else {
            (0, dy.signum())
        }
    } else {
        (dx.signum(), dy.signum())
    }
}

fn clamp_cell(x: i16, y: i16) -> (u8, u8) {
    (
        x.clamp(0, GRID_WIDTH as i16 - 1) as u8,
        y.clamp(0, GRID_HEIGHT as i16 - 1) as u8,
    )
}

fn adjacent_cells(x: u8, y: u8) -> [(u8, u8); 4] {
    let xi = x as i16;
    let yi = y as i16;
    let max_x = GRID_WIDTH as i16 - 1;
    let max_y = GRID_HEIGHT as i16 - 1;
    [
        ((xi + 1).clamp(0, max_x) as u8, y),
        ((xi - 1).clamp(0, max_x) as u8, y),
        (x, (yi + 1).clamp(0, max_y) as u8),
        (x, (yi - 1).clamp(0, max_y) as u8),
    ]
}

pub fn get_entity_priority(world: &WorldState, id: EntityId) -> u8 {
    let Some(e) = world.entities.get(&id) else {
        return 0;
    };
    match e.ecology_state {
        EcologyState::Fleeing => 5,
        EcologyState::Hunting => 4,
        EcologyState::SeekingFood => 3,
        EcologyState::Wandering | EcologyState::Patrolling | EcologyState::Scavenging => 2,
        EcologyState::Idle => 1,
        _ => 0,
    }
}

fn with_collision_resolve<R>(world: &mut WorldState, f: impl FnOnce(&mut WorldState) -> R) -> R {
    world.sim_observer_depth = world.sim_observer_depth.saturating_add(1);
    let result = f(world);
    world.sim_observer_depth = world.sim_observer_depth.saturating_sub(1);
    result
}

fn try_shove(
    world: &mut WorldState,
    mover_id: EntityId,
    blocker_id: EntityId,
    gx: u8,
    gy: u8,
    step_dx: i16,
    step_dy: i16,
) -> bool {
    if step_dx == 0 && step_dy == 0 {
        return false;
    }
    let mut push_x = gx as i16 + step_dx;
    let mut push_y = gy as i16 + step_dy;
    let mut dest = None;
    while push_x >= 0
        && push_y >= 0
        && push_x < GRID_WIDTH as i16
        && push_y < GRID_HEIGHT as i16
    {
        let ux = push_x as u8;
        let uy = push_y as u8;
        if world.cell_composition.slot(ux, uy).living_count > 0
            || is_blocked_for(world, ux, uy, Some(blocker_id.0))
        {
            break;
        }
        dest = Some((ux, uy));
        push_x += step_dx;
        push_y += step_dy;
    }
    if let Some((bx, by)) = dest {
        return with_collision_resolve(world, |world| {
            world.move_entity(blocker_id, bx, by) == MoveResult::Moved
                && world.move_entity(mover_id, gx, gy) == MoveResult::Moved
        });
    }
    false
}

fn try_yield_and_enter(
    world: &mut WorldState,
    mover_id: EntityId,
    blocker_id: EntityId,
    gx: u8,
    gy: u8,
) -> bool {
    for (bx, by) in adjacent_cells(gx, gy) {
        if world.cell_composition.slot(bx, by).living_count > 0 {
            continue;
        }
        if is_blocked_for(world, bx, by, Some(blocker_id.0)) {
            continue;
        }
        let ok = with_collision_resolve(world, |world| {
            world.move_entity(blocker_id, bx, by) == MoveResult::Moved
                && world.move_entity(mover_id, gx, gy) == MoveResult::Moved
        });
        if ok {
            return true;
        }
    }
    false
}

fn attempt_move_with_resolution(
    world: &mut WorldState,
    id: EntityId,
    from_x: u8,
    from_y: u8,
    step_dx: i16,
    step_dy: i16,
) -> MoveResult {
    if step_dx == 0 && step_dy == 0 {
        return MoveResult::NoOp;
    }
    let (gx, gy) = clamp_cell(from_x as i16 + step_dx, from_y as i16 + step_dy);
    if gx == from_x && gy == from_y {
        return MoveResult::NoOp;
    }

    if !is_blocked_for(world, gx, gy, Some(id.0)) {
        if world.move_entity(id, gx, gy) == MoveResult::Moved {
            return MoveResult::Moved;
        }
    }

    let blocker_id = world
        .entities_at(gx, gy)
        .into_iter()
        .find(|bid| *bid != id);

    if let Some(blocker_id) = blocker_id {
        let mover_priority = get_entity_priority(world, id);
        let blocker_priority = get_entity_priority(world, blocker_id);

        // Layer 2: Yield — lower-priority blocker steps aside (before dodge).
        if blocker_priority < mover_priority && try_yield_and_enter(world, id, blocker_id, gx, gy) {
            return MoveResult::Moved;
        }

        // Layer 3: Shove — significantly higher priority pushes blocker forward.
        if mover_priority >= blocker_priority.saturating_add(2)
            && try_shove(world, id, blocker_id, gx, gy, step_dx, step_dy)
        {
            return MoveResult::Moved;
        }
    }

    // Layer 1: Dodge — try perpendicular axes (terrain / equal-priority blocks).
    for (ax, ay) in [(step_dy, step_dx), (-step_dy, -step_dx)] {
        if ax == 0 && ay == 0 {
            continue;
        }
        let (alt_x, alt_y) = clamp_cell(from_x as i16 + ax, from_y as i16 + ay);
        if alt_x == from_x && alt_y == from_y {
            continue;
        }
        if is_blocked_for(world, alt_x, alt_y, Some(id.0)) {
            continue;
        }
        if world.move_entity(id, alt_x, alt_y) == MoveResult::Moved {
            return MoveResult::Moved;
        }
    }

    MoveResult::Blocked
}

pub fn move_toward(world: &mut WorldState, id: EntityId, x: u8, y: u8, tx: u8, ty: u8) {
    if x == tx && y == ty {
        return;
    }
    let dx = tx as i16 - x as i16;
    let dy = ty as i16 - y as i16;
    let (step_dx, step_dy) = manhattan_step(dx, dy, world.rng_coin_for(id.0));

    let (gx, gy) = clamp_cell(x as i16 + step_dx, y as i16 + step_dy);
    if (gx != x || gy != y) && !is_blocked_for(world, gx, gy, Some(id.0)) {
        if attempt_move_with_resolution(world, id, x, y, step_dx, step_dy) == MoveResult::Moved {
            return;
        }
    } else if attempt_move_with_resolution(world, id, x, y, step_dx, step_dy) == MoveResult::Moved
    {
        return;
    }

    if let Some((nx, ny)) = find_path(world, x, y, tx, ty, Some(id.0)).first().copied() {
        let raw_pdx = nx as i16 - x as i16;
        let raw_pdy = ny as i16 - y as i16;
        let (pdx, pdy) = manhattan_step(raw_pdx, raw_pdy, world.rng_coin_for(id.0 ^ 0x5A5A));
        if attempt_move_with_resolution(world, id, x, y, pdx, pdy) == MoveResult::Moved {
            return;
        }
    }
    move_toward_greedy(world, id, x, y, tx, ty);
}

fn move_toward_greedy(world: &mut WorldState, id: EntityId, x: u8, y: u8, tx: u8, ty: u8) {
    let dx = tx as i16 - x as i16;
    let dy = ty as i16 - y as i16;
    let (step_dx, step_dy) = manhattan_step(dx, dy, world.rng_coin_for(id.0 ^ 0xA5A5));
    let _ = attempt_move_with_resolution(world, id, x, y, step_dx, step_dy);
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
    let (step_dx, step_dy) = manhattan_step(dx, dy, world.rng_coin_for(id.0 ^ 0xBEEF));
    if attempt_move_with_resolution(world, id, x, y, step_dx, step_dy) == MoveResult::Moved {
        return;
    }
    let (alt_dx, alt_dy) = manhattan_step(dx, dy, !world.rng_coin_for(id.0 ^ 0xBEEF));
    if (alt_dx, alt_dy) != (step_dx, step_dy) {
        let _ = attempt_move_with_resolution(world, id, x, y, alt_dx, alt_dy);
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
    let salt = tick.wrapping_add(id.0 as u64);
    let (step_dx, step_dy) = if world.rng_coin_for(salt) {
        if x + 1 < GRID_WIDTH {
            (1, 0)
        } else if x > 0 {
            (-1, 0)
        } else if y + 1 < GRID_HEIGHT {
            (0, 1)
        } else {
            (0, -1)
        }
    } else if y + 1 < GRID_HEIGHT {
        (0, 1)
    } else if y > 0 {
        (0, -1)
    } else if x > 0 {
        (-1, 0)
    } else {
        (1, 0)
    };
    let _ = attempt_move_with_resolution(world, id, x, y, step_dx, step_dy);
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
        let dx = e.x.abs_diff(5);
        let dy = e.y.abs_diff(5);
        assert!(dx == 0 || dy == 0, "path fallback must not diagonal: dx={dx} dy={dy}");
        assert!(e.x == 5 || e.x == 6 || e.x == 7);
    }

    #[test]
    fn manhattan_step_never_diagonal() {
        let mut w = empty_world();
        let id = w.spawn("sheep", 5, 5);
        move_toward(&mut w, id, 5, 5, 8, 8);
        let e = &w.entities[&id];
        let dx = e.x.abs_diff(5);
        let dy = e.y.abs_diff(5);
        assert!(dx == 0 || dy == 0, "diagonal step dx={dx} dy={dy}");
        assert_eq!(dx + dy, 1);
    }

    #[test]
    fn higher_priority_yield_enters_blocked_cell() {
        let mut w = empty_world();
        let mover = w.spawn("sheep", 5, 5);
        let blocker = w.spawn("sheep", 6, 5);
        w.entities.get_mut(&mover).unwrap().ecology_state = EcologyState::Fleeing;
        w.entities.get_mut(&blocker).unwrap().ecology_state = EcologyState::Idle;
        move_toward(&mut w, mover, 5, 5, 6, 5);
        assert_eq!(w.entities[&mover].x, 6);
        assert_eq!(w.entities[&mover].y, 5);
        assert_ne!(w.entities[&blocker].x, 6);
    }

    #[test]
    fn face_to_face_swap_via_yield() {
        let mut w = empty_world();
        let a = w.spawn("sheep", 5, 5);
        let b = w.spawn("sheep", 6, 5);
        w.entities.get_mut(&a).unwrap().ecology_state = EcologyState::Fleeing;
        w.entities.get_mut(&b).unwrap().ecology_state = EcologyState::Idle;
        move_toward(&mut w, a, 5, 5, 6, 5);
        move_toward(&mut w, b, 6, 5, 5, 5);
        assert_eq!(w.entities[&a].x, 6);
        assert_eq!(w.entities[&b].x, 5);
    }
}
