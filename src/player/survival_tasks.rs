//! Survival tasks — forage / eat / flee.

use crate::game_constants::{PLAYER_HUNGER_SEEK, PLAYER_PREDATOR_PERCEPTION_RANGE};
use crate::spatial_index::EntityId;
use crate::systems::movement::{flee_from, move_toward};
use crate::world_rules::{card_has_tag, chebyshev_distance};
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

pub fn force_forage_needed(world: &WorldState, player_id: EntityId, mind: &PlayerMind) -> bool {
    let starve_days = world
        .entities
        .get(&player_id)
        .map(|e| e.starve_days)
        .unwrap_or(0);
    starve_days >= 1
        || mind.hunger >= PLAYER_HUNGER_SEEK
        || super::brain_tags::has_tag(mind, "hungry")
        || super::brain_tags::has_tag(mind, "food_seek")
}

pub fn predator_threat_near(
    world: &WorldState,
    player_id: EntityId,
    px: u8,
    py: u8,
) -> Option<(u8, u8)> {
    let threats = world.query_near_filtered(
        px,
        py,
        "predator",
        PLAYER_PREDATOR_PERCEPTION_RANGE,
        player_id,
    );
    threats
        .iter()
        .filter_map(|id| world.spatial_index.position(*id))
        .min_by_key(|(tx, ty)| chebyshev_distance(px, py, *tx, *ty))
}

fn is_player_food_type(type_name: &str) -> bool {
    matches!(type_name, "berry" | "grass" | "bush")
}

fn food_targets_by_distance(world: &WorldState, px: u8, py: u8) -> Vec<(EntityId, u8, u8, String)> {
    let mut targets: Vec<(EntityId, u8, u8, String)> = Vec::new();
    for id in world
        .spatial_index
        .query_near(px, py, "grass", PERCEPTION_SCAN_RADIUS)
        .into_iter()
        .chain(world.spatial_index.query_near(px, py, "bush", PERCEPTION_SCAN_RADIUS))
    {
        if let Some(e) = world.entities.get(&id) {
            if !e.is_corpse && is_player_food_type(&e.type_name) {
                targets.push((id, e.x, e.y, e.type_name.clone()));
            }
        }
    }
    for e in world.entities.values() {
        if e.is_corpse || !is_player_food_type(&e.type_name) {
            continue;
        }
        if chebyshev_distance(px, py, e.x, e.y) <= PERCEPTION_SCAN_RADIUS {
            targets.push((e.id, e.x, e.y, e.type_name.clone()));
        }
    }
    targets.sort_by_key(|(_, x, y, ty)| {
        let dist = chebyshev_distance(px, py, *x, *y);
        let tier = match ty.as_str() {
            "berry" => 0,
            "bush" => 1,
            "grass" => 2,
            _ => 3,
        };
        (tier, dist)
    });
    targets.dedup_by_key(|(id, _, _, _)| *id);
    targets
}

const PERCEPTION_SCAN_RADIUS: u8 = 24;

fn mark_player_fed(world: &mut WorldState, player_id: EntityId, mind: &mut PlayerMind, hunger_delta: f32) {
    mind.hunger = (mind.hunger - hunger_delta).max(0.0);
    if let Some(e) = world.entities.get_mut(&player_id) {
        e.starve_days = 0;
        e.fed_today = true;
        e.fed = true;
    }
}

fn eat_neighbor_food(
    world: &mut WorldState,
    player_id: EntityId,
    mind: &mut PlayerMind,
    food_id: EntityId,
    food_type: &str,
) -> bool {
    match food_type {
        "berry" => {
            world.remove_entity(food_id);
            mark_player_fed(world, player_id, mind, 35.0);
            mind.goal_text = "觅食完成".into();
            true
        }
        "grass" => {
            let remove = {
                if let Some(g) = world.entities.get_mut(&food_id) {
                    g.hp = (g.hp - 1).max(0);
                    g.hp <= 0
                } else {
                    false
                }
            };
            if remove {
                world.remove_entity(food_id);
            }
            mark_player_fed(world, player_id, mind, 25.0);
            mind.goal_text = "啃草完成".into();
            true
        }
        "bush" => {
            mind.goal_text = "灌木觅食".into();
            mark_player_fed(world, player_id, mind, 20.0);
            true
        }
        _ => {
            if world.card_defs.get(food_type).is_some_and(|d| card_has_tag(d, "foodSource")) {
                if food_type == "grass" {
                    return eat_neighbor_food(world, player_id, mind, food_id, "grass");
                }
                mark_player_fed(world, player_id, mind, 20.0);
                mind.goal_text = "觅食完成".into();
                true
            } else {
                false
            }
        }
    }
}

pub fn satisfy_hunger(world: &mut WorldState, player_id: EntityId, mind: &mut PlayerMind) -> bool {
    if !force_forage_needed(world, player_id, mind) && !should_forage(mind) {
        return false;
    }
    let (px, py) = {
        let e = &world.entities[&player_id];
        (e.x, e.y)
    };

    for (food_id, fx, fy, food_type) in food_targets_by_distance(world, px, py) {
        if is_neighbor(px, py, fx, fy) {
            if eat_neighbor_food(world, player_id, mind, food_id, &food_type) {
                mind.state_label = "觅食".into();
                return true;
            }
        }
        move_toward(world, player_id, px, py, fx, fy);
        mind.goal_text = match food_type.as_str() {
            "grass" => "前往草皮",
            "bush" => "前往灌木觅食",
            "berry" => "前往浆果",
            _ => "前往食物",
        }
        .into();
        mind.state_label = "觅食".into();
        return true;
    }

    for berry_id in sorted_by_distance(world, px, py, "berry") {
        let Some(berry) = world.entities.get(&berry_id) else {
            continue;
        };
        if is_neighbor(px, py, berry.x, berry.y) {
            return eat_neighbor_food(world, player_id, mind, berry_id, "berry");
        }
        move_toward(world, player_id, px, py, berry.x, berry.y);
        mind.goal_text = "前往浆果".into();
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
    let Some((tx, ty)) = predator_threat_near(world, player_id, px, py) else {
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
