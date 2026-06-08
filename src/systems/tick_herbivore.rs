use crate::axioms::{AxiomEngine, TransformAction};
use crate::card_def::CardDef;
use crate::game_constants::{RABBIT_WOLF_FEAR_DIST, SHEEP_FEAR_RANGE, WILDPREY_FEAR_RANGE};
use crate::spatial_index::EntityId;
use crate::systems::movement::{flee_from, move_toward, nearest_of, wander};
use crate::world_rules::{
    ecology_was_fed_today, herbivore_grazer_profile, mark_ecology_fed, wolves_near, GrazerProfile,
};
use crate::world_state::{EcologyState, WorldState};

pub fn tick_herbivores(world: &mut WorldState) {
    let ids: Vec<EntityId> = world
        .entities
        .values()
        .filter(|e| {
            world
                .card_defs
                .get(&e.type_name)
                .map(|d| {
                    crate::world_rules::can_forage(d)
                        || crate::world_rules::card_has_tag(d, "herbivore")
                })
                .unwrap_or(false)
        })
        .filter(|e| !e.is_corpse)
        .map(|e| e.id)
        .collect();
    for id in ids {
        if let Some(def) = world.card_defs.get(&world.entities[&id].type_name).cloned() {
            tick_one_grazer(world, id, &def);
        }
    }
}

pub fn tick_one_grazer(world: &mut WorldState, id: EntityId, def: &CardDef) {
    match herbivore_grazer_profile(def) {
        GrazerProfile::Sheep => tick_sheep(world, id, def),
        GrazerProfile::Deer => tick_deer(world, id, def),
        GrazerProfile::Rabbit => tick_rabbit(world, id, def, true),
        GrazerProfile::Pheasant => tick_rabbit(world, id, def, false),
        GrazerProfile::Slow => tick_slow(world, id, def),
        GrazerProfile::Juvenile => tick_juvenile(world, id),
    }
}

fn tick_sheep(world: &mut WorldState, id: EntityId, def: &CardDef) {
    let (x, y) = {
        let e = &world.entities[&id];
        (e.x, e.y)
    };
    if ecology_was_fed_today(&world.entities[&id], def) {
        world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Idle;
        return;
    }
    let wolves = wolves_near(world, x, y, SHEEP_FEAR_RANGE);
    if let Some((wid, _)) = nearest_of(world, x, y, &wolves) {
        let (wx, wy) = world.spatial_index.position(wid).unwrap_or((x, y));
        flee_from(world, id, x, y, wx, wy);
        if let Some(e) = world.entities.get_mut(&id) {
            e.ecology_state = EcologyState::Fleeing;
        }
        return;
    }
    try_eat_grass(world, id, x, y, def);
}

fn tick_deer(world: &mut WorldState, id: EntityId, def: &CardDef) {
    let (x, y) = {
        let e = &world.entities[&id];
        (e.x, e.y)
    };
    if ecology_was_fed_today(&world.entities[&id], def) {
        world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Idle;
        return;
    }
    let wolves = wolves_near(world, x, y, WILDPREY_FEAR_RANGE);
    if let Some((wid, _)) = nearest_of(world, x, y, &wolves) {
        let (wx, wy) = world.spatial_index.position(wid).unwrap_or((x, y));
        flee_from(world, id, x, y, wx, wy);
        if let Some(e) = world.entities.get_mut(&id) {
            e.ecology_state = EcologyState::Fleeing;
        }
        return;
    }
    try_eat_grass(world, id, x, y, def);
}

fn tick_rabbit(world: &mut WorldState, id: EntityId, def: &CardDef, may_hide: bool) {
    let (x, y) = {
        let e = &world.entities[&id];
        (e.x, e.y)
    };
    if ecology_was_fed_today(&world.entities[&id], def) {
        world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Idle;
        return;
    }
    let wolves = wolves_near(world, x, y, RABBIT_WOLF_FEAR_DIST);
    if !wolves.is_empty() {
        if may_hide && world.has_tag_at(x, y, "grass") {
            world.entities.get_mut(&id).unwrap().hidden_in_grass = true;
            world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Fleeing;
            return;
        }
        if let Some((wid, _)) = nearest_of(world, x, y, &wolves) {
            let (wx, wy) = world.spatial_index.position(wid).unwrap_or((x, y));
            flee_from(world, id, x, y, wx, wy);
            if let Some(e) = world.entities.get_mut(&id) {
                e.ecology_state = EcologyState::Fleeing;
            }
            return;
        }
    }
    world.entities.get_mut(&id).unwrap().hidden_in_grass = false;
    try_eat_grass(world, id, x, y, def);
}

fn tick_slow(world: &mut WorldState, id: EntityId, def: &CardDef) {
    let (x, y) = (world.entities[&id].x, world.entities[&id].y);
    if ecology_was_fed_today(&world.entities[&id], def) {
        world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Idle;
        return;
    }
    try_eat_grass(world, id, x, y, def);
}

fn tick_juvenile(world: &mut WorldState, id: EntityId) {
    let tick = world.tick_count;
    let (x, y) = (world.entities[&id].x, world.entities[&id].y);
    wander(world, id, x, y, tick);
    world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Wandering;
}

fn try_eat_grass(world: &mut WorldState, id: EntityId, x: u8, y: u8, def: &CardDef) {
    let _ = crate::rule_index::dual_track_graze(crate::rule_index::rule_index(), world, id);
    let _ = crate::rule_index::dual_track_eat(crate::rule_index::rule_index(), world, id);
    let grass_near = world.query_near_filtered(x, y, "foodSource", 6, id);
    if grass_near.is_empty() {
        world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Idle;
        return;
    }
    let grass_id = grass_near[0];
    let (gx, gy) = world.spatial_index.position(grass_id).unwrap_or((x, y));
    if x == gx && y == gy {
        eat_grass_at(world, id, grass_id, def);
    } else {
        move_toward(world, id, x, y, gx, gy);
        world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::SeekingFood;
    }
}

fn eat_grass_at(world: &mut WorldState, eater_id: EntityId, grass_id: EntityId, def: &CardDef) {
    let consumed = world.entities.get(&grass_id).map(|g| g.consumed).unwrap_or(true);
    if consumed {
        return;
    }
    let (src_profile, eater_profile) = {
        let grass = world.entities.get(&grass_id);
        let eater = world.entities.get(&eater_id);
        match (grass, eater) {
            (Some(g), Some(e)) => (g.profile.clone(), e.profile.clone()),
            _ => return,
        }
    };
    let result = AxiomEngine::transform(&src_profile, &eater_profile, TransformAction::Eat);
    if let Some(grass) = world.entities.get_mut(&grass_id) {
        grass.consumed = true;
        grass.hp = 0;
    }
    world.remove_entity(grass_id);
    if let Some(eater) = world.entities.get_mut(&eater_id) {
        mark_ecology_fed(eater, def);
        eater.profile.energy = eater.profile.energy.saturating_add(result.energy_received);
        eater.ecology_state = EcologyState::Idle;
    }
}
