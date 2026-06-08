use crate::game_constants::{CONE_PRODUCE_INTERVAL, NUT_PRODUCE_INTERVAL, POOL_HARVEST_REGEN_SECONDS};
use crate::world_state::WorldState;

pub fn tick_contained_producers(world: &mut WorldState, delta: f32) {
    tick_tree_producers(world, delta);
    tick_pool_hosts(world, delta);
}

fn tick_tree_producers(world: &mut WorldState, delta: f32) {
    let trees: Vec<_> = world
        .entities
        .values()
        .filter(|e| matches!(e.type_name.as_str(), "oak" | "pine" | "tree"))
        .map(|e| (e.id, e.type_name.clone(), e.x, e.y, e.produce_timer))
        .collect();
    for (id, type_name, x, y, mut timer) in trees {
        let interval = if type_name == "oak" {
            NUT_PRODUCE_INTERVAL
        } else {
            CONE_PRODUCE_INTERVAL
        };
        timer += delta;
        if timer >= interval {
            timer = 0.0;
            let product = if type_name == "oak" {
                "acorn"
            } else {
                "pineCone"
            };
            if !world.has_tag_at(x, y, product) {
                let drop = world.spawn(product, x, y);
                if let Some(e) = world.entities.get_mut(&drop) {
                    e.in_tree = true;
                    e.host_tree_id = Some(id);
                }
            }
        }
        if let Some(e) = world.entities.get_mut(&id) {
            e.produce_timer = timer;
        }
    }
}

fn tick_pool_hosts(world: &mut WorldState, delta: f32) {
    let hosts: Vec<_> = world
        .entities
        .values()
        .filter(|e| matches!(e.type_name.as_str(), "waterCaltrop" | "lotus"))
        .map(|e| (e.id, e.type_name.clone(), e.x, e.y, e.harvest_cooldown))
        .collect();
    for (id, type_name, x, y, mut cd) in hosts {
        cd = (cd - delta).max(0.0);
        if cd <= 0.0 {
            let product = if type_name == "waterCaltrop" {
                "caltropFruit"
            } else {
                "lotusSeed"
            };
            if !world.has_tag_at(x, y, product) {
                world.spawn(product, x, y);
            }
            cd = POOL_HARVEST_REGEN_SECONDS;
        }
        if let Some(e) = world.entities.get_mut(&id) {
            e.harvest_cooldown = cd;
        }
    }
}

pub fn entities_in_tree(world: &WorldState, tree_id: crate::spatial_index::EntityId) -> Vec<crate::spatial_index::EntityId> {
    world
        .entities
        .values()
        .filter(|e| e.host_tree_id == Some(tree_id) || e.in_tree)
        .map(|e| e.id)
        .collect()
}

pub fn entities_in_pool(world: &WorldState, x: u8, y: u8) -> Vec<crate::spatial_index::EntityId> {
    world
        .entities
        .values()
        .filter(|e| e.in_pool && e.x == x && e.y == y)
        .map(|e| e.id)
        .collect()
}

pub fn entities_underground(world: &WorldState, x: u8, y: u8) -> Vec<crate::spatial_index::EntityId> {
    world
        .entities
        .values()
        .filter(|e| e.in_ground && e.x == x && e.y == y)
        .map(|e| e.id)
        .collect()
}
