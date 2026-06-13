//! 大脑标签状态——已掏空，等待人类大脑系统设计。

use crate::spatial_index::EntityId;
use crate::world_state::WorldState;
use super::state::PlayerMind;

pub fn sync_player_tags(_world: &WorldState, _player_id: EntityId, _mind: &mut PlayerMind) {}
pub fn has_tag(_mind: &PlayerMind, _tag: &str) -> bool { false }
pub fn fire_bond_satisfied(_world: &WorldState) -> bool { false }
pub fn owns_tool_type(_mind: &PlayerMind, _type_name: &str) -> bool { false }
pub fn card_has_material_tag(_def: &crate::card_def::CardDef, _tag: &str) -> bool { false }
