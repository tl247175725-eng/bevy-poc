//! 快速 tick——时间跳过用的简化版 main_tick
//! 跳过 Phase 1（感知），其他阶段照跑

use crate::world_state::WorldState;
use crate::meta_values::{TICKS_PER_DAY, TICK_SECONDS};

/// 快速 tick：跳过感知，只做需求衰减+决策+执行+应用
pub fn fast_tick(world: &mut WorldState) {
    world.tick_delta = TICK_SECONDS;
    world.tick_count += 1;
    world.elapsed += TICK_SECONDS;

    // 跳过 Phase 1 感知（最贵）

    // Phase 2: 需求衰减
    for entity in world.entities.values_mut() {
        for need in &mut entity.needs {
            crate::need_match::activation::tick_need(need, TICK_SECONDS);
        }
    }

    // Phase 3: 安全阻断
    for entity in world.entities.values_mut() {
        crate::need_match::activation::apply_safety_block(&mut entity.needs);
    }

    // Phase 4-6: 决策+执行+应用（复用 main_tick 的逻辑）
    // 简化：不重建环境，用空环境——动物在快进期间"凭记忆行动"
    let entity_ids: Vec<crate::spatial_index::EntityId> = world.entities.keys().copied().collect();
    for &eid in &entity_ids {
        let Some(entity) = world.entities.get_mut(&eid) else { continue; };
        if entity.execution.intention.is_none()
            || crate::need_match::execution::is_plan_failed(&entity.execution)
        {
            if let Some(action) = crate::need_match::engine::tick_need_engine(
                &mut entity.needs,
                &mut entity.execution,
                &entity.knowledge,
                &[],  // 空环境——快进时不做感知搜索
                TICK_SECONDS,
                (entity.x, entity.y),
            ) {
                world.pending_actions.push((entity.id, action));
            }
        }
    }

    // 应用
    let pending = std::mem::take(&mut world.pending_actions);
    for (entity_id, action) in pending {
        crate::systems::main_tick::apply_meta_action_public(world, entity_id, action);
    }
}

/// 执行时间跳过：快进 N 天
pub fn execute_time_skip(world: &mut WorldState, days: u32) -> usize {
    let total_ticks = days as u64 * TICKS_PER_DAY;
    let mut event_count = 0usize;

    for _ in 0..total_ticks {
        fast_tick(world);
        event_count += world.drain_pending_events().len();
    }

    event_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn execute_time_skip_increases_tick_count() {
        let mut world = empty_world();
        let initial_tick = world.tick_count;
        execute_time_skip(&mut world, 1);
        assert_eq!(world.tick_count, initial_tick + TICKS_PER_DAY);
    }
}
