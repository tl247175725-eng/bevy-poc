//! 批量更新——已掏空，等待元数值驱动的生命周期系统。

use crate::world_state::WorldState;

/// 均匀更新——已移除（冷却、年龄、腐解等待新系统）
pub fn batch_uniform_entity_updates(_world: &mut WorldState, _delta: f32) {}

/// 尸体腐解——已移除
pub fn flush_corpse_decay(_world: &mut WorldState) {}
