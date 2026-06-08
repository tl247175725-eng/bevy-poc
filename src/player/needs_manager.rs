//! Player tick orchestration — Godot `player_needs_manager.gd` (brain + execution).

use std::collections::HashMap;

use crate::interaction::InteractionState;
use crate::sim_events::SimEventQueue;
use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

use super::action_runner::ActionRunner;
use super::behavior::tick_brain;
use super::state::PlayerMind;

pub fn ensure_player_mind(minds: &mut HashMap<EntityId, PlayerMind>, player_id: EntityId) {
    minds.entry(player_id).or_insert_with(PlayerMind::new_spawn);
}

pub fn tick_player(
    world: &mut WorldState,
    player_id: EntityId,
    delta: f32,
    interaction: &mut InteractionState,
    events: &mut SimEventQueue,
) {
    tick_brain(world, player_id);
    let mut mind = world
        .player_minds
        .remove(&player_id)
        .unwrap_or_else(PlayerMind::new_spawn);
    ActionRunner::tick(world, player_id, &mut mind, delta, interaction, events);
    world.player_minds.insert(player_id, mind);
    world.pending_events.extend(events.drain());
}

/// Headless sim/tests — local interaction + event queue per tick.
pub fn tick_player_world(world: &mut WorldState, player_id: EntityId, delta: f32) {
    let mut interaction = InteractionState::default();
    let mut events = SimEventQueue::default();
    tick_player(world, player_id, delta, &mut interaction, &mut events);
}

pub fn player_mind<'a>(world: &'a WorldState, player_id: EntityId) -> Option<&'a PlayerMind> {
    world.player_minds.get(&player_id)
}

pub fn player_mind_mut<'a>(world: &'a mut WorldState, player_id: EntityId) -> Option<&'a mut PlayerMind> {
    world.player_minds.get_mut(&player_id)
}
