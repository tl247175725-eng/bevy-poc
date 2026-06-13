use crate::terrain::terrain_at;
use crate::world_rules::{card_has_tag, GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;

/// 湿地检测——基础设施查询
pub fn near_wetland(world: &WorldState, x: u8, y: u8, radius: u8) -> bool {
    let r = radius as i16;
    for dy in -r..=r {
        for dx in -r..=r {
            let nx = x as i16 + dx;
            let ny = y as i16 + dy;
            if nx < 0 || ny < 0 { continue; }
            let (nx, ny) = (nx as u8, ny as u8);
            if nx >= GRID_WIDTH || ny >= GRID_HEIGHT { continue; }
            if world.ecology.ready && world.ecology.is_wetland_cell(nx, ny) { return true; }
            if matches!(terrain_at(world, nx, ny), "wetland" | "bank" | "riverbank" | "ford" | "pool") {
                return true;
            }
        }
    }
    false
}

/// 草再生——已掏空，等待环境系统重构
pub fn tick_grass_regen(_world: &mut WorldState) {}

/// 灌木再生——已掏空
pub fn tick_bush_regen(_world: &mut WorldState) {}

/// 掩护检测——基础设施查询
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
