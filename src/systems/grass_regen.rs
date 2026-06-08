use crate::game_constants::LIVING_GRASS_CAP;
use crate::world_rules::count_living_grasses;
use crate::world_state::regen_due;
use crate::world_state::WorldState;

pub fn tick_grass_regen(world: &mut WorldState) {
    if !regen_due(world) {
        return;
    }
    if count_living_grasses(world) >= LIVING_GRASS_CAP as usize {
        return;
    }
    let river_cells: Vec<(u8, u8)> = world.river_cells.iter().copied().collect();
    for (x, y) in river_cells {
        if count_living_grasses(world) >= LIVING_GRASS_CAP as usize {
            break;
        }
        if !world.spatial_index.has_grass_at(x, y) {
            world.spawn("grass", x, y);
        }
    }
}
