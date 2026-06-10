//! Survival tasks — forage / eat / flee.

use crate::spatial_index::EntityId;
use crate::systems::movement::{flee_from, move_toward};
use crate::world_rules::chebyshev_distance;
use crate::world_state::WorldState;

use super::brain_world::{is_neighbor, sorted_by_distance};
use super::state::PlayerMind;

pub fn should_forage(mind: &PlayerMind) -> bool {
    super::brain_tags::has_tag(mind, "hungry")
        && mind.affordances.contains_key("forage")
}

pub fn should_not_forage_when_full(mind: &PlayerMind) -> bool {
    !super::brain_tags::has_tag(mind, "hungry")
}

pub fn satisfy_hunger(world: &mut WorldState, player_id: EntityId, mind: &mut PlayerMind) -> bool {
    if !should_forage(mind) {
        return false;
    }
    let (px, py) = {
        let e = &world.entities[&player_id];
        (e.x, e.y)
    };

    for berry_id in sorted_by_distance(world, px, py, "berry") {
        let Some(berry) = world.entities.get(&berry_id) else {
            continue;
        };
        if is_neighbor(px, py, berry.x, berry.y) {
            world.remove_entity(berry_id);
            mind.hunger = (mind.hunger - 35.0).max(0.0);
            mind.goal_text = "觅食完成".into();
            return true;
        }
        move_toward(world, player_id, px, py, berry.x, berry.y);
        mind.goal_text = "前往浆果".into();
        mind.state_label = "觅食".into();
        return true;
    }

    for grass_id in sorted_by_distance(world, px, py, "grass") {
        let Some(grass) = world.entities.get(&grass_id) else {
            continue;
        };
        if is_neighbor(px, py, grass.x, grass.y) {
            let remove = {
                if let Some(g) = world.entities.get_mut(&grass_id) {
                    g.hp = (g.hp - 1).max(0);
                    g.hp <= 0
                } else {
                    false
                }
            };
            if remove {
                world.remove_entity(grass_id);
            }
            mind.hunger = (mind.hunger - 25.0).max(0.0);
            mind.goal_text = "啃草完成".into();
            return true;
        }
        move_toward(world, player_id, px, py, grass.x, grass.y);
        mind.goal_text = "前往草皮".into();
        mind.state_label = "觅食".into();
        return true;
    }

    if let Some(bush_id) = sorted_by_distance(world, px, py, "bush").first() {
        let Some(bush) = world.entities.get(bush_id) else {
            return false;
        };
        if !is_neighbor(px, py, bush.x, bush.y) {
            move_toward(world, player_id, px, py, bush.x, bush.y);
        }
        mind.goal_text = "前往灌木觅食".into();
        mind.state_label = "觅食".into();
        return true;
    }
    false
}

pub fn flee_from_threat(world: &mut WorldState, player_id: EntityId, mind: &mut PlayerMind) -> bool {
    let (px, py) = {
        let e = &world.entities[&player_id];
        (e.x, e.y)
    };
    let mut threats: Vec<(EntityId, u8, u8)> = world
        .spatial_index
        .query_near(px, py, "wolf", 8)
        .into_iter()
        .filter_map(|id| world.spatial_index.position(id).map(|pos| (id, pos.0, pos.1)))
        .collect();
    if threats.is_empty() {
        for id in world.spatial_index.query_near(px, py, "predator", 8) {
            if let Some(pos) = world.spatial_index.position(id) {
                threats.push((id, pos.0, pos.1));
            }
        }
    }
    let Some((_, tx, ty)) = threats
        .into_iter()
        .min_by_key(|(_, tx, ty)| chebyshev_distance(px, py, *tx, *ty))
    else {
        return false;
    };
    flee_from(world, player_id, px, py, tx, ty);
    mind.goal_text = "躲避威胁".into();
    mind.state_label = "躲避威胁".into();
    true
}

pub fn hunger_priority_when_starving(mind: &PlayerMind) -> bool {
    super::brain_tags::has_tag(mind, "food_seek")
}
