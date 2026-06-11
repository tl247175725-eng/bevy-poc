//! Per-tick uniform entity field updates — single scan instead of scattered passes.

use crate::ecology_log::eco_log;
use crate::game_constants::{
    CORPSE_DECAY_SECONDS, PLAYER_CORPSE_DECAY, WOLF_CORPSE_DECAY,
};
use crate::world_rules::is_being;
use crate::world_state::WorldState;

/// Cooldowns, age, and corpse decay advance — one `entities` scan per tick.
pub fn batch_uniform_entity_updates(world: &mut WorldState, delta: f32) {
    let land_bugs_active = world.count_type("landBug") > 0;
    let corpses: Vec<_> = world
        .entities
        .values()
        .filter(|e| e.is_corpse || e.type_name.ends_with("Corpse"))
        .map(|e| (e.id, e.x, e.y))
        .collect();

    for entity in world.entities.values_mut() {
        entity.consumed = false;

        if entity.hunt_cooldown > 0.0 {
            entity.hunt_cooldown = (entity.hunt_cooldown - delta).max(0.0);
        }
        if entity.harvest_cooldown > 0.0 {
            entity.harvest_cooldown = (entity.harvest_cooldown - delta).max(0.0);
        }

        if !entity.is_corpse {
            if let Some(def) = world.card_defs.get(&entity.type_name) {
                if is_being(def) {
                    entity.age += delta;
                }
            }
        }
    }

    for (id, x, y) in corpses {
        let bonus = corpse_decay_bonus(land_bugs_active, world, x, y);
        if let Some(entity) = world.entities.get_mut(&id) {
            entity.decay_timer += delta * bonus;
        }
    }
}

fn corpse_decay_bonus(land_bugs_active: bool, world: &WorldState, x: u8, y: u8) -> f32 {
    if land_bugs_active && world.has_tag_at(x, y, "landBug") {
        2.0
    } else {
        1.0
    }
}

/// Remove corpses whose decay timer exceeded type-specific threshold.
pub fn flush_corpse_decay(world: &mut WorldState) {
    let corpses: Vec<_> = world
        .entities
        .values()
        .filter(|e| e.is_corpse || e.type_name.ends_with("Corpse"))
        .map(|e| (e.id, e.type_name.clone(), e.x, e.y, e.decay_timer))
        .collect();

    for (id, type_name, x, y, decay) in corpses {
        let max_decay = match type_name.as_str() {
            "wolfCorpse" => WOLF_CORPSE_DECAY,
            "playerCorpse" => PLAYER_CORPSE_DECAY,
            _ => CORPSE_DECAY_SECONDS,
        };
        if decay >= max_decay {
            world.remove_entity(id);
            let had_layers = world.humus_layers.get(&(x, y)).copied().unwrap_or(0);
            try_spawn_humus(world, x, y);
            if world.humus_layers.get(&(x, y)).copied().unwrap_or(0) > had_layers {
                eco_log(world, "尸体腐解 → 腐殖土（humus）回到土壤循环");
            } else {
                eco_log(world, "尸体腐解 → 该格腐殖层已满");
            }
        }
    }
}

fn try_spawn_humus(world: &mut WorldState, x: u8, y: u8) {
    use crate::game_constants::{HUMUS_MAX_LAYERS};
    let layers = world.humus_layers.get(&(x, y)).copied().unwrap_or(0);
    if layers >= HUMUS_MAX_LAYERS {
        return;
    }
    world.humus_layers.insert((x, y), layers + 1);
    world.humus_age.insert((x, y), 0.0);
    world.spawn("humus", x, y);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn batch_uniform_advances_corpse_decay_and_age() {
        let mut world = empty_world();
        let sheep = world.spawn("sheep", 5, 5);
        let corpse = world.spawn("sheepCorpse", 6, 5);
        world.entities.get_mut(&corpse).unwrap().is_corpse = true;

        batch_uniform_entity_updates(&mut world, 1.0);

        assert!(world.entities[&sheep].age > 0.0);
        assert!(world.entities[&corpse].decay_timer > 0.0);
        assert!(!world.entities[&sheep].consumed);
    }

    #[test]
    fn flush_corpse_decay_removes_fully_decayed() {
        let mut world = empty_world();
        let corpse = world.spawn("rabbitCorpse", 5, 5);
        world.entities.get_mut(&corpse).unwrap().is_corpse = true;
        world.entities.get_mut(&corpse).unwrap().decay_timer = CORPSE_DECAY_SECONDS;

        flush_corpse_decay(&mut world);

        assert!(!world.entities.contains_key(&corpse));
    }
}
