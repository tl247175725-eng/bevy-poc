use crate::axioms::{AxiomEngine, TransformAction};
use crate::card_def::CardDef;
use crate::ecology_log::eco_log;
use crate::game_constants::WOLF_DEN_DELIVERY_RANGE;
use crate::spatial_index::EntityId;
use crate::systems::movement::{flee_from, move_toward, wander};
use crate::world_rules::{
    card_has_tag, chebyshev_distance, corpse_type_for, hunt_target_score, is_hunt_target_for_pack,
    mark_ecology_fed, HUNT_RANGE,
};
use crate::world_state::{EcologyState, WorldState};

/// Every-tick baseline: all active predators/mesopredators await patrol at tick end.
pub fn mark_baseline_predator_patrol(world: &mut WorldState) {
    for entity in world.entities.values_mut() {
        if entity.is_corpse || entity.in_den {
            continue;
        }
        let Some(def) = world.card_defs.get(&entity.type_name) else {
            continue;
        };
        if card_has_tag(def, "predator") || card_has_tag(def, "mesopredator") {
            entity.needs_patrol = true;
        }
    }
}

/// OnMove prey notify — defer hunt to end-of-tick patrol flush (no per-event patrol).
pub fn mark_predators_near_prey_needs_patrol(world: &mut WorldState, x: u8, y: u8) {
    let mut predators: Vec<EntityId> = world
        .spatial_index
        .query_near(x, y, "predator", HUNT_RANGE)
        .into_iter()
        .chain(
            world
                .spatial_index
                .query_near(x, y, "mesopredator", HUNT_RANGE),
        )
        .collect();
    predators.sort_unstable_by_key(|id| id.0);
    predators.dedup();
    for pid in predators {
        if let Some(entity) = world.entities.get_mut(&pid) {
            if !entity.is_corpse && !entity.in_den {
                entity.needs_patrol = true;
            }
        }
    }
}

/// End of `main_tick` — one patrol pass for all `needs_patrol` predators.
pub fn flush_predator_patrol(world: &mut WorldState, delta: f32) {
    world.tick_scratch.clear();
    world
        .tick_scratch
        .extend(
            world
                .entities
                .values()
                .filter(|e| e.needs_patrol && !e.is_corpse && !e.in_den)
                .map(|e| e.id),
        );
    world.tick_scratch.sort_unstable_by_key(|id| id.0);
    let patrol_len = world.tick_scratch.len();
    for i in 0..patrol_len {
        let id = world.tick_scratch[i];
        if let Some(entity) = world.entities.get_mut(&id) {
            entity.needs_patrol = false;
        }
        let Some(type_name) = world.entities.get(&id).map(|e| e.type_name.as_str()) else {
            continue;
        };
        let Some(def) = world.card_defs.get(type_name).cloned() else {
            continue;
        };
        tick_predator_patrol(world, id, &def, delta);
    }
}

pub fn tick_predators(world: &mut WorldState) {
    let ids: Vec<EntityId> = world
        .entities
        .values()
        .filter(|e| {
            (e.type_name == "wolf" || e.type_name == "fox") && !e.is_corpse && !e.in_den
        })
        .map(|e| e.id)
        .collect();
    for id in ids {
        if let Some(def) = world.card_defs.get(&world.entities[&id].type_name).cloned() {
            tick_predator_patrol(world, id, &def, 1.0);
        }
    }
}

/// Alias for direct / test invocation.
pub fn tick_one_predator(world: &mut WorldState, id: EntityId, def: &CardDef, delta: f32) {
    tick_predator_patrol(world, id, def, delta);
}

/// FixedTimestep patrol — hunt / wander / den work when prey is not moving.
pub fn tick_predator_patrol(world: &mut WorldState, id: EntityId, def: &CardDef, delta: f32) {
    if card_has_tag(def, "juvenile") {
        return;
    }
    let type_name = def.type_name.clone();
    let (x, y, in_den) = {
        let e = &world.entities[&id];
        (e.x, e.y, e.in_den)
    };
    if in_den {
        return;
    }

    if flee_fire(world, id, x, y) {
        return;
    }

    if type_name == "wolf" {
        if den_work_wolf(world, id, x, y, delta) {
            return;
        }
    }
    if type_name == "fox" {
        if den_work_fox(world, id, x, y) {
            return;
        }
        if try_scavenge(world, id, x, y, def) {
            return;
        }
    }

    try_hunt(world, id, x, y, def);
}

fn flee_fire(world: &mut WorldState, id: EntityId, x: u8, y: u8) -> bool {
    for &(fx, fy) in &world.fire_cells {
        if chebyshev_distance(x, y, fx, fy) <= crate::game_constants::FIRE_FEAR_RANGE {
            flee_from(world, id, x, y, fx, fy);
            world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Fleeing;
            world.entities.get_mut(&id).unwrap().hunt_cooldown = 2.0;
            return true;
        }
    }
    false
}

fn den_work_wolf(world: &mut WorldState, id: EntityId, x: u8, y: u8, _delta: f32) -> bool {
    if let Some(carrying) = world.entities.get(&id).and_then(|e| e.carrying) {
        if let Some(den_id) = world.entities.get(&id).and_then(|e| e.den_id) {
            if let Some((dx, dy)) = world.spatial_index.position(den_id) {
                if chebyshev_distance(x, y, dx, dy) <= WOLF_DEN_DELIVERY_RANGE {
                    world.remove_entity(carrying);
                    world.entities.get_mut(&id).unwrap().carrying = None;
                    world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::InDen;
                    return true;
                }
                move_toward(world, id, x, y, dx, dy);
                return true;
            }
        }
    }
    if world.entities.get(&id).and_then(|e| e.den_id).is_none() {
        if world.count_type("wolfDen") == 0 {
            world.spawn("wolfDen", x, y);
            eco_log(world, "狼在远离草棚和火源的边缘形成狼穴");
            if let Some(den) = world
                .entities
                .values()
                .find(|e| e.type_name == "wolfDen" && e.x == x && e.y == y)
                .map(|e| e.id)
            {
                world.entities.get_mut(&id).unwrap().den_id = Some(den);
            }
            return true;
        }
    }
    false
}

fn den_work_fox(world: &mut WorldState, id: EntityId, x: u8, y: u8) -> bool {
    if world.entities.get(&id).and_then(|e| e.den_id).is_some() {
        return false;
    }
    let bush = world
        .entities
        .values()
        .find(|e| e.type_name == "bush" && chebyshev_distance(x, y, e.x, e.y) <= 1)
        .map(|e| (e.id, e.x, e.y));
    if let Some((bush_id, bx, by)) = bush {
        world.remove_entity(bush_id);
        let den_id = world.spawn("foxDen", bx, by);
        eco_log(world, "狐狸占灌木筑成狐窝");
        world.entities.get_mut(&id).unwrap().den_id = Some(den_id);
        return true;
    }
    false
}

fn try_scavenge(world: &mut WorldState, id: EntityId, x: u8, y: u8, def: &CardDef) -> bool {
    let scavenge_cap = world.entities[&id].scavenge_today;
    if scavenge_cap >= crate::game_constants::FOX_SCAVENGE_PER_DAY {
        return false;
    }
    let corpses: Vec<EntityId> = world
        .query_near_filtered(x, y, "corpse", 4, id)
        .into_iter()
        .filter(|cid| {
            world
                .entities
                .get(cid)
                .map(|c| c.type_name != "playerCorpse")
                .unwrap_or(false)
        })
        .collect();
    let Some(corpse_id) = corpses.first() else {
        return false;
    };
    let (cx, cy) = world.spatial_index.position(*corpse_id).unwrap_or((x, y));
    if x == cx && y == cy {
        world.remove_entity(*corpse_id);
        let today = world.entities.get(&id).map(|f| f.scavenge_today + 1).unwrap_or(1);
        if let Some(fox) = world.entities.get_mut(&id) {
            mark_ecology_fed(fox, def);
            fox.scavenge_today = today;
            fox.ecology_state = EcologyState::Scavenging;
        }
        eco_log(
            world,
            format!(
                "狐狸从尸体取肉回窝（今日 {today}/{}）",
                crate::game_constants::FOX_SCAVENGE_PER_DAY
            ),
        );
        return true;
    }
    move_toward(world, id, x, y, cx, cy);
    true
}

fn try_hunt(world: &mut WorldState, id: EntityId, x: u8, y: u8, hunter_def: &CardDef) {
    if world
        .entities
        .get(&id)
        .is_some_and(|e| e.hunt_cooldown > 0.0)
    {
        wander(world, id, x, y, world.tick_count);
        world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Patrolling;
        return;
    }
    let _ = crate::rule_index::dual_track_hunt(crate::rule_index::rule_index(), world, id);
    let _ = crate::rule_index::dual_track_stalk(crate::rule_index::rule_index(), world, id);
    let pack_size = if hunter_def.type_name == "wolf" {
        world.wolf_count()
    } else {
        1
    };
    let prey_ids: Vec<EntityId> = world
        .query_near_filtered(x, y, "herbivore", HUNT_RANGE, id)
        .into_iter()
        .chain(world.query_near_filtered(x, y, "smallPrey", HUNT_RANGE, id))
        .collect();

    let mut best: Option<(EntityId, f32)> = None;
    for prey_id in prey_ids {
        let prey = match world.entities.get(&prey_id) {
            Some(p) if !p.is_corpse => p,
            _ => continue,
        };
        let prey_def = match world.card_defs.get(&prey.type_name) {
            Some(d) => d,
            None => continue,
        };
        if !is_hunt_target_for_pack(hunter_def, prey_def, pack_size) {
            continue;
        }
        if prey.hidden_in_grass && chebyshev_distance(x, y, prey.x, prey.y) > 1 {
            continue;
        }
        let dist = chebyshev_distance(x, y, prey.x, prey.y) as f32;
        let score = hunt_target_score(hunter_def, prey_def, dist, pack_size);
        if score < crate::world_rules::HUNT_SCORE_INF {
            if best.map(|(_, s)| score < s).unwrap_or(true) {
                best = Some((prey_id, score));
            }
        }
    }

    let Some((prey_id, _)) = best else {
        wander(world, id, x, y, world.tick_count);
        world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Patrolling;
        return;
    };

    let (px, py) = world.entities.get(&prey_id).map(|p| (p.x, p.y)).unwrap_or((x, y));
    world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Hunting;

    if x == px && y == py {
        hunt_kill(world, id, prey_id, hunter_def);
    } else {
        move_toward(world, id, x, y, px, py);
    }
}

fn hunt_kill(world: &mut WorldState, hunter_id: EntityId, prey_id: EntityId, hunter_def: &CardDef) {
    let prey_type = world.entities.get(&prey_id).map(|p| p.type_name.clone());
    let Some(prey_type) = prey_type else {
        return;
    };
    let (prey_profile, hunter_profile) = {
        let prey = world.entities.get(&prey_id);
        let hunter = world.entities.get(&hunter_id);
        match (prey, hunter) {
            (Some(p), Some(h)) => (p.profile.clone(), h.profile.clone()),
            _ => return,
        }
    };
    let _kill = AxiomEngine::transform(&prey_profile, &hunter_profile, TransformAction::Kill);
    let corpse_type = corpse_type_for(&prey_type);
    let (px, py) = world.entities.get(&prey_id).map(|p| (p.x, p.y)).unwrap_or((0, 0));
    world.remove_entity(prey_id);
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
    crate::sim_observer::on_kill(world, &hunter_def.type_name, &prey_type, px, py);
    if let Some(corpse) = world.entities.get_mut(&corpse_id) {
        corpse.is_corpse = true;
        corpse.decay_timer = 0.0;
    }
    if hunter_def.type_name == "wolf" {
        if let Some(hunter) = world.entities.get_mut(&hunter_id) {
            hunter.carrying = Some(corpse_id);
            mark_ecology_fed(hunter, hunter_def);
        }
    } else if let Some(hunter) = world.entities.get_mut(&hunter_id) {
        mark_ecology_fed(hunter, hunter_def);
    }
    if let Some(hunter) = world.entities.get_mut(&hunter_id) {
        hunter.hunt_cooldown = 2.0;
    }
}

pub fn wolf_can_hunt_at_distance(dist: u8) -> bool {
    crate::world_rules::in_range(0, 0, dist, 0, HUNT_RANGE)
}
