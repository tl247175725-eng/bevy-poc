//! Runtime brain tags — Godot `player_brain_tags.gd`.

use std::collections::HashMap;

use crate::card_def::CardDef;
use crate::game_constants::{
    PLAYER_HUNGER_NEED, PLAYER_HUNGER_SEEK, PLAYER_WOLF_THREAT_DIST,
};
use crate::spatial_index::EntityId;
use crate::world_rules::{card_has_tag, chebyshev_distance, in_range};
use crate::world_state::{Entity, WorldState};

use super::state::PlayerMind;

pub fn sync_player_tags(world: &WorldState, player_id: EntityId, mind: &mut PlayerMind) {
    let Some(player) = world.entities.get(&player_id) else {
        return;
    };
    let tags = compute_runtime_tags(world, player, mind);
    mind.runtime_tags = tags;
}

pub fn has_tag(mind: &PlayerMind, tag: &str) -> bool {
    if mind.runtime_tags.get(tag).copied().unwrap_or(false) {
        return true;
    }
    matches!(
        tag,
        "fire_bond" | "tool_dependent" | "omnivore" | "actor"
    )
}

pub fn fire_bond_satisfied(world: &WorldState) -> bool {
    world
        .entities
        .values()
        .any(|e| e.type_name == "fire" && !e.is_corpse)
}

pub fn owns_tool_type(mind: &PlayerMind, type_name: &str) -> bool {
    mind.tools.iter().any(|t| t == type_name)
}

fn compute_runtime_tags(
    world: &WorldState,
    player: &Entity,
    mind: &PlayerMind,
) -> HashMap<String, bool> {
    let mut tags = HashMap::new();
    tags.insert(
        "hungry".into(),
        mind.hunger >= PLAYER_HUNGER_NEED,
    );
    tags.insert(
        "food_seek".into(),
        mind.hunger >= PLAYER_HUNGER_SEEK,
    );
    tags.insert(
        "has_weapon".into(),
        owns_sharp_tool(world, player, mind),
    );
    tags.insert("has_tools".into(), !mind.tools.is_empty());
    tags.insert(
        "recent_hunt_success".into(),
        mind.recent_hunt_success,
    );
    tags.insert("has_shelter".into(), shelter_exists(world));
    tags.insert(
        "near_camp".into(),
        near_camp_anchor(world, player.x, player.y),
    );
    tags.insert(
        "fire_bond_satisfied".into(),
        fire_bond_satisfied(world),
    );
    tags.insert(
        "predator_nearby_unsafe".into(),
        predator_nearby_unsafe(world, player),
    );
    tags.insert(
        "survival_surplus".into(),
        survival_surplus(world, player, mind, &tags),
    );
    tags
}

fn owns_sharp_tool(world: &WorldState, player: &Entity, mind: &PlayerMind) -> bool {
    for tool in &mind.tools {
        if card_has_sharp_tag(world, tool) {
            return true;
        }
    }
    if let Some(cid) = player.carrying {
        if let Some(carried) = world.entities.get(&cid) {
            return world
                .card_defs
                .get(&carried.type_name)
                .is_some_and(|d| card_has_tag(d, "sharp"));
        }
    }
    false
}

fn card_has_sharp_tag(world: &WorldState, type_name: &str) -> bool {
    world
        .card_defs
        .get(type_name)
        .is_some_and(|d| card_has_tag(d, "sharp"))
}

fn shelter_exists(world: &WorldState) -> bool {
    world
        .entities
        .values()
        .any(|e| e.type_name == "hut" && !e.is_corpse)
}

fn near_camp_anchor(world: &WorldState, x: u8, y: u8) -> bool {
    for e in world.entities.values() {
        if e.type_name == "fire" || e.type_name == "hut" {
            if in_range(x, y, e.x, e.y, PLAYER_WOLF_THREAT_DIST) {
                return true;
            }
        }
    }
    false
}

fn near_fire(world: &WorldState, x: u8, y: u8) -> bool {
    world.entities.values().any(|e| {
        e.type_name == "fire" && in_range(x, y, e.x, e.y, PLAYER_WOLF_THREAT_DIST)
    })
}

fn predator_nearby_unsafe(world: &WorldState, player: &Entity) -> bool {
    if near_fire(world, player.x, player.y) {
        return false;
    }
    nearest_predator(world, player.x, player.y, PLAYER_WOLF_THREAT_DIST).is_some()
}

fn nearest_predator(world: &WorldState, x: u8, y: u8, range: u8) -> Option<EntityId> {
    world
        .spatial_index
        .query_near(x, y, "predator", range)
        .into_iter()
        .filter(|id| {
            world
                .entities
                .get(id)
                .is_some_and(|e| !e.is_corpse && !e.in_den)
        })
        .min_by_key(|id| {
            world
                .spatial_index
                .position(*id)
                .map(|(ex, ey)| chebyshev_distance(x, y, ex, ey))
                .unwrap_or(u8::MAX)
        })
}

fn survival_surplus(
    world: &WorldState,
    player: &Entity,
    mind: &PlayerMind,
    tags: &HashMap<String, bool>,
) -> bool {
    if !fire_bond_satisfied(world) {
        return false;
    }
    if tags.get("hungry").copied().unwrap_or(false) {
        return false;
    }
    if tags.get("predator_nearby_unsafe").copied().unwrap_or(false) {
        return false;
    }
    let _ = player;
    let _ = mind;
    true
}

pub fn card_has_material_tag(def: &CardDef, tag: &str) -> bool {
    def.tags.iter().any(|t| t == tag || t.contains(tag))
}
