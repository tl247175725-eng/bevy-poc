//! 工具任务——已掏空，等待人类大脑系统设计。

use crate::interaction::InteractionState;
use crate::sim_events::SimEventQueue;
use crate::spatial_index::EntityId;
use crate::world_state::WorldState;
use super::state::PlayerMind;

pub fn plan_craft_knife(_world: &WorldState, _player_id: EntityId, _mind: &mut PlayerMind) -> bool { false }
pub fn advance_knife_task(_world: &mut WorldState, _player_id: EntityId, _mind: &mut PlayerMind, _interaction: &mut InteractionState, _events: &mut SimEventQueue) {}
pub fn craft_spear_relation(_world: &mut WorldState, _twig: EntityId, _shard: EntityId) -> Option<String> { None }
pub fn craft_axe_relation(_world: &mut WorldState, _tri: EntityId, _wood: EntityId) -> Option<String> { None }
pub fn knap_stones_to_shard() {}
pub fn fsm_phase_sequence() -> &'static [TaskPhase] { &[] }

#[derive(Debug, Clone)]
pub struct TaskPhase { pub name: String }
