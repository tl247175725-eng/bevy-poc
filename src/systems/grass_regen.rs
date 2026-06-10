use crate::game_constants::{
    BUSH_REGEN_HP_INTERVAL, GRASS_REGEN_HP_INTERVAL, GRASS_WETLAND_HP_CAP, WETLAND_REGEN_RADIUS,
};
use crate::terrain::terrain_at;
use crate::world_rules::{card_has_tag, GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;

pub fn near_wetland(world: &WorldState, x: u8, y: u8, radius: u8) -> bool {
    let r = radius as i16;
    for dy in -r..=r {
        for dx in -r..=r {
            let nx = x as i16 + dx;
            let ny = y as i16 + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let (nx, ny) = (nx as u8, ny as u8);
            if nx >= GRID_WIDTH || ny >= GRID_HEIGHT {
                continue;
            }
            if world.ecology.ready && world.ecology.is_wetland_cell(nx, ny) {
                return true;
            }
            if matches!(
                terrain_at(world, nx, ny),
                "wetland" | "bank" | "riverbank" | "ford" | "pool"
            ) {
                return true;
            }
        }
    }
    false
}

fn grass_regen_cap(world: &WorldState, x: u8, y: u8, def_hp: i32) -> i32 {
    let on_wet = world.ecology.ready && world.ecology.is_wetland_cell(x, y)
        || matches!(
            terrain_at(world, x, y),
            "wetland" | "bank" | "riverbank" | "ford" | "pool"
        );
    if on_wet {
        def_hp.min(GRASS_WETLAND_HP_CAP)
    } else {
        def_hp
    }
}

/// Wetland-adjacent grass regains +1 HP every 15 ticks (no timed cell spawn).
pub fn tick_grass_regen(world: &mut WorldState) {
    if world.tick_count == 0 || world.tick_count % GRASS_REGEN_HP_INTERVAL != 0 {
        return;
    }
    let grass_ids: Vec<_> = world
        .entities
        .values()
        .filter(|e| e.type_name == "grass" && !e.is_corpse && e.hp > 0)
        .map(|e| e.id)
        .collect();
    for id in grass_ids {
        let (x, y, hp, max_hp) = {
            let e = &world.entities[&id];
            let def_hp = world
                .card_defs
                .get(&e.type_name)
                .map(|d| d.hp)
                .unwrap_or(4)
                .max(1);
            (
                e.x,
                e.y,
                e.hp,
                grass_regen_cap(world, e.x, e.y, def_hp),
            )
        };
        if !near_wetland(world, x, y, WETLAND_REGEN_RADIUS) {
            continue;
        }
        if hp < max_hp {
            if let Some(e) = world.entities.get_mut(&id) {
                e.hp += 1;
            }
        }
    }
}

/// Bush cover regains +1 HP every 30 ticks.
pub fn tick_bush_regen(world: &mut WorldState) {
    if world.tick_count == 0 || world.tick_count % BUSH_REGEN_HP_INTERVAL != 0 {
        return;
    }
    let bush_ids: Vec<_> = world
        .entities
        .values()
        .filter(|e| e.type_name == "bush" && !e.is_corpse && e.hp > 0)
        .map(|e| e.id)
        .collect();
    for id in bush_ids {
        let max_hp = world
            .card_defs
            .get("bush")
            .map(|d| d.hp)
            .unwrap_or(8)
            .max(1);
        if let Some(e) = world.entities.get_mut(&id) {
            if e.hp < max_hp {
                e.hp += 1;
            }
        }
    }
}

pub fn cover_at_cell(
    world: &WorldState,
    x: u8,
    y: u8,
    tag: &str,
) -> Option<crate::spatial_index::EntityId> {
    world.entities_at(x, y).into_iter().find(|&id| {
        world.entities.get(&id).is_some_and(|e| {
            if e.is_corpse || e.hp <= 0 {
                return false;
            }
            e.type_name == tag
                || world
                    .card_defs
                    .get(&e.type_name)
                    .is_some_and(|d| card_has_tag(d, tag))
        })
    })
}
