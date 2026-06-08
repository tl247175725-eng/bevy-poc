//! Player brain tick — Godot `player_behavior.gd`.

use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

use super::affordance::compute_affordances;
use super::intention::select_intention;
use super::needs::evaluate_needs;
use super::state::PlayerMind;

pub fn tick_brain(world: &mut WorldState, player_id: EntityId) {
    super::needs_manager::ensure_player_mind(&mut world.player_minds, player_id);
    let mut mind = world
        .player_minds
        .remove(&player_id)
        .unwrap_or_else(PlayerMind::new_spawn);
    compute_affordances(world, player_id, &mut mind);
    evaluate_needs(world, player_id, &mut mind);
    select_intention(world, player_id, &mut mind);
    world.player_minds.insert(player_id, mind);
}

pub fn detect_threat_level(mind: &PlayerMind) -> u8 {
    if super::brain_tags::has_tag(mind, "predator_nearby_unsafe") {
        3
    } else if !mind.runtime_tags.get("fire_bond_satisfied").copied().unwrap_or(false) {
        2
    } else if super::brain_tags::has_tag(mind, "hungry") {
        2
    } else {
        0
    }
}

pub fn should_abort_current(mind: &PlayerMind, threat_level: u8) -> bool {
    threat_level >= 3
}
