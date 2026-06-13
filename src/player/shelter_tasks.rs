//! 建造任务——已掏空，等待人类大脑系统设计。

use crate::spatial_index::EntityId;
use crate::world_state::WorldState;
use super::state::PlayerMind;

pub fn build_hut_affordable(_world: &WorldState, _player_id: EntityId, _mind: &PlayerMind) -> bool { false }
pub fn has_hut_materials(_world: &WorldState, _player_id: EntityId) -> bool { false }
pub fn plan_build_hut(_world: &mut WorldState, _player_id: EntityId, _mind: &mut PlayerMind) -> bool { false }
pub fn execute_build_hut(_world: &mut WorldState, _player_id: EntityId, _mind: &mut PlayerMind) -> bool { false }
pub fn craft_hut_relation(_world: &mut WorldState, _wood_id: EntityId, _grass_id: EntityId) -> Option<String> { None }
pub fn materials_near_player(_world: &WorldState, _player_id: EntityId) -> bool { false }
pub fn neighbor_materials(_world: &WorldState, _player_id: EntityId) -> bool { false }
