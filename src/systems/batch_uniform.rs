//! Per-tick uniform entity field updates — single scan instead of scattered passes.

use std::collections::HashSet;

use crate::ecology_log::eco_log;
use crate::game_constants::{
    CORPSE_DECAY_SECONDS, PLAYER_CORPSE_DECAY, TICKS_PER_DAY, WOLF_CORPSE_DECAY,
};
use crate::world_state::WorldState;

/// Cooldowns, age, and corpse decay advance — one `entities` scan per tick.
pub fn batch_uniform_entity_updates(world: &mut WorldState, delta: f32) {
    let has_corpses = world
        .entities
        .values()
        .any(|e| e.is_corpse || e.type_name.ends_with("Corpse"));
    let land_bug_cells: HashSet<(u8, u8)> = if has_corpses && world.count_type("landBug") > 0 {
        world
            .entities
            .values()
            .filter(|e| e.type_name == "landBug")
            .map(|e| (e.x, e.y))
            .collect()
    } else {
        HashSet::new()
    };

    for entity in world.entities.values_mut() {
        entity.consumed = false;

        if entity.hunt_cooldown > 0.0 {
            entity.hunt_cooldown = (entity.hunt_cooldown - delta).max(0.0);
        }
        if entity.harvest_cooldown > 0.0 {
            entity.harvest_cooldown = (entity.harvest_cooldown - delta).max(0.0);
        }

        if has_corpses && (entity.is_corpse || entity.type_name.ends_with("Corpse")) {
            let bonus = if land_bug_cells.contains(&(entity.x, entity.y)) {
                2.0
            } else {
                1.0
            };
            entity.decay_timer += delta * bonus;
        }

        if !entity.is_corpse && entity.max_age > 0.0 {
            entity.age += delta / TICKS_PER_DAY as f32;
        }
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
    if corpses.is_empty() {
        return;
    }

    for (id, _type_name, x, y, decay) in corpses {
        let def = world.card_defs.get(&_type_name);
        let has_tag = |tag: &str| def.is_some_and(|d| crate::world_rules::card_has_tag(d, tag));
        let max_decay = if has_tag("decay:player") {
            PLAYER_CORPSE_DECAY
        } else if has_tag("decay:long") {
            WOLF_CORPSE_DECAY
        } else {
            CORPSE_DECAY_SECONDS
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
