//! Shelter tasks — build hut when threatened.

use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

use super::brain_world::{is_neighbor, sorted_by_distance};
use super::state::PlayerMind;

pub fn build_hut_affordable(world: &WorldState, player_id: EntityId, mind: &PlayerMind) -> bool {
    mind.affordances.contains_key("build_hut")
        && has_hut_materials(world, player_id)
}

fn has_hut_materials(world: &WorldState, player_id: EntityId) -> bool {
    let (px, py) = {
        let e = &world.entities[&player_id];
        (e.x, e.y)
    };
    let twig = !sorted_by_distance(world, px, py, "twig").is_empty()
        || !sorted_by_distance(world, px, py, "wood").is_empty();
    let grass = !sorted_by_distance(world, px, py, "grass").is_empty()
        || !sorted_by_distance(world, px, py, "dryGrass").is_empty();
    twig && grass
}

pub fn plan_build_hut(world: &WorldState, player_id: EntityId, mind: &mut PlayerMind) -> bool {
    if !build_hut_affordable(world, player_id, mind) {
        return false;
    }
    mind.goal_text = "搭建草棚".into();
    mind.state_label = "建造".into();
    true
}

pub fn craft_hut_relation(world: &mut WorldState, wood_id: EntityId, grass_id: EntityId) -> Option<String> {
    let (tx, ty) = {
        let g = world.entities.get(&grass_id)?;
        (g.x, g.y)
    };
    let wood_type = world.entities.get(&wood_id)?.type_name.clone();
    let grass_type = world.entities.get(&grass_id)?.type_name.clone();
    let interaction = crate::interaction::InteractionState::default();
    crate::interaction::try_relation(world, &wood_type, &grass_type, tx, ty, &interaction)
}

pub fn materials_near_player(world: &WorldState, player_id: EntityId) -> bool {
    has_hut_materials(world, player_id)
}

pub fn neighbor_materials(world: &WorldState, player_id: EntityId) -> bool {
    let (px, py) = {
        let e = &world.entities[&player_id];
        (e.x, e.y)
    };
    for e in world.entities.values() {
        if matches!(e.type_name.as_str(), "twig" | "wood" | "grass" | "dryGrass") {
            if is_neighbor(px, py, e.x, e.y) {
                return true;
            }
        }
    }
    false
}
