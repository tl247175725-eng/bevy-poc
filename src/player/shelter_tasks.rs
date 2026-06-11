//! Shelter tasks — build hut when threatened.

use crate::spatial_index::EntityId;
use crate::systems::movement::move_toward;
use crate::world_state::WorldState;

use super::brain_world::{is_neighbor, sorted_by_distance};
use super::state::PlayerMind;

pub fn build_hut_affordable(world: &WorldState, player_id: EntityId, mind: &PlayerMind) -> bool {
    mind.affordances.contains_key("build_hut")
        && has_hut_materials(world, player_id)
}

pub fn has_hut_materials(world: &WorldState, player_id: EntityId) -> bool {
    hut_material_ids(world, player_id).is_some()
}

fn hut_material_ids(world: &WorldState, player_id: EntityId) -> Option<(EntityId, EntityId)> {
    let (px, py) = {
        let e = &world.entities[&player_id];
        (e.x, e.y)
    };
    let twig = sorted_by_distance(world, px, py, "twig")
        .into_iter()
        .next()
        .or_else(|| sorted_by_distance(world, px, py, "wood").into_iter().next())?;
    let grass = sorted_by_distance(world, px, py, "grass")
        .into_iter()
        .next()
        .or_else(|| sorted_by_distance(world, px, py, "dryGrass").into_iter().next())?;
    Some((twig, grass))
}

pub fn plan_build_hut(world: &mut WorldState, player_id: EntityId, mind: &mut PlayerMind) -> bool {
    execute_build_hut(world, player_id, mind)
}

/// Headless path — move to twig+grass and craft hut via relation recipe.
pub fn execute_build_hut(world: &mut WorldState, player_id: EntityId, mind: &mut PlayerMind) -> bool {
    if !build_hut_affordable(world, player_id, mind) {
        return false;
    }
    let Some((twig_id, grass_id)) = hut_material_ids(world, player_id) else {
        return false;
    };
    let (px, py) = {
        let e = &world.entities[&player_id];
        (e.x, e.y)
    };
    let twig = world.entities.get(&twig_id);
    let grass = world.entities.get(&grass_id);
    let (Some(twig), Some(grass)) = (twig, grass) else {
        return false;
    };

    if is_neighbor(px, py, twig.x, twig.y) && is_neighbor(px, py, grass.x, grass.y) {
        if craft_hut_relation(world, twig_id, grass_id).is_some() {
            mind.goal_text = "草棚搭好".into();
            mind.state_label = "建造".into();
            return true;
        }
        return false;
    }

    let twig_dist = crate::world_rules::chebyshev_distance(px, py, twig.x, twig.y);
    let grass_dist = crate::world_rules::chebyshev_distance(px, py, grass.x, grass.y);
    let (tx, ty) = if twig_dist <= grass_dist {
        (twig.x, twig.y)
    } else {
        (grass.x, grass.y)
    };
    move_toward(world, player_id, px, py, tx, ty);
    mind.goal_text = "前往搭建材料".into();
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
