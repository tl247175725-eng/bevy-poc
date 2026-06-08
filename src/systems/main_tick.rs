use crate::event_registry::EventRegistry;
use crate::spatial_index::EntityId;
use crate::systems::tick_predator::{flush_predator_patrol, mark_baseline_predator_patrol};
use crate::world_rules::{card_has_capability, card_has_tag};
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

    mark_baseline_predator_patrol(world);
    mark_baseline_herbivore_tick(world);

    crate::systems::tick_environment::tick_environment(world, delta);

    crate::systems::tick_reproduction::tick_reproduction(world, delta);

    flush_predator_patrol(world, delta);
    flush_herbivore_tick(world, delta);

    if !world.pending_spawn_ecology.is_empty() {
        EventRegistry::flush_spawn_ecology(world);
    }
}

pub fn mark_baseline_herbivore_tick(world: &mut WorldState) {
    for entity in world.entities.values_mut() {
        if entity.is_corpse || entity.in_den {
            continue;
        }
        let Some(def) = world.card_defs.get(&entity.type_name) else {
            continue;
        };
        if card_has_tag(def, "herbivore")
            || card_has_tag(def, "omnivore.small")
            || card_has_capability(def, "capability.forage")
        {
            entity.needs_grazing_tick = true;
        }
    }
}

pub fn flush_herbivore_tick(world: &mut WorldState, delta: f32) {
    let ids: Vec<EntityId> = world
        .entities
        .iter()
        .filter(|(_, e)| e.needs_grazing_tick)
        .map(|(id, _)| *id)
        .collect();
    for id in ids {
        if let Some(entity) = world.entities.get_mut(&id) {
            entity.needs_grazing_tick = false;
        }
        let Some(def) = world
            .entities
            .get(&id)
            .and_then(|e| world.card_defs.get(&e.type_name).cloned())
        else {
            continue;
        };
        EventRegistry::tick_non_predator_ecology(world, id, &def, delta);
    }
}
