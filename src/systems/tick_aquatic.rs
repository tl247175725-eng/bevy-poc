use crate::card_def::CardDef;
use crate::game_constants::{ALGAE_REGEN_SECONDS, AQUATIC_MIGRATION_INTERVAL, FISH_CAP, WATER_BUG_CAP};
use crate::spatial_index::EntityId;
use crate::systems::movement::{move_toward, wander};
use crate::world_rules::{is_sessile, mark_ecology_fed};
use crate::world_state::{EcologyState, WorldState};

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
            tick_aquatic_card(world, id, &def, spatial_delta);
        }
    }
    world.tick_scratch.clear();
    world
        .tick_scratch
        .extend(
            world
                .entities
                .values()
                .filter(|e| matches!(e.type_name.as_str(), "fish" | "waterBug") && !e.is_corpse)
                .map(|e| e.id),
        );
    world.tick_scratch.sort_unstable_by_key(|id| id.0);
    let mut mobile: Vec<EntityId> = world.tick_scratch.drain(..).collect();
    mobile.sort_by_key(|id| {
        world
            .entities
            .get(&id)
            .map(|e| aquatic_tick_priority_for(e.type_name.as_str()))
            .unwrap_or(1)
    });
    for id in mobile {
        let type_name = world.entities.get(&id).map(|e| e.type_name.clone());
        let Some(type_name) = type_name else {
            continue;
        };
        if let Some(def) = world.card_defs.get(&type_name).cloned() {
            tick_aquatic_card(world, id, &def, spatial_delta);
        }
    }
}

pub fn tick_aquatic_card(world: &mut WorldState, id: EntityId, def: &CardDef, _delta: f32) {
    let Some(entity) = world.entities.get(&id) else {
        return;
    };
    if !world.pool_cells.contains(&(entity.x, entity.y)) && def.type_name != "algae" {
        return;
    }
    let type_name = def.type_name.clone();
    let (x, y) = (entity.x, entity.y);

    if type_name == "algae" {
        return;
    }
    if is_sessile(def) {
        if world.has_tag_at(x, y, "primary_producer") || world.count_type("algae") > 0 {
            if let Some(e) = world.entities.get_mut(&id) {
                mark_ecology_fed(e, def);
                e.ecology_state = EcologyState::SeekingFood;
            }
        }
        return;
    }
    if type_name == "waterBug" {
        tick_water_bug(world, id, x, y, def);
    } else if type_name == "fish" {
        tick_fish(world, id, x, y, def);
    }
}

fn aquatic_tick_priority_for(type_name: &str) -> u8 {
    match type_name {
        "fish" => 0,
        "waterBug" => 2,
        _ => 1,
    }
}

fn tick_algae_regen(world: &mut WorldState, delta: f32) {
    world.grass_regen_timer += delta;
    if world.grass_regen_timer < ALGAE_REGEN_SECONDS {
        return;
    }
    world.grass_regen_timer = 0.0;
    let pools: Vec<(u8, u8)> = world.pool_cells.iter().copied().collect();
    for (x, y) in pools {
        if !world.has_tag_at(x, y, "primary_producer") && !world.has_tag_at(x, y, "aquatic") {
            world.spawn("algae", x, y);
        }
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

fn tick_water_bug(world: &mut WorldState, id: EntityId, x: u8, y: u8, def: &CardDef) {
    let algae = world
        .spatial_index
        .query_near(x, y, "primary_producer", 3)
        .into_iter()
        .find(|aid| world.entities.get(aid).map(|e| e.type_name == "algae").unwrap_or(false));
    if let Some(algae_id) = algae {
        let (ax, ay) = world.spatial_index.position(algae_id).unwrap_or((x, y));
        if x == ax && y == ay {
            world.remove_entity(algae_id);
            if let Some(bug) = world.entities.get_mut(&id) {
                mark_ecology_fed(bug, def);
            }
        } else {
            move_toward(world, id, x, y, ax, ay);
        }
        return;
    }
    wander(world, id, x, y, world.tick_count);
    world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Wandering;
}

fn tick_fish(world: &mut WorldState, id: EntityId, x: u8, y: u8, def: &CardDef) {
    let bugs = world
        .spatial_index
        .query_near(x, y, "smallPrey", 4)
        .into_iter()
        .filter(|bid| {
            world
                .entities
                .get(bid)
                .map(|e| e.type_name == "waterBug")
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    if let Some(bug_id) = bugs.first() {
        let (bx, by) = world.spatial_index.position(*bug_id).unwrap_or((x, y));
        if x == bx && y == by {
            world.remove_entity(*bug_id);
            if let Some(fish) = world.entities.get_mut(&id) {
                mark_ecology_fed(fish, def);
            }
        } else {
            move_toward(world, id, x, y, bx, by);
        }
        return;
    }
    wander(world, id, x, y, world.tick_count);
}
