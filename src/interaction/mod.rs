//! Interaction recipes — impact/relation/harvest (Godot `interaction_manager.gd`).

use bevy::prelude::*;

use crate::world_state::WorldState;

mod recipes;

use std::collections::HashMap;

use crate::card_def::CardDef;
use crate::sim_events::{SimEvent, SimEventQueue};
use crate::spatial_index::EntityId;

pub use recipes::{ImpactRecipe, RelationRecipe, RecipeBook};

#[derive(Resource)]
pub struct InteractionState {
    pub book: RecipeBook,
    /// source entity → target entity → hit count
    pub hit_counts: HashMap<(EntityId, EntityId), u32>,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            book: RecipeBook::load_embedded(),
            hit_counts: HashMap::new(),
        }
    }
}

pub fn try_impact(
    world: &mut WorldState,
    source: EntityId,
    target: EntityId,
    state: &mut InteractionState,
    events: &mut SimEventQueue,
) -> bool {
    let Some(src) = world.entities.get(&source).cloned() else {
        return false;
    };
    let Some(tgt) = world.entities.get(&target).cloned() else {
        return false;
    };
    if source == target {
        return false;
    }

    let recipe = state
        .book
        .resolve_impact(&src.type_name, &tgt.type_name);
    let key = (source, target);
    let hits = state.hit_counts.entry(key).or_insert(0);
    *hits += 1;

    let src_name = display_name(world, &src.type_name);
    let tgt_name = display_name(world, &tgt.type_name);

    if let Some(recipe) = recipe {
        if *hits < recipe.hits_required {
            events.push(SimEvent::Impact {
                source: src_name.clone(),
                target: tgt_name.clone(),
                x: tgt.x,
                y: tgt.y,
            });
            return true;
        }
        *hits = 0;
        apply_impact_recipe(world, &recipe, &src, &tgt);
        events.push(SimEvent::Impact {
            source: src_name,
            target: tgt_name,
            x: tgt.x,
            y: tgt.y,
        });
        return true;
    }

    events.push(SimEvent::Generic(format!("{src_name} 碰到 {tgt_name}，无有效配方")));
    *hits >= 1
}

pub fn try_relation(
    world: &mut WorldState,
    a_type: &str,
    b_type: &str,
    x: u8,
    y: u8,
    state: &InteractionState,
) -> Option<String> {
    state
        .book
        .find_relation(a_type, b_type)
        .map(|r| {
            world.spawn(&r.result, x, y);
            r.result.clone()
        })
}

pub fn try_ghost_drop(
    world: &mut WorldState,
    source: EntityId,
    gx: u8,
    gy: u8,
    origin_x: u8,
    origin_y: u8,
    interaction: &InteractionState,
    events: &mut SimEventQueue,
) -> bool {
    let Some(src) = world.entities.get(&source).cloned() else {
        return false;
    };
    if gx == origin_x && gy == origin_y {
        return true;
    }

    let targets: Vec<EntityId> = world
        .entities_at(gx, gy)
        .into_iter()
        .filter(|id| *id != source)
        .collect();

    if let Some(target_id) = targets.first() {
        let tgt_type = world
            .entities
            .get(target_id)
            .map(|e| e.type_name.clone())
            .unwrap_or_default();
        if let Some(result) = try_relation(world, &src.type_name, &tgt_type, gx, gy, interaction) {
            world.remove_entity(source);
            events.push(SimEvent::Generic(format!(
                "{} + {} → {}",
                display_name(world, &src.type_name),
                display_name(world, &tgt_type),
                display_name(world, &result)
            )));
            return true;
        }
    }

    if targets.is_empty() {
        if crate::terrain::is_blocked_terrain(crate::terrain::terrain_at(world, gx, gy)) {
            let _ = world.move_entity(source, origin_x, origin_y);
            world.reindex_entity(source);
            return false;
        }
        if let Some(profile) = world.entities.get(&source).map(|e| e.profile.clone()) {
            if !world.cell_composition.can_occupy(gx, gy, &profile) {
                let _ = world.move_entity(source, origin_x, origin_y);
                world.reindex_entity(source);
                return false;
            }
        }
        if world.move_entity(source, gx, gy) != crate::world_state::MoveResult::Moved {
            let _ = world.move_entity(source, origin_x, origin_y);
            world.reindex_entity(source);
            return false;
        }
        world.reindex_entity(source);
        return true;
    }

    let _ = world.move_entity(source, origin_x, origin_y);
    world.reindex_entity(source);
    false
}

pub fn try_harvest(
    world: &mut WorldState,
    x: u8,
    y: u8,
    events: &mut SimEventQueue,
) -> Option<String> {
    let product = crate::systems::tick_harvest::harvest_at(world, x, y, "")?;
    events.push(SimEvent::Harvest {
        product: product.clone(),
        x,
        y,
    });
    Some(product)
}

fn apply_impact_recipe(
    world: &mut WorldState,
    recipe: &ImpactRecipe,
    _source: &crate::world_state::Entity,
    target: &crate::world_state::Entity,
) {
    if !recipe.handler_id.is_empty() {
        return;
    }
    if recipe.consumes_target {
        world.remove_entity(target.id);
    }
    if !recipe.result.is_empty() {
        world.spawn(&recipe.result, target.x, target.y);
        if !recipe.extra_result.is_empty() {
            let ox = target.x.saturating_add(recipe.extra_offset.0.max(0) as u8);
            let oy = target.y.saturating_add(recipe.extra_offset.1.max(0) as u8);
            world.spawn(&recipe.extra_result, ox, oy);
        }
    }
}

fn display_name(world: &WorldState, type_name: &str) -> String {
    world
        .card_defs
        .get(type_name)
        .map(|d| d.display_name.clone())
        .unwrap_or_else(|| type_name.to_string())
}

pub fn card_matches_tag(def: &CardDef, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.ends_with("_like") {
        let base = &pattern[..pattern.len() - 5];
        return def.tags.iter().any(|t| t.contains(base));
    }
    def.tags.iter().any(|t| t == pattern) || def.type_name == pattern
}
