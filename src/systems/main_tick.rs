use crate::event_registry::EventRegistry;
use crate::spatial_index::EntityId;
use crate::systems::tick_reactive::{flush_reactive_tick, mark_baseline_reactive_tick, tick_reactive};
use crate::world_state::WorldState;

pub fn main_tick(world: &mut WorldState, delta: f32) {
    world.tick_delta = delta;
    world.tick_count += 1;
    world.elapsed += delta;

    for entity in world.entities.values_mut() {
        entity.consumed = false;
        if entity.hunt_cooldown > 0.0 {
            entity.hunt_cooldown = (entity.hunt_cooldown - delta).max(0.0);
        }
        if entity.harvest_cooldown > 0.0 {
            entity.harvest_cooldown = (entity.harvest_cooldown - delta).max(0.0);
        }
    }

    mark_baseline_reactive_tick(world);

    crate::systems::tick_environment::tick_environment(world, delta);

    crate::systems::tick_reproduction::tick_reproduction(world, delta);

    flush_reactive_tick(world, delta);

    if !world.pending_spawn_ecology.is_empty() {
        EventRegistry::flush_spawn_ecology(world);
    }
}

pub fn mark_baseline_herbivore_tick(world: &mut WorldState) {
    mark_baseline_reactive_tick(world);
}

pub fn flush_herbivore_tick(world: &mut WorldState, delta: f32) {
    flush_reactive_tick(world, delta);
}

pub fn flush_reactive_entity_tick(world: &mut WorldState, id: EntityId, delta: f32) {
    tick_reactive(world, id, delta);
}
