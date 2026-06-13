//! 大脑世界感知——已掏空，等待人类大脑系统设计。

use crate::spatial_index::EntityId;
use crate::world_state::WorldState;
use super::state::PlayerMind;

pub fn check_all(_world: &WorldState, _player_id: EntityId, _mind: &PlayerMind, _conditions: &[&str]) -> bool { false }
pub fn check(_world: &WorldState, _player_id: EntityId, _mind: &PlayerMind, _condition: &str) -> bool { false }
pub fn nearest_entity_of_type(_world: &WorldState, _px: u8, _py: u8, _type_name: &str) -> Option<(EntityId, u8, u8)> { None }
pub fn is_neighbor(ax: u8, ay: u8, bx: u8, by: u8) -> bool { (ax as i16 - bx as i16).abs() + (ay as i16 - by as i16).abs() == 1 }
pub fn sorted_by_distance(_world: &WorldState, _px: u8, _py: u8, _type_name: &str) -> Vec<EntityId> { Vec::new() }
pub fn card_def_has_tag(_def: &crate::card_def::CardDef, _tag: &str) -> bool { false }
