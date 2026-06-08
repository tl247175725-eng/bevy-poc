//! Godot `eco_log` — routes through unified `SimEvent` on WorldState.

use crate::sim_events::SimEvent;
use crate::world_state::WorldState;

pub fn push_sim_event(world: &mut WorldState, event: SimEvent) {
    world.pending_events.push(event);
}

pub fn eco_log(world: &mut WorldState, msg: impl Into<String>) {
    push_sim_event(world, SimEvent::Generic(msg.into()));
}

pub fn card_display_name(world: &WorldState, type_name: &str) -> String {
    world
        .card_defs
        .get(type_name)
        .map(|d| d.display_name.clone())
        .unwrap_or_else(|| type_name.to_string())
}
