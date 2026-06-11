//! World-model queries — Godot `player_brain_world.gd`.

use crate::card_def::CardDef;
use crate::game_constants::{
    PERCEPTION_SCAN_BERRY, PERCEPTION_SCAN_LUMBER, PERCEPTION_SCAN_STONE,
};
use crate::spatial_index::EntityId;
use crate::world_rules::{card_has_tag, chebyshev_distance, in_range};
use crate::world_state::{Entity, WorldState};

use super::brain_tags::{card_has_material_tag, owns_tool_type};
use super::state::PlayerMind;

pub fn check_all(world: &WorldState, player_id: EntityId, mind: &PlayerMind, conditions: &[&str]) -> bool {
    conditions
        .iter()
        .all(|c| check(world, player_id, mind, c))
}

pub fn check(world: &WorldState, player_id: EntityId, mind: &PlayerMind, condition: &str) -> bool {
    let Some(player) = world.entities.get(&player_id) else {
        return false;
    };
    match condition {
        "prey_available" => prey_available(world, player),
        "hunt_quota_ok" => true,
        "camp_fire_exists" => super::brain_tags::fire_bond_satisfied(world),
        "no_camp_fire" => !super::brain_tags::fire_bond_satisfied(world),
        "wood_nearby" => card_with_tag_in_range(world, player, "material.lumber", PERCEPTION_SCAN_LUMBER),
        "stones_available" => {
            count_cards_with_tag(world, player, mind, "material.stone", PERCEPTION_SCAN_STONE) >= 2
        }
        "stone_available" => {
            count_cards_with_tag(world, player, mind, "material.stone", PERCEPTION_SCAN_STONE) >= 1
        }
        "no_knife_owned" => !owns_tool_type(mind, "knife"),
        "no_spear_owned" => !owns_tool_type(mind, "spear"),
        "berry_bush_nearby" => berry_available_near(world, player),
        "no_shelter_exists" => !super::brain_tags::has_tag(mind, "has_shelter"),
        "hut_materials_nearby" => hut_materials_near(world, player),
        _ => false,
    }
}

fn prey_available(world: &WorldState, player: &Entity) -> bool {
    world
        .spatial_index
        .query_near(player.x, player.y, "herbivore", 8)
        .into_iter()
        .any(|id| {
            world
                .entities
                .get(&id)
                .is_some_and(|e| !e.is_corpse && !e.stunned)
        })
}

fn card_with_tag_in_range(world: &WorldState, player: &Entity, tag: &str, radius: u8) -> bool {
    for e in world.entities.values() {
        if e.is_corpse || e.carrying.is_some() {
            continue;
        }
        if let Some(def) = world.card_defs.get(&e.type_name) {
            if card_has_material_tag(def, tag) && in_range(player.x, player.y, e.x, e.y, radius) {
                return true;
            }
        }
    }
    false
}

fn count_cards_with_tag(
    world: &WorldState,
    player: &Entity,
    mind: &PlayerMind,
    tag: &str,
    radius: u8,
) -> usize {
    let mut n = 0usize;
    if let Some(cid) = player.carrying {
        if let Some(e) = world.entities.get(&cid) {
            if let Some(def) = world.card_defs.get(&e.type_name) {
                if card_has_material_tag(def, tag) {
                    n += 1;
                }
            }
        }
    }
    for e in world.entities.values() {
        if e.is_corpse {
            continue;
        }
        if e.carrying.is_some() {
            continue;
        }
        if let Some(def) = world.card_defs.get(&e.type_name) {
            if card_has_material_tag(def, tag) && in_range(player.x, player.y, e.x, e.y, radius) {
                n += 1;
            }
        }
    }
    let _ = mind;
    n
}

fn hut_materials_near(world: &WorldState, player: &Entity) -> bool {
    let has_twig = !sorted_by_distance(world, player.x, player.y, "twig").is_empty()
        || !sorted_by_distance(world, player.x, player.y, "wood").is_empty();
    let has_grass = !sorted_by_distance(world, player.x, player.y, "grass").is_empty()
        || !sorted_by_distance(world, player.x, player.y, "dryGrass").is_empty();
    has_twig && has_grass
}

fn berry_available_near(world: &WorldState, player: &Entity) -> bool {
    if card_with_tag_in_range(world, player, "berry.source", PERCEPTION_SCAN_BERRY) {
        return true;
    }
    for e in world.entities.values() {
        if e.is_corpse {
            continue;
        }
        if !in_range(player.x, player.y, e.x, e.y, PERCEPTION_SCAN_BERRY) {
            continue;
        }
        if matches!(e.type_name.as_str(), "bush" | "berry" | "grass") {
            return true;
        }
        if world
            .card_defs
            .get(&e.type_name)
            .is_some_and(|d| card_has_tag(d, "foodSource"))
        {
            return true;
        }
    }
    false
}

pub fn nearest_entity_of_type(
    world: &WorldState,
    px: u8,
    py: u8,
    type_name: &str,
    radius: u8,
) -> Option<EntityId> {
    world
        .entities
        .values()
        .filter(|e| e.type_name == type_name && in_range(px, py, e.x, e.y, radius))
        .min_by_key(|e| chebyshev_distance(px, py, e.x, e.y))
        .map(|e| e.id)
}

pub fn is_neighbor(ax: u8, ay: u8, bx: u8, by: u8) -> bool {
    chebyshev_distance(ax, ay, bx, by) <= 1
}

pub fn sorted_by_distance(world: &WorldState, px: u8, py: u8, type_name: &str) -> Vec<EntityId> {
    let mut ids: Vec<EntityId> = world
        .entities
        .values()
        .filter(|e| e.type_name == type_name && !e.is_corpse)
        .map(|e| e.id)
        .collect();
    ids.sort_by_key(|id| {
        world
            .entities
            .get(id)
            .map(|e| chebyshev_distance(px, py, e.x, e.y))
            .unwrap_or(u8::MAX)
    });
    ids
}

pub fn card_def_has_tag(def: &CardDef, tag: &str) -> bool {
    card_has_tag(def, tag)
}
