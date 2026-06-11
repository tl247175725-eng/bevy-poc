//! Soil-tag-driven producer spawn and regrowth.

use crate::ecology_log::eco_log;
use crate::terrain::terrain_at;
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;

const SPAWN_INTERVAL: u64 = 25;
const SAMPLES_PER_PASS: usize = 12;

fn cell_has_tag(world: &WorldState, x: u8, y: u8, tag: &str) -> bool {
    world
        .cell_composition
        .slot(x, y)
        .tags
        .iter()
        .any(|t| t == tag)
}

fn spawn_roll(tick: u64, x: u8, y: u8, salt: u32) -> u32 {
    let h = tick
        .wrapping_mul(1_103_515_245)
        .wrapping_add(x as u64 * 12_345)
        .wrapping_add(y as u64 * 67_890)
        .wrapping_add(salt as u64);
    (h % 100) as u32
}

fn cell_open_for_surface(world: &WorldState, x: u8, y: u8) -> bool {
    if !matches!(terrain_at(world, x, y), "land" | "wetland" | "bank" | "riverbank") {
        return false;
    }
    world.cell_composition.slot(x, y).living_count == 0
}

pub fn tick_producer_spawn(world: &mut WorldState) {
    if world.tick_count % SPAWN_INTERVAL != 0 {
        return;
    }
    let tick = world.tick_count;
    for i in 0..SAMPLES_PER_PASS {
        let x = 1 + (spawn_roll(tick, i as u8, 0, 1) as u8 % (GRID_WIDTH - 2));
        let y = 1 + (spawn_roll(tick, 0, i as u8, 2) as u8 % (GRID_HEIGHT - 2));
        if !cell_open_for_surface(world, x, y) {
            continue;
        }
        try_spawn_for_soil(world, x, y, tick, i as u32);
    }
}

fn try_spawn_for_soil(world: &mut WorldState, x: u8, y: u8, tick: u64, salt: u32) {
    let roll = spawn_roll(tick, x, y, salt);
    if cell_has_tag(world, x, y, "soil:rocky") {
        return;
    }
    if cell_has_tag(world, x, y, "soil:deep") && cell_has_tag(world, x, y, "fertility:high") {
        if roll < 20 {
            world.spawn("wildYam", x, y);
            eco_log(world, "深土山药萌芽");
        } else if roll < 30 {
            world.spawn("wildYamRoot", x, y);
        }
        return;
    }
    if cell_has_tag(world, x, y, "soil:rich") && cell_has_tag(world, x, y, "fertility:high") {
        if roll < 70 {
            world.spawn("grass", x, y);
        } else if roll < 80 {
            world.spawn("soilMushroom", x, y);
        }
        return;
    }
    if cell_has_tag(world, x, y, "soil:wet") && cell_has_tag(world, x, y, "fertility:high") {
        if roll < 60 {
            world.spawn("grass", x, y);
        }
        return;
    }
    if cell_has_tag(world, x, y, "soil:loose") && cell_has_tag(world, x, y, "shaded") {
        if roll < 30 {
            world.spawn("grass", x, y);
        } else if roll < 40 {
            world.spawn("burrowTuber", x, y);
        }
        return;
    }
    if cell_has_tag(world, x, y, "soil:dry") && cell_has_tag(world, x, y, "fertility:low") {
        if roll < 30 {
            world.spawn("aridGrass", x, y);
        } else if roll < 40 {
            world.spawn("fern", x, y);
        }
    }
}

struct GrowthRule {
    type_name: &'static str,
    interval: u64,
    max_hp: i32,
}

const GROWTH_RULES: &[GrowthRule] = &[
    GrowthRule {
        type_name: "grass",
        interval: 10,
        max_hp: 4,
    },
    GrowthRule {
        type_name: "aridGrass",
        interval: 20,
        max_hp: 2,
    },
    GrowthRule {
        type_name: "soilMushroom",
        interval: 15,
        max_hp: 3,
    },
    GrowthRule {
        type_name: "fern",
        interval: 30,
        max_hp: 1,
    },
];

pub fn tick_producer_growth(world: &mut WorldState) {
    let tick = world.tick_count;
    for rule in GROWTH_RULES {
        if tick % rule.interval != 0 {
            continue;
        }
        let ids: Vec<_> = world
            .entities
            .values()
            .filter(|e| e.type_name == rule.type_name && !e.is_corpse && e.hp > 0)
            .map(|e| e.id)
            .collect();
        for id in ids {
            if let Some(e) = world.entities.get_mut(&id) {
                if e.hp < rule.max_hp {
                    e.hp += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn dry_soil_spawns_arid_plants() {
        let mut world = empty_world();
        world.ensure_map_ecology();
        let cell = (1..GRID_WIDTH - 1)
            .flat_map(|x| (1..GRID_HEIGHT - 1).map(move |y| (x, y)))
            .find(|&(x, y)| {
                cell_has_tag(&world, x, y, "soil:dry")
                    && cell_has_tag(&world, x, y, "fertility:low")
            })
            .expect("dry soil cell");
        world.cell_composition.slot_mut(cell.0, cell.1).living_count = 0;
        for salt in 0..200u32 {
            try_spawn_for_soil(&mut world, cell.0, cell.1, 100, salt);
            if world.count_type("aridGrass") > 0 || world.count_type("fern") > 0 {
                return;
            }
        }
        panic!("expected arid producer on dry soil");
    }

    #[test]
    fn rich_soil_spawns_grass() {
        let mut world = empty_world();
        world.ensure_map_ecology();
        let cell = world
            .ecology
            .wetland_cells
            .iter()
            .copied()
            .find(|&(x, y)| world.ecology.pool_manhattan_dist(x, y) == 6)
            .expect("inner wetland rich soil");
        world.cell_composition.slot_mut(cell.0, cell.1).living_count = 0;
        for tick in (0..100).step_by(1) {
            try_spawn_for_soil(&mut world, cell.0, cell.1, tick, 0);
            if world.count_type("grass") > 0 {
                return;
            }
        }
        panic!("expected grass on rich soil");
    }
}
