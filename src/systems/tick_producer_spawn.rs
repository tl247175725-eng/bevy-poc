//! 生产者生成——已掏空，等待需求匹配引擎。

use crate::world_state::WorldState;

/// 生产者生成——已移除（土壤驱动的植物生成将在新系统中重新实现）
pub fn tick_producer_spawn(_world: &mut WorldState) {}

/// 生产者生长——已移除
pub fn tick_producer_growth(_world: &mut WorldState) {}
