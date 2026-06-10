use crate::game_constants::LIVING_GRASS_CAP;
use crate::terrain::terrain_at;
use crate::world_rules::{count_living_grasses, GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::{regen_due, WorldState};

pub fn tick_grass_regen(world: &mut WorldState) {
    if !regen_due(world) {
        return;
    }
    if count_living_grasses(world) >= LIVING_GRASS_CAP as usize {
        return;
    }
    'scan: for y in 1..GRID_HEIGHT.saturating_sub(1) {
        for x in 1..GRID_WIDTH.saturating_sub(1) {
            if count_living_grasses(world) >= LIVING_GRASS_CAP as usize {
                break 'scan;
            }
            if terrain_at(world, x, y) != "land" {
                continue;
            }
            if !world.entities_at(x, y).is_empty() {
                continue;
            }
            world.spawn("grass", x, y);
        }
    }
}
