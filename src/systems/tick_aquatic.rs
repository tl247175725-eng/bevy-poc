use crate::card_def::CardDef;
use crate::game_constants::{
    ALGAE_CAP, ALGAE_REGEN_SECONDS, AQUATIC_MIGRATION_INTERVAL, FISH_CAP, WATER_BUG_CAP,
};
use crate::spatial_index::EntityId;
use crate::world_rules::{card_has_tag, is_sessile, mark_ecology_fed};
use crate::world_state::{EcologyState, WorldState};

/// Environment-only aquatic tick — algae regen, migration, sessile filter feeders.
/// Mobile fish/waterBug behavior is handled by `tick_reactive`.
pub fn tick_aquatic(world: &mut WorldState, spatial_delta: f32) {
    tick_algae_regen(world, spatial_delta);
    world.aquatic_timer += spatial_delta;
    if world.aquatic_timer >= AQUATIC_MIGRATION_INTERVAL {
        world.aquatic_timer = 0.0;
        migrate_aquatic(world);
    }
    let mut sessile: Vec<EntityId> = world
        .entities
        .values()
        .filter(|e| world.card_defs.get(&e.type_name)
            .is_some_and(|d| card_has_tag(d, "sessile_aquatic")))
        .map(|e| e.id)
        .collect();
    for id in sessile.drain(..) {
        let type_name = world.entities.get(&id).map(|e| e.type_name.clone());
        let Some(type_name) = type_name else {
            continue;
        };
        if let Some(def) = world.card_defs.get(&type_name).cloned() {
            tick_sessile_aquatic(world, id, &def);
        }
    }
}

fn tick_sessile_aquatic(world: &mut WorldState, id: EntityId, def: &CardDef) {
    let Some(entity) = world.entities.get(&id) else {
        return;
    };
    if !world.pool_cells.contains(&(entity.x, entity.y)) && !card_has_tag(def, "primary_producer") {
        return;
    }
    let (x, y) = (entity.x, entity.y);

    if card_has_tag(def, "primary_producer") {
        return;
    }
    if is_sessile(def) {
        if world.has_tag_at(x, y, "primary_producer") || world.count_by_tag("primary_producer") > 0 {
            if let Some(e) = world.entities.get_mut(&id) {
                mark_ecology_fed(e, def);
                e.ecology_state = EcologyState::SeekingFood;
            }
        }
    }
}

fn tick_algae_regen(world: &mut WorldState, delta: f32) {
    world.grass_regen_timer += delta;
    if world.grass_regen_timer < ALGAE_REGEN_SECONDS {
        return;
    }
    world.grass_regen_timer = 0.0;
    if (world.count_by_tag("primary_producer") as i32) >= ALGAE_CAP {
        return;
    }
    let pools: Vec<(u8, u8)> = world.pool_cells.iter().copied().collect();
    let per_pool_cap = pools.len().max(1);
    for (x, y) in pools {
        if (world.count_by_tag("primary_producer") as i32) >= ALGAE_CAP {
            break;
        }
        if world.count_by_tag("primary_producer") >= per_pool_cap {
            break;
        }
        if world.has_tag_at(x, y, "algae") || !world.entities_at(x, y).is_empty() {
            continue;
        }
        world.spawn("algae", x, y);
    }
}

fn migrate_aquatic(world: &mut WorldState) {
    if (world.count_by_tag("species:waterBug") as i32) < WATER_BUG_CAP {
        if let Some((x, y)) = empty_pool_tag(world, "species:waterBug", None) {
            world.spawn("waterBug", x, y);
        }
    }
    if (world.count_by_tag("species:fish") as i32) < FISH_CAP {
        let cell = empty_pool_tag(world, "species:fish", Some("species:waterBug"))
            .or_else(|| empty_pool_tag(world, "species:fish", None));
        if let Some((x, y)) = cell {
            world.spawn("fish", x, y);
        }
    }
}

fn entity_at_has_tag(world: &WorldState, x: u8, y: u8, tag: &str) -> bool {
    world.entities_at(x, y).iter().any(|id| {
        world.entities.get(id)
            .and_then(|e| world.card_defs.get(&e.type_name))
            .is_some_and(|d| card_has_tag(d, tag))
    })
}

fn empty_pool_tag(world: &WorldState, tag: &str, avoid_tag: Option<&str>) -> Option<(u8, u8)> {
    world.pool_cells.iter().copied().find(|&(x, y)| {
        !entity_at_has_tag(world, x, y, tag)
            && !avoid_tag.is_some_and(|at| entity_at_has_tag(world, x, y, at))
    })
}
