use crate::game_constants::{CONE_PRODUCE_INTERVAL, NUT_PRODUCE_INTERVAL, POOL_HARVEST_REGEN_SECONDS};
use crate::world_rules::card_has_tag;
use crate::world_state::WorldState;

pub fn tick_contained_producers(world: &mut WorldState, delta: f32) {
    tick_tree_producers(world, delta);
    tick_pool_hosts(world, delta);
}

fn tick_tree_producers(world: &mut WorldState, delta: f32) {
    let trees: Vec<_> = world
        .entities
        .values()
        .filter_map(|e| {
            let def = world.card_defs.get(&e.type_name)?;
            let (product, interval) = if card_has_tag(def, "nut_producer") {
                ("acorn", NUT_PRODUCE_INTERVAL)
            } else if card_has_tag(def, "cone_producer") || card_has_tag(def, "forest") {
                ("pineCone", CONE_PRODUCE_INTERVAL)
            } else {
                return None;
            };
            Some((e.id, product.to_string(), interval, e.x, e.y, e.produce_timer))
        })
        .collect();
    for (id, product, interval, x, y, mut timer) in trees {
        timer += delta;
        if timer >= interval {
            timer = 0.0;
            let product_exists = world.entities.values().any(|e| {
                e.host_tree_id == Some(id) && e.type_name == product
            });
            let host_only = world
                .entities_at(x, y)
                .iter()
                .all(|occupant| *occupant == id);
            if !product_exists
                && !world.has_tag_at(x, y, &product)
                && (world.entities_at(x, y).is_empty() || host_only)
            {
                let drop = world.spawn(&product, x, y);
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
        .filter_map(|e| {
            let def = world.card_defs.get(&e.type_name)?;
            let product = def
                .tags
                .iter()
                .find_map(|t| t.strip_prefix("harvest_product:"))
                .map(str::to_string)?;
            Some((e.id, product, e.x, e.y, e.harvest_cooldown))
        })
        .collect();
    for (id, product, x, y, mut cd) in hosts {
        cd = (cd - delta).max(0.0);
        if cd <= 0.0 {
            if !world.has_tag_at(x, y, &product) {
                world.spawn(&product, x, y);
            }
            cd = POOL_HARVEST_REGEN_SECONDS;
        }
        if let Some(e) = world.entities.get_mut(&id) {
            e.harvest_cooldown = cd;
        }
    }
}

pub fn entities_in_tree(
    world: &WorldState,
    tree_id: crate::spatial_index::EntityId,
) -> Vec<crate::spatial_index::EntityId> {
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
