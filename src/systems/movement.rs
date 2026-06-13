//! 纯基础设施——移动、碰撞解决、寻路。行为决策（wander/flee/move_toward）已移除。

use crate::pathfinding::{find_path, is_blocked_for};
use crate::spatial_index::EntityId;
use crate::world_rules::{card_has_tag, chebyshev_distance, GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::{EcologyState, MoveResult, WorldState};

fn manhattan_step(dx: i16, dy: i16, pick_x: bool) -> (i16, i16) {
    if dx != 0 && dy != 0 {
        if pick_x { (dx.signum(), 0) } else { (0, dy.signum()) }
    } else {
        (dx.signum(), dy.signum())
    }
}

fn clamp_cell(x: i16, y: i16) -> (u8, u8) {
    (x.clamp(0, GRID_WIDTH as i16 - 1) as u8, y.clamp(0, GRID_HEIGHT as i16 - 1) as u8)
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

fn with_collision_resolve<R>(world: &mut WorldState, f: impl FnOnce(&mut WorldState) -> R) -> R {
    world.sim_observer_depth = world.sim_observer_depth.saturating_add(1);
    let result = f(world);
    world.sim_observer_depth = world.sim_observer_depth.saturating_sub(1);
    result
}

fn blocker_is_being(world: &WorldState, blocker_id: EntityId) -> bool {
    world.entities.get(&blocker_id).and_then(|b| world.card_defs.get(&b.type_name))
        .is_some_and(|d| card_has_tag(d, "being"))
}

fn blocker_is_immovable(world: &WorldState, blocker_id: EntityId) -> bool {
    let Some(blocker) = world.entities.get(&blocker_id) else { return true };
    let Some(def) = world.card_defs.get(&blocker.type_name) else { return true };
    card_has_tag(def, "rooted") || def.is_rooted || blocker.in_tree || blocker.in_pool
}

fn can_trigger_yield(world: &WorldState, mover_id: EntityId, blocker_id: EntityId) -> bool {
    let Some(m) = world.entities.get(&mover_id) else { return false };
    let Some(b) = world.entities.get(&blocker_id) else { return false };
    match m.ecology_state {
        EcologyState::Fleeing => matches!(b.ecology_state, EcologyState::Idle | EcologyState::Wandering),
        EcologyState::Hunting => {
            if b.ecology_state != EcologyState::Idle { return false; }
            let Some(mdef) = world.card_defs.get(&m.type_name) else { return false };
            if !card_has_tag(mdef, "predator") && !card_has_tag(mdef, "mesopredator") { return false; }
            world.card_defs.get(&b.type_name).is_some_and(|d| card_has_tag(d, "herbivore") || card_has_tag(d, "smallPrey"))
        }
        _ => false,
    }
}

fn mark_blocker_displaced(world: &mut WorldState, blocker_id: EntityId) {
    if let Some(b) = world.entities.get_mut(&blocker_id) {
        b.ecology_state = EcologyState::Idle;
        b.needs_grazing_tick = false;
    }
}

fn try_shove(world: &mut WorldState, mover_id: EntityId, blocker_id: EntityId, gx: u8, gy: u8, step_dx: i16, step_dy: i16) -> bool {
    if step_dx == 0 && step_dy == 0 || !blocker_is_being(world, blocker_id) || blocker_is_immovable(world, blocker_id) {
        return false;
    }
    let push_x = gx as i16 + step_dx;
    let push_y = gy as i16 + step_dy;
    if push_x < 0 || push_y < 0 || push_x >= GRID_WIDTH as i16 || push_y >= GRID_HEIGHT as i16 { return false; }
    let (ux, uy) = (push_x as u8, push_y as u8);
    if world.cell_composition.slot(ux, uy).living_count > 0 || is_blocked_for(world, ux, uy, Some(blocker_id.0)) {
        return false;
    }
    let ok = with_collision_resolve(world, |w| {
        w.move_entity(blocker_id, ux, uy) == MoveResult::Moved
            && w.move_entity(mover_id, gx, gy) == MoveResult::Moved
    });
    if ok { mark_blocker_displaced(world, blocker_id); }
    ok
}

fn try_yield_and_enter(world: &mut WorldState, mover_id: EntityId, blocker_id: EntityId, gx: u8, gy: u8) -> bool {
    if !blocker_is_being(world, blocker_id) || blocker_is_immovable(world, blocker_id) { return false; }
    for (bx, by) in adjacent_cells(gx, gy) {
        if world.cell_composition.slot(bx, by).living_count > 0 { continue; }
        if is_blocked_for(world, bx, by, Some(blocker_id.0)) { continue; }
        let ok = with_collision_resolve(world, |w| {
            w.move_entity(blocker_id, bx, by) == MoveResult::Moved
                && w.move_entity(mover_id, gx, gy) == MoveResult::Moved
        });
        if ok { mark_blocker_displaced(world, blocker_id); return true; }
    }
    false
}

/// 带碰撞解决的移动尝试（三层：闪避→让路→挤开）
pub fn attempt_move_with_resolution(world: &mut WorldState, id: EntityId, from_x: u8, from_y: u8, step_dx: i16, step_dy: i16) -> MoveResult {
    if step_dx == 0 && step_dy == 0 { return MoveResult::NoOp; }
    let (gx, gy) = clamp_cell(from_x as i16 + step_dx, from_y as i16 + step_dy);
    if gx == from_x && gy == from_y { return MoveResult::NoOp; }

    if !is_blocked_for(world, gx, gy, Some(id.0)) {
        if world.move_entity(id, gx, gy) == MoveResult::Moved { return MoveResult::Moved; }
    }

    let blocker_id = world.entities_at(gx, gy).into_iter().find(|bid| *bid != id);
    if let Some(bid) = blocker_id {
        if can_trigger_yield(world, id, bid) && try_yield_and_enter(world, id, bid, gx, gy) {
            return MoveResult::Moved;
        }
        if try_shove(world, id, bid, gx, gy, step_dx, step_dy) {
            return MoveResult::Moved;
        }
    }

    for (ax, ay) in [(step_dy, step_dx), (-step_dy, -step_dx)] {
        if ax == 0 && ay == 0 { continue; }
        let (alt_x, alt_y) = clamp_cell(from_x as i16 + ax, from_y as i16 + ay);
        if alt_x == from_x && alt_y == from_y { continue; }
        if is_blocked_for(world, alt_x, alt_y, Some(id.0)) { continue; }
        if world.move_entity(id, alt_x, alt_y) == MoveResult::Moved { return MoveResult::Moved; }
    }

    MoveResult::Blocked
}

/// 最近实体查询
pub fn nearest_of(world: &WorldState, x: u8, y: u8, ids: &[EntityId]) -> Option<(EntityId, u8)> {
    let mut best: Option<(EntityId, u8)> = None;
    for &id in ids {
        if let Some((ex, ey)) = world.spatial_index.position(id) {
            let d = chebyshev_distance(x, y, ex, ey);
            if best.map(|(_, bd)| d < bd).unwrap_or(true) { best = Some((id, d)); }
        }
    }
    best
}

/// 寻找最近的安全陆地
pub fn find_safe_land_near(world: &WorldState, x: u8, y: u8) -> Option<(u8, u8)> {
    for r in 1..7u8 {
        for dx in -(r as i16)..=(r as i16) {
            for dy in -(r as i16)..=(r as i16) {
                if dx.unsigned_abs() as u8 != r && dy.unsigned_abs() as u8 != r { continue; }
                let nx = x as i16 + dx;
                let ny = y as i16 + dy;
                if nx < 0 || ny < 0 || nx >= GRID_WIDTH as i16 || ny >= GRID_HEIGHT as i16 { continue; }
                let (ux, uy) = (nx as u8, ny as u8);
                let terrain = crate::terrain::terrain_at(world, ux, uy);
                if matches!(terrain, "river" | "ford" | "barren" | "pool") { continue; }
                if !world.entities_at(ux, uy).is_empty() { continue; }
                return Some((ux, uy));
            }
        }
    }
    None
}

// ═══════════════════════════════════════════════
// 行为决策函数——已掏空为空壳
// 保留签名供外部编译，等待元动作+需求匹配引擎后重新实现。
// ═══════════════════════════════════════════════

/// 向目标移动（已掏空）
pub fn move_toward(_world: &mut WorldState, _id: EntityId, _x: u8, _y: u8, _tx: u8, _ty: u8) {}

/// 远离威胁（已掏空）
pub fn flee_from(_world: &mut WorldState, _id: EntityId, _x: u8, _y: u8, _tx: u8, _ty: u8) {}

/// 寻路逃跑（已掏空）
pub fn flee_pathfind(_world: &mut WorldState, _id: EntityId, _x: u8, _y: u8, _threat_x: u8, _threat_y: u8) {}

/// 漫游（已掏空）
pub fn wander(_world: &mut WorldState, _id: EntityId, _x: u8, _y: u8, _tick: u64) {}

/// 捕食者邻格检测（已掏空）
pub fn hunting_predator_adjacent(_world: &WorldState, _x: u8, _y: u8, _self_id: EntityId) -> Option<(u8, u8)> { None }

/// 实体优先级（已掏空）
pub fn get_entity_priority(_world: &WorldState, _id: EntityId) -> u8 { 0 }
