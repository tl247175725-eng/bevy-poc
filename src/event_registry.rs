//! Event-driven ecology dispatch — replaces per-tick `BehaviorRegistry` bucket.

use bevy::prelude::Resource;

use crate::card_def::CardDef;
use crate::axioms::DriveBehavior;
use crate::game_constants::WILDPREY_FEAR_RANGE;
use crate::sim_events::SimEvent;
use crate::spatial_index::EntityId;
use crate::systems::movement::flee_from;
use crate::systems::tick_reactive::{mark_predators_near_prey_needs_patrol, tick_reactive};
use crate::world_rules::{chebyshev_distance, card_has_tag};
use crate::world_state::{EcologyState, WorldState};

#[derive(Resource, Default)]
pub struct EventRegistry {
    pub dispatch_count: u64,
}

impl EventRegistry {
    /// Godot `EcosystemTickRegistry.tick_card` — retained for direct / test invocation.
    pub fn tick_entity_ecology(world: &mut WorldState, id: EntityId, delta: f32) {
        tick_reactive(world, id, delta);
    }

    pub fn tick_non_predator_ecology(
        world: &mut WorldState,
        id: EntityId,
        _def: &CardDef,
        delta: f32,
    ) {
        tick_reactive(world, id, delta);
    }

    /// `SimEvent::Spawn` — queue ecology; predator arrival notifies nearby prey.
    pub fn handle_spawn(world: &mut WorldState, id: EntityId) {
        let Some(entity) = world.entities.get(&id).cloned() else {
            return;
        };
        if entity.is_corpse {
            return;
        }
        let Some(def) = world.card_defs.get(&entity.type_name).cloned() else {
            return;
        };
        if card_has_tag(&def, "player") {
            return;
        }

        if card_has_tag(&def, "predator") || card_has_tag(&def, "mesopredator") {
            notify_prey_near_predator(world, id, (entity.x, entity.y), (entity.x, entity.y));
            return;
        }

        if entity.is_autonomous(&world.card_defs) {
            world.pending_spawn_ecology.push(id);
        }
    }

    /// End-of-tick flush for cards spawned earlier in the same tick.
    pub fn flush_spawn_ecology(world: &mut WorldState) {
        let mut pending: Vec<EntityId> = world.pending_spawn_ecology.drain(..).collect();
        pending.sort_by_key(|id| spawn_ecology_priority(world, *id));
        pending.dedup();
        let delta = world.tick_delta;
        for id in pending {
            let type_name = world.entities.get(&id).map(|e| e.type_name.clone());
            let Some(type_name) = type_name else {
                continue;
            };
            let Some(def) = world.card_defs.get(&type_name).cloned() else {
                continue;
            };
            if def.tags.iter().any(|t| t == "aquatic")
                && world
                    .entities
                    .get(&id)
                    .is_some_and(|e| e.profile.native_medium == "water")
            {
                continue;
            }
            Self::tick_non_predator_ecology(world, id, &def, delta);
        }
    }

    /// `SimEvent::Move` — neighbor hunt/flee + mover ecology (quiet cards stay quiet).
    pub fn handle_move(world: &mut WorldState, id: EntityId, from: (u8, u8), to: (u8, u8)) {
        if from == to || world.sim_observer_depth > 0 {
            return;
        }
        world.sim_observer_depth += 1;

        let delta = world.tick_delta;
        let mover = world.entities.get(&id).cloned();
        let mover_def = mover
            .as_ref()
            .and_then(|m| world.card_defs.get(&m.type_name).cloned());

        if let Some(def) = mover_def.as_ref() {
            if is_hunt_prey(def) {
                mark_predators_near_prey_needs_patrol(world, id, to.0, to.1);
            }
            if card_has_tag(def, "predator") || card_has_tag(def, "mesopredator") {
                notify_prey_near_predator(world, id, from, to);
            } else if !card_has_tag(def, "player")
                && !(def.tags.iter().any(|t| t == "aquatic")
                    && mover
                        .as_ref()
                        .is_some_and(|m| m.profile.native_medium == "water"))
            {
                Self::tick_non_predator_ecology(world, id, def, delta);
                if let Some(entity) = world.entities.get_mut(&id) {
                    entity.needs_grazing_tick = false;
                }
            }
        }

        world.sim_observer_depth = world.sim_observer_depth.saturating_sub(1);
    }

    pub fn handle_kill(
        world: &mut WorldState,
        _predator_type: &str,
        _prey_type: &str,
        _x: u8,
        _y: u8,
    ) {
        let _ = world;
        // Corpse spawn + land-bug attraction handled at kill site / environment tick.
    }

    pub fn dispatch(world: &mut WorldState, event: &SimEvent) {
        match event {
            SimEvent::Spawn { entity_id, .. } => Self::handle_spawn(world, *entity_id),
            SimEvent::Move {
                entity_id,
                from,
                to,
                ..
            } => Self::handle_move(world, *entity_id, *from, *to),
            SimEvent::Kill { .. } => {}
            _ => {}
        }
    }
}

fn spawn_ecology_priority(world: &WorldState, id: EntityId) -> u8 {
    let Some(entity) = world.entities.get(&id) else {
        return 1;
    };
    let Some(def) = world.card_defs.get(&entity.type_name) else {
        return 1;
    };
    if card_has_tag(def, "predator") || card_has_tag(def, "mesopredator") {
        return 0;
    }
    if card_has_tag(def, "aquatic") && entity.profile.native_medium == "water" {
        return 2;
    }
    1
}

fn is_hunt_prey(def: &CardDef) -> bool {
    card_has_tag(def, "herbivore") || card_has_tag(def, "smallPrey")
}

fn notify_prey_near_predator(
    world: &mut WorldState,
    predator_id: EntityId,
    from: (u8, u8),
    to: (u8, u8),
) {
    let prey_ids: Vec<EntityId> = world
        .query_near_filtered(
            to.0,
            to.1,
            "herbivore",
            WILDPREY_FEAR_RANGE,
            predator_id,
        )
        .into_iter()
        .filter(|&id| id != predator_id)
        .collect();

    for prey_id in prey_ids {
        let Some(prey) = world.entities.get(&prey_id).cloned() else {
            continue;
        };
        if prey.is_corpse || prey.in_den {
            continue;
        }
        let fear = prey
            .profile
            .drives
            .iter()
            .filter(|d| d.behavior == DriveBehavior::Flee)
            .map(|d| d.range)
            .max()
            .unwrap_or(WILDPREY_FEAR_RANGE);
        let before = chebyshev_distance(prey.x, prey.y, from.0, from.1);
        let after = chebyshev_distance(prey.x, prey.y, to.0, to.1);
        if before <= fear || after > fear {
            continue;
        }
        flee_from(world, prey_id, prey.x, prey.y, to.0, to.1);
        if let Some(e) = world.entities.get_mut(&prey_id) {
            e.ecology_state = EcologyState::Fleeing;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_rules::{ecosystem_behavior_key, BEHAVIOR_PREDATOR_DEN};
    use crate::world_state::empty_world;

    #[test]
    fn wolf_gets_predator_den_key() {
        let w = empty_world();
        let def = w.card_defs.get("wolf").unwrap();
        assert_eq!(ecosystem_behavior_key(def, "wolf"), BEHAVIOR_PREDATOR_DEN);
    }

    #[test]
    fn registry_ticks_sheep_without_panic() {
        let mut w = empty_world();
        w.spawn("grass", 5, 5);
        let sheep = w.spawn("sheep", 4, 5);
        EventRegistry::tick_entity_ecology(&mut w, sheep, 1.0);
    }
}
