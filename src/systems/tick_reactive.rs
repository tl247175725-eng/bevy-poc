//! 纯基础设施——行为决策已掏空，等待元动作+需求匹配引擎。
//! 保留：唤醒、tick 分发、藏/出藏。移除：所有驱动生成、效用评分、行为执行。

use crate::spatial_index::EntityId;
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;

pub const SLEEP_DURATION_TICKS: u64 = 5;
pub const WAKE_ACTIVITY_RANGE: u8 = 6;

/// 唤醒范围内的所有自主实体
pub fn wake_autonomous_near(world: &mut WorldState, x: u8, y: u8, range: u8) {
    let tick = world.tick_count;
    let min_x = x.saturating_sub(range);
    let max_x = (x as u16 + range as u16).min(GRID_WIDTH as u16 - 1) as u8;
    let min_y = y.saturating_sub(range);
    let max_y = (y as u16 + range as u16).min(GRID_HEIGHT as u16 - 1) as u8;
    for gy in min_y..=max_y {
        for gx in min_x..=max_x {
            for id in world.entities_at(gx, gy) {
                let Some(entity) = world.entities.get_mut(&id) else { continue };
                if entity.is_sleeping(tick) && entity.is_autonomous(&world.card_defs) {
                    entity.wake();
                }
            }
        }
    }
}

/// 标记所有自主实体需要 tick
pub fn mark_baseline_reactive_tick(world: &mut WorldState) {
    let tick = world.tick_count;
    for entity in world.entities.values_mut() {
        if entity.is_corpse || entity.in_den || entity.in_burrow { continue; }
        if entity.is_sleeping(tick) { continue; }
        if entity.is_autonomous(&world.card_defs) {
            entity.needs_grazing_tick = true;
        }
    }
}

/// 分发 tick——当前所有行为逻辑已掏空，仅清理标记
pub fn flush_reactive_tick(world: &mut WorldState, _delta: f32) {
    for entity in world.entities.values_mut() {
        entity.needs_grazing_tick = false;
        entity.needs_patrol = false;
    }
}

/// 猎物移动通知附近的捕食者
pub fn mark_predators_near_prey_needs_patrol(
    world: &mut WorldState,
    prey_id: EntityId,
    x: u8,
    y: u8,
) {
    let mut predators: Vec<EntityId> = world
        .query_near_filtered(x, y, "predator", 6, prey_id)
        .into_iter()
        .chain(world.query_near_filtered(x, y, "mesopredator", 6, prey_id))
        .collect();
    predators.sort_unstable_by_key(|id| id.0);
    predators.dedup();
    for pid in predators {
        if let Some(entity) = world.entities.get_mut(&pid) {
            if !entity.is_corpse && !entity.in_den {
                entity.wake();
            }
        }
    }
}

/// 行为主入口——已掏空，等待元动作+需求匹配引擎
pub fn tick_reactive(_world: &mut WorldState, _id: EntityId, _delta: f32) {
    // 行为决策已移除。实体自主行为将在元动作 + 需求匹配引擎完成后重新实现。
}

/// 离开掩护，重新占有格子
pub fn exit_cover(world: &mut WorldState, id: EntityId) {
    let Some(entity) = world.entities.get(&id).cloned() else { return };
    if !entity.in_cover { return; }
    let (x, y) = (entity.x, entity.y);
    if let Some(e) = world.entities.get_mut(&id) {
        e.in_cover = false;
        e.hidden_in_grass = false;
        e.host_cover_id = None;
        e.wake();
    }
    if let Some(e) = world.entities.get(&id) {
        world.cell_composition.occupy_entity(x, y, e);
    }
}

/// 掩护被破坏，强制弹出
pub fn eject_from_cover(world: &mut WorldState, id: EntityId) {
    let Some(entity) = world.entities.get(&id).cloned() else { return };
    if !entity.in_cover { return; }
    let (x, y) = (entity.x, entity.y);
    if let Some(e) = world.entities.get_mut(&id) {
        e.in_cover = false;
        e.hidden_in_grass = false;
        e.host_cover_id = None;
        e.wake();
    }
    let profile = world.entities.get(&id).map(|e| e.profile.clone());
    let Some(profile) = profile else { return };
    if world.cell_composition.can_occupy(x, y, &profile) {
        if let Some(e) = world.entities.get(&id) {
            world.cell_composition.occupy_entity(x, y, e);
        }
        return;
    }
    if let Some((nx, ny)) = crate::systems::movement::find_safe_land_near(world, x, y) {
        let _ = world.move_entity(id, nx, ny);
    } else if let Some(e) = world.entities.get(&id) {
        world.cell_composition.occupy_entity(x, y, e);
    }
}
