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
        .filter(|e| matches!(e.type_name.as_str(), "shellfish" | "algae"))
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
        if world.has_tag_at(x, y, "primary_producer") || world.count_type("algae") > 0 {
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
    if (world.count_type("algae") as i32) >= ALGAE_CAP {
        return;
    }
    let pools: Vec<(u8, u8)> = world.pool_cells.iter().copied().collect();
    let per_pool_cap = pools.len().max(1);
    for (x, y) in pools {
        if (world.count_type("algae") as i32) >= ALGAE_CAP {
            break;
        }
        if world.count_type("algae") >= per_pool_cap {
            break;
        }
        if world.has_tag_at(x, y, "algae") || !world.entities_at(x, y).is_empty() {
            continue;
        }
        world.spawn("algae", x, y);
    }
}

fn migrate_aquatic(world: &mut WorldState) {
    if (world.count_type("waterBug") as i32) < WATER_BUG_CAP {
        if let Some((x, y)) = empty_pool_for(world, "waterBug") {
            world.spawn("waterBug", x, y);
        }
    }
    if (world.count_type("fish") as i32) < FISH_CAP {
        let cell = empty_pool_without(world, "fish", "waterBug")
            .or_else(|| empty_pool_for(world, "fish"));
        if let Some((x, y)) = cell {
            world.spawn("fish", x, y);
        }
    }
}

fn empty_pool_for(world: &WorldState, type_name: &str) -> Option<(u8, u8)> {
    empty_pool_without(world, type_name, type_name)
}

fn empty_pool_without(
    world: &WorldState,
    type_name: &str,
    avoid_type: &str,
) -> Option<(u8, u8)> {
    world.pool_cells.iter().copied().find(|&(x, y)| {
        let occupants: Vec<&str> = world
            .entities_at(x, y)
            .iter()
            .filter_map(|id| world.entities.get(id).map(|e| e.type_name.as_str()))
            .collect();
        !occupants.iter().any(|t| *t == type_name)
            && !occupants.iter().any(|t| *t == avoid_type)
    })
}
