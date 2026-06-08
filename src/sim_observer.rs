//! WorldState SimObserver — scheme B (no Bevy ECS migration).
//!
//! Choke points: spawn / despawn / kill / move → ecology dispatch; UI-visible events → `pending_events`.

use crate::ecology_log::push_sim_event;
use crate::event_registry::EventRegistry;
use crate::sim_events::SimEvent;
use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

pub fn emit(world: &mut WorldState, event: SimEvent) {
    push_sim_event(world, event);
}

pub fn on_spawn(world: &mut WorldState, id: EntityId, type_name: &str, x: u8, y: u8) {
    let is_corpse = world
        .entities
        .get(&id)
        .map(|e| e.is_corpse || type_name.ends_with("Corpse"))
        .unwrap_or(false);
    if is_corpse {
        emit(
            world,
            SimEvent::Spawn {
                entity_id: id,
                type_name: type_name.to_string(),
                x,
                y,
                is_corpse: true,
            },
        );
    }
    EventRegistry::handle_spawn(world, id);
}

pub fn on_despawn(world: &mut WorldState, _type_name: &str, _x: u8, _y: u8) {}

pub fn on_kill(
    world: &mut WorldState,
    predator_type: &str,
    prey_type: &str,
    x: u8,
    y: u8,
) {
    emit(
        world,
        SimEvent::Kill {
            predator: crate::ecology_log::card_display_name(world, predator_type),
            prey: crate::ecology_log::card_display_name(world, prey_type),
            x,
            y,
        },
    );
    EventRegistry::handle_kill(world, predator_type, prey_type, x, y);
}

/// Cut 2: move notification — neighbor hunt/flee without replacing bucket tick.
pub fn on_move(world: &mut WorldState, id: EntityId, from: (u8, u8), to: (u8, u8)) {
    let _ = world.entities.get(&id);
    if from == to {
        return;
    }
    EventRegistry::handle_move(world, id, from, to);
}
