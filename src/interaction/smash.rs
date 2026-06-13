//! Per-hit smash damage — 1 hit = 1 HP (drag + AI + impact recipes).

use crate::axioms::{AxiomEngine, TransformAction};
use crate::sim_events::{SimEvent, SimEventQueue};
use crate::spatial_index::EntityId;
use crate::world_rules::{
    card_has_capability, card_has_tag, corpse_type_for, mark_ecology_fed, parse_meat_product,
    parse_meat_yield,
};
use crate::world_state::WorldState;

use super::{display_name, InteractionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmashOutcome {
    Hit,
    Killed,
    RecipeComplete,
    NoEffect,
}

const CORPSE_BUTCHER_HITS: u32 = 2;

pub fn is_corpse_entity(world: &WorldState, type_name: &str, is_corpse: bool) -> bool {
    is_corpse
        || type_name.ends_with("Corpse")
        || world
            .card_defs
            .get(type_name)
            .is_some_and(|d| card_has_tag(d, "corpse"))
}

pub fn butcher_corpse(world: &mut WorldState, corpse_id: EntityId) {
    let Some(corpse) = world.entities.get(&corpse_id).cloned() else {
        return;
    };
    let Some(def) = world.card_defs.get(&corpse.type_name) else {
        return;
    };
    let meat_count = parse_meat_yield(def);
    let Some(meat_type) = parse_meat_product(def) else {
        world.remove_entity(corpse_id);
        return;
    };
    let (x, y) = (corpse.x, corpse.y);
    world.remove_entity(corpse_id);
    for _ in 0..meat_count {
        world.spawn(&meat_type, x, y);
    }
}

pub fn damage_entity_hp(world: &mut WorldState, target: EntityId, amount: i32) -> bool {
    let Some(entity) = world.entities.get_mut(&target) else {
        return false;
    };
    if entity.hp <= 0 {
        return false;
    }
    entity.hp = (entity.hp - amount).max(0);
    entity.wake();
    entity.hp <= 0
}

pub fn apply_smash_hit(
    world: &mut WorldState,
    source: EntityId,
    target: EntityId,
    state: &mut InteractionState,
    events: &mut SimEventQueue,
) -> SmashOutcome {
    let Some(src) = world.entities.get(&source).cloned() else {
        return SmashOutcome::NoEffect;
    };
    let Some(tgt) = world.entities.get(&target).cloned() else {
        return SmashOutcome::NoEffect;
    };
    if source == target {
        return SmashOutcome::NoEffect;
    }
    if tgt.in_cover {
        return SmashOutcome::NoEffect;
    }

    let src_name = display_name(world, &src.type_name);
    let tgt_name = display_name(world, &tgt.type_name);
    let key = (source, target);
    let hits = state.hit_counts.entry(key).or_insert(0);
    *hits += 1;
    let hit_n = *hits;

    let _ = AxiomEngine::transform(&tgt.profile, &src.profile, TransformAction::Smash);

    if is_corpse_entity(world, &tgt.type_name, tgt.is_corpse) {
        events.push(SimEvent::Impact {
            source: src_name.clone(),
            target: tgt_name.clone(),
            x: tgt.x,
            y: tgt.y,
        });
        if hit_n < CORPSE_BUTCHER_HITS {
            return SmashOutcome::Hit;
        }
        *hits = 0;
        butcher_corpse(world, target);
        return SmashOutcome::RecipeComplete;
    }

    if let Some(recipe) = state.book.resolve_impact(&src.type_name, &tgt.type_name) {
        events.push(SimEvent::Impact {
            source: src_name.clone(),
            target: tgt_name.clone(),
            x: tgt.x,
            y: tgt.y,
        });
        if hit_n < recipe.hits_required {
            return SmashOutcome::Hit;
        }
        *hits = 0;
        super::apply_impact_recipe(world, recipe, &src, &tgt);
        return SmashOutcome::RecipeComplete;
    }

    if tgt.hp > 0 {
        events.push(SimEvent::Impact {
            source: src_name,
            target: tgt_name.clone(),
            x: tgt.x,
            y: tgt.y,
        });
        if damage_entity_hp(world, target, 1) {
            let killer_type = src.type_name.clone();
            finalize_prey_kill(world, target, Some(source), killer_type.as_str());
            *hits = 0;
            return SmashOutcome::Killed;
        }
        return SmashOutcome::Hit;
    }

    events.push(SimEvent::Generic(format!("{src_name} 碰到 {tgt_name}，无有效配方")));
    SmashOutcome::NoEffect
}

pub fn apply_hunt_smash(
    world: &mut WorldState,
    hunter_id: EntityId,
    prey_id: EntityId,
    hunter_def: &crate::card_def::CardDef,
) -> SmashOutcome {
    if world
        .entities
        .get(&hunter_id)
        .is_some_and(|e| e.hunt_cooldown > 0.0)
    {
        return SmashOutcome::NoEffect;
    }

    let prey = world.entities.get(&prey_id).cloned();
    let hunter = world.entities.get(&hunter_id).cloned();
    let (prey_ent, hunter_ent) = match (prey, hunter) {
        (Some(p), Some(h)) => (p, h),
        _ => return SmashOutcome::NoEffect,
    };

    let prey_profile = prey_ent.profile.clone();
    let hunter_profile = hunter_ent.profile.clone();
    let _ = AxiomEngine::transform(&prey_profile, &hunter_profile, TransformAction::Smash);

    let src_name = display_name(world, &hunter_ent.type_name);
    let tgt_name = display_name(world, &prey_ent.type_name);
    world.pending_events.push(SimEvent::Impact {
        source: src_name,
        target: tgt_name,
        x: prey_ent.x,
        y: prey_ent.y,
    });

    let killed = if prey_ent.hp > 0 {
        damage_entity_hp(world, prey_id, 1)
    } else {
        true
    };

    if let Some(hunter) = world.entities.get_mut(&hunter_id) {
        hunter.hunt_cooldown = crate::game_constants::TICK_SECONDS;
    }

    if !killed {
        if let Some(prey) = world.entities.get_mut(&prey_id) {
            prey.needs_grazing_tick = true;
        }
    }

    if killed {
        let hunter_type = hunter_def.type_name.clone();
        finalize_prey_kill(world, prey_id, Some(hunter_id), hunter_type.as_str());
        SmashOutcome::Killed
    } else {
        SmashOutcome::Hit
    }
}

pub fn finalize_prey_kill(
    world: &mut WorldState,
    prey_id: EntityId,
    killer_id: Option<EntityId>,
    killer_type: &str,
) {
    let Some(prey) = world.entities.get(&prey_id).cloned() else {
        return;
    };
    if prey.is_corpse {
        return;
    }
    let prey_type = prey.type_name.clone();
    let (prey_profile, hunter_profile, px, py) = {
        let prey_ref = &prey;
        let hunter_prof = killer_id
            .and_then(|id| world.entities.get(&id))
            .map(|h| h.profile.clone())
            .unwrap_or_else(|| prey_ref.profile.clone());
        (
            prey_ref.profile.clone(),
            hunter_prof,
            prey_ref.x,
            prey_ref.y,
        )
    };
    let _kill = AxiomEngine::transform(&prey_profile, &hunter_profile, TransformAction::Kill);

    world.remove_entity(prey_id);

    let corpse_type = corpse_type_for(world, &prey_type);
    if corpse_type == "none" || !world.card_defs.contains_key(&corpse_type) {
        if let Some(kid) = killer_id {
            crate::sim_observer::on_kill(world, killer_type, &prey_type, px, py);
            if let Some(kdef) = world.card_defs.get(killer_type) {
                if let Some(hunter) = world.entities.get_mut(&kid) {
                    mark_ecology_fed(hunter, kdef);
                }
            }
        }
        return;
    }

    let old_corpses: Vec<EntityId> = world
        .entities_at(px, py)
        .into_iter()
        .filter(|id| {
            world
                .entities
                .get(id)
                .is_some_and(|e| e.is_corpse || e.type_name.ends_with("Corpse"))
        })
        .collect();
    for old in old_corpses {
        world.remove_entity(old);
    }
    let corpse_id = world.spawn(&corpse_type, px, py);
    if let Some(kid) = killer_id {
        crate::sim_observer::on_kill(world, killer_type, &prey_type, px, py);
        if let Some(kdef) = world.card_defs.get(killer_type) {
            if card_has_capability(kdef, "capability.carry") {
                if let Some(hunter) = world.entities.get_mut(&kid) {
                    hunter.carrying = Some(corpse_id);
                    mark_ecology_fed(hunter, kdef);
                }
            } else if let Some(hunter) = world.entities.get_mut(&kid) {
                mark_ecology_fed(hunter, kdef);
            }
        }
    }
    if let Some(corpse) = world.entities.get_mut(&corpse_id) {
        corpse.is_corpse = true;
        corpse.decay_timer = 0.0;
    }
}

#[cfg(test)]
mod tests {
    // 测试已禁用——行为实体卡已删除，等待新卡定义后重新编写
}
