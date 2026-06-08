use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

/// Player-style harvest interactions (POC: API only, no UI).
pub fn harvest_at(world: &mut WorldState, x: u8, y: u8, tool: &str) -> Option<String> {
    if tool.is_empty() {
        if let Some(fruit) = harvest_pool_host(world, x, y) {
            return Some(fruit);
        }
        if let Some(nut) = harvest_tree(world, x, y) {
            return Some(nut);
        }
        if let Some(yam) = dig_yam(world, x, y) {
            return Some(yam);
        }
        if let Some(meat) = gather_shellfish(world, x, y) {
            return Some(meat);
        }
    }
    None
}

fn harvest_pool_host(world: &mut WorldState, x: u8, y: u8) -> Option<String> {
    for id in world.entities_at(x, y) {
        let type_name = world.entities[&id].type_name.clone();
        let product = match type_name.as_str() {
            "waterCaltrop" => "caltropFruit",
            "lotus" => "lotusSeed",
            _ => continue,
        };
        world.spawn(product, x, y);
        if let Some(e) = world.entities.get_mut(&id) {
            e.harvest_cooldown = crate::game_constants::POOL_HARVEST_REGEN_SECONDS;
        }
        return Some(product.to_string());
    }
    None
}

fn harvest_tree(world: &mut WorldState, x: u8, y: u8) -> Option<String> {
    for id in world.entities_at(x, y) {
        let type_name = world.entities[&id].type_name.clone();
        let product = match type_name.as_str() {
            "oak" => "acorn",
            "pine" => "pineCone",
            _ => continue,
        };
        world.spawn(product, x, y);
        return Some(product.to_string());
    }
    None
}

fn dig_yam(world: &mut WorldState, x: u8, y: u8) -> Option<String> {
    let underground: Vec<EntityId> = world
        .entities
        .values()
        .filter(|e| e.type_name == "wildYam" && e.x == x && e.y == y)
        .map(|e| e.id)
        .collect();
    if underground.is_empty() {
        return None;
    }
    world.remove_entity(underground[0]);
    world.spawn("wildYamRoot", x, y);
    Some("wildYamRoot".to_string())
}

fn gather_shellfish(world: &mut WorldState, x: u8, y: u8) -> Option<String> {
    let shell: Vec<EntityId> = world
        .entities
        .values()
        .filter(|e| e.type_name == "shellfish" && e.x == x && e.y == y)
        .map(|e| e.id)
        .collect();
    if shell.is_empty() {
        return None;
    }
    world.remove_entity(shell[0]);
    world.spawn("fishMeat", x, y);
    Some("fishMeat".to_string())
}
