use crate::axioms::{
    AxiomEngine, DriveBehavior, DriveDef, EntityProfile, SocialStructure, TransformAction,
};
use crate::card_def::CardDef;
use crate::ecology_log::eco_log;
use crate::game_constants::{
    FIELD_MOUSE_REPRODUCE_MIN_MICRO, FOX_SCAVENGE_PER_DAY, WOLF_DEN_DELIVERY_RANGE,
};
use crate::spatial_index::EntityId;
use crate::systems::movement::{flee_from, move_toward, wander};
use crate::world_rules::{
    card_has_tag, chebyshev_distance, corpse_type_for, hunt_target_score, is_hunt_target_for_pack,
    mark_ecology_fed, GRID_HEIGHT, GRID_WIDTH, HUNT_RANGE, HUNT_SCORE_INF,
};
use crate::world_state::{EcologyState, WorldState};

/// End-of-tick reactive pass for all autonomous entities.
pub fn mark_baseline_reactive_tick(world: &mut WorldState) {
    for entity in world.entities.values_mut() {
        if entity.is_corpse || entity.in_den || entity.in_burrow {
            continue;
        }
        if entity.is_autonomous(&world.card_defs) {
            entity.needs_grazing_tick = true;
        }
    }
}

pub fn flush_reactive_tick(world: &mut WorldState, delta: f32) {
    let mut ids: Vec<EntityId> = world
        .entities
        .iter()
        .filter(|(_, e)| e.needs_grazing_tick || e.needs_patrol)
        .map(|(id, _)| *id)
        .collect();
    ids.sort_by_key(|id| {
        world
            .entities
            .get(id)
            .map(|e| reactive_tick_priority(e.type_name.as_str()))
            .unwrap_or(1)
    });
    for id in ids {
        if let Some(e) = world.entities.get_mut(&id) {
            e.needs_grazing_tick = false;
            e.needs_patrol = false;
        }
        tick_reactive(world, id, delta);
    }
}

fn reactive_tick_priority(type_name: &str) -> u8 {
    match type_name {
        "fish" | "wolf" | "fox" => 0,
        "waterBug" => 2,
        _ => 1,
    }
}

fn drive_tie_rank(behavior: DriveBehavior) -> u8 {
    match behavior {
        DriveBehavior::Hide => 0,
        DriveBehavior::Flee => 1,
        DriveBehavior::Scavenge => 2,
        DriveBehavior::Seek => 3,
        DriveBehavior::ReturnDen => 4,
        DriveBehavior::Flock => 5,
        DriveBehavior::Wander => 6,
        DriveBehavior::Idle => 7,
    }
}

/// OnMove prey notify — defer hunt to end-of-tick reactive flush.
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
                entity.needs_grazing_tick = true;
            }
        }
    }
}

/// Unified reactive layer — all finite-intelligence entities tick through here.
pub fn tick_reactive(world: &mut WorldState, id: EntityId, delta: f32) {
    let Some(def) = world
        .entities
        .get(&id)
        .and_then(|e| world.card_defs.get(&e.type_name).cloned())
    else {
        return;
    };

    if card_has_tag(&def, "juvenile") {
        return;
    }

    let (x, y, profile, in_den, in_burrow) = {
        let e = &world.entities[&id];
        (e.x, e.y, e.profile.clone(), e.in_den, e.in_burrow)
    };

    if in_den || in_burrow {
        return;
    }

    if profile.herd_alert_range > 0 && profile.social_structure != SocialStructure::None {
        try_scatter_herd_on_predator(world, id, x, y, &profile);
        if !world.entities.contains_key(&id) {
            return;
        }
    }

    if !world.pool_cells.contains(&(x, y))
        && (def.type_name == "fish" || def.type_name == "waterBug")
    {
        return;
    }

    if card_has_tag(&def, "predator") || card_has_tag(&def, "mesopredator") {
        if flee_fire(world, id, x, y) {
            return;
        }
        if def.type_name == "wolf" && den_work_wolf(world, id, x, y, delta) {
            return;
        }
        if def.type_name == "fox" {
            if den_work_fox(world, id, x, y) {
                return;
            }
            if try_scavenge(world, id, x, y, &def) {
                return;
            }
        }
    }

    let drives = active_drives(world, id, x, y, &profile, &def);
    let mut sorted: Vec<_> = drives.iter().collect();
    sorted.sort_by(|a, b| {
        (-(a.priority as i32), drive_tie_rank(a.behavior)).cmp(&(-(b.priority as i32), drive_tie_rank(b.behavior)))
    });

    if let Some(drive) = sorted.first() {
        execute_drive(world, id, x, y, drive, &profile, &def, delta);
    } else if card_has_tag(&def, "predator") || card_has_tag(&def, "mesopredator") {
        wander(world, id, x, y, world.tick_count);
        if let Some(e) = world.entities.get_mut(&id) {
            e.ecology_state = EcologyState::Patrolling;
        }
    } else if let Some(e) = world.entities.get_mut(&id) {
        e.ecology_state = EcologyState::Idle;
    }

    if let Some(e) = world.entities.get_mut(&id) {
        if e.scatter_timer > 0 {
            e.scatter_timer -= 1;
        }
    }
}

struct ActiveDrive {
    behavior: DriveBehavior,
    target: Option<(EntityId, u8, u8)>,
    priority: u8,
    range: u8,
    hide_tag: String,
}

fn active_drives(
    world: &mut WorldState,
    id: EntityId,
    x: u8,
    y: u8,
    profile: &EntityProfile,
    def: &CardDef,
) -> Vec<ActiveDrive> {
    let mut drives = Vec::new();
    let Some(entity) = world.entities.get(&id) else {
        return Vec::new();
    };

    for drive_def in &profile.drives {
        if drive_def.condition_fed && entity.fed_today {
            continue;
        }

        match drive_def.behavior {
            DriveBehavior::Seek => {
                if let Some((tid, tx, ty)) =
                    best_seek_target(world, id, x, y, drive_def, def)
                {
                    drives.push(ActiveDrive {
                        behavior: DriveBehavior::Seek,
                        target: Some((tid, tx, ty)),
                        priority: drive_def.priority,
                        range: drive_def.range,
                        hide_tag: String::new(),
                    });
                }
            }
            DriveBehavior::Flee => {
                let threats = world.query_near_filtered(
                    x,
                    y,
                    &drive_def.target_tag,
                    drive_def.range,
                    id,
                );
                if let Some(&tid) = threats.first() {
                    if let Some(pos) = world.spatial_index.position(tid) {
                        drives.push(ActiveDrive {
                            behavior: DriveBehavior::Flee,
                            target: Some((tid, pos.0, pos.1)),
                            priority: drive_def.priority,
                            range: drive_def.range,
                            hide_tag: String::new(),
                        });
                    }
                }
            }
            DriveBehavior::Flock => {
                let mates = world.query_near_filtered(
                    x,
                    y,
                    &drive_def.target_tag,
                    drive_def.range,
                    id,
                );
                if mates.len() >= 2 {
                    let avg = average_position(world, &mates);
                    drives.push(ActiveDrive {
                        behavior: DriveBehavior::Flock,
                        target: Some((EntityId(0), avg.0, avg.1)),
                        priority: drive_def.priority,
                        range: drive_def.range,
                        hide_tag: String::new(),
                    });
                }
            }
            DriveBehavior::Hide => {
                let range = if drive_def.range > 0 {
                    drive_def.range
                } else {
                    4
                };
                let threats =
                    world.query_near_filtered(x, y, "predator", range, id);
                if !threats.is_empty()
                    && world.has_tag_at(x, y, &drive_def.target_tag)
                {
                    drives.push(ActiveDrive {
                        behavior: DriveBehavior::Hide,
                        target: None,
                        priority: drive_def.priority,
                        range,
                        hide_tag: drive_def.target_tag.clone(),
                    });
                }
            }
            DriveBehavior::ReturnDen => {
                if entity.fed_today {
                    if let Some(den_id) = entity.den_id {
                        if let Some(pos) = world.spatial_index.position(den_id) {
                            if chebyshev_distance(x, y, pos.0, pos.1) > 1 {
                                drives.push(ActiveDrive {
                                    behavior: DriveBehavior::ReturnDen,
                                    target: Some((den_id, pos.0, pos.1)),
                                    priority: drive_def.priority,
                                    range: 0,
                                    hide_tag: String::new(),
                                });
                            }
                        }
                    }
                }
            }
            DriveBehavior::Scavenge => {
                let corpses: Vec<EntityId> = world
                    .query_near_filtered(x, y, &drive_def.target_tag, drive_def.range, id)
                    .into_iter()
                    .filter(|cid| {
                        world
                            .entities
                            .get(cid)
                            .map(|c| c.type_name != "playerCorpse")
                            .unwrap_or(false)
                    })
                    .collect();
                if let Some(&cid) = corpses.first() {
                    if let Some(pos) = world.spatial_index.position(cid) {
                        drives.push(ActiveDrive {
                            behavior: DriveBehavior::Scavenge,
                            target: Some((cid, pos.0, pos.1)),
                            priority: drive_def.priority,
                            range: drive_def.range,
                            hide_tag: String::new(),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    drives
}

fn best_seek_target(
    world: &WorldState,
    id: EntityId,
    x: u8,
    y: u8,
    drive_def: &DriveDef,
    hunter_def: &CardDef,
) -> Option<(EntityId, u8, u8)> {
    let targets = world.query_near_filtered(
        x,
        y,
        &drive_def.target_tag,
        drive_def.range,
        id,
    );
    if targets.is_empty() {
        return None;
    }

    let can_hunt = card_has_tag(hunter_def, "predator") || card_has_tag(hunter_def, "mesopredator");
    let is_prey_seek = drive_def.target_tag == "herbivore" || drive_def.target_tag == "smallPrey";

    if can_hunt && is_prey_seek {
        let pack_size = if hunter_def.type_name == "wolf" {
            world.wolf_count()
        } else {
            1
        };
        let mut best: Option<(EntityId, f32)> = None;
        for prey_id in targets {
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
            if score < HUNT_SCORE_INF {
                if best.map(|(_, s)| score < s).unwrap_or(true) {
                    best = Some((prey_id, score));
                }
            }
        }
        return best.and_then(|(prey_id, _)| {
            world
                .spatial_index
                .position(prey_id)
                .map(|pos| (prey_id, pos.0, pos.1))
        });
    }

    targets.first().and_then(|&tid| {
        world
            .spatial_index
            .position(tid)
            .map(|pos| (tid, pos.0, pos.1))
    })
}

fn average_position(world: &WorldState, ids: &[EntityId]) -> (u8, u8) {
    let mut sx = 0u32;
    let mut sy = 0u32;
    let n = ids.len() as u32;
    for id in ids {
        if let Some((x, y)) = world.spatial_index.position(*id) {
            sx += x as u32;
            sy += y as u32;
        }
    }
    ((sx / n) as u8, (sy / n) as u8)
}

fn try_scatter_herd_on_predator(
    world: &mut WorldState,
    id: EntityId,
    x: u8,
    y: u8,
    profile: &EntityProfile,
) {
    let has_predator = !world
        .query_near_filtered(x, y, "predator", profile.herd_alert_range, id)
        .is_empty()
        || !world
            .query_near_filtered(x, y, "mesopredator", profile.herd_alert_range, id)
            .is_empty();
    if !has_predator {
        return;
    }
    let herd_count = world.entities.get(&id).map(|e| e.herd_count).unwrap_or(0);
    if herd_count >= 2 {
        scatter_herd(world, id);
    }
}

fn scatter_cells(world: &WorldState, x: u8, y: u8, count: u8) -> Vec<(u8, u8)> {
    let mut out = Vec::new();
    for r in 0..8u8 {
        for dx in -(r as i16)..=(r as i16) {
            for dy in -(r as i16)..=(r as i16) {
                if r > 0 && dx.unsigned_abs().max(dy.unsigned_abs()) != r as u16 {
                    continue;
                }
                let nx = x as i16 + dx;
                let ny = y as i16 + dy;
                if nx < 0
                    || ny < 0
                    || nx >= GRID_WIDTH as i16
                    || ny >= GRID_HEIGHT as i16
                {
                    continue;
                }
                let ux = nx as u8;
                let uy = ny as u8;
                if world.cell_composition.slot(ux, uy).living_count > 0 {
                    continue;
                }
                let terrain = crate::terrain::terrain_at(world, ux, uy);
                if matches!(terrain, "river" | "ford" | "barren" | "pool") {
                    continue;
                }
                out.push((ux, uy));
                if out.len() >= count as usize {
                    return out;
                }
            }
        }
    }
    out
}

fn scatter_herd(world: &mut WorldState, id: EntityId) {
    let Some(entity) = world.entities.get(&id).cloned() else {
        return;
    };
    if entity.herd_count < 2 {
        return;
    }
    let count = entity.herd_count;
    let type_name = entity.type_name.clone();
    let (x, y) = (entity.x, entity.y);
    world.remove_entity(id);

    let mut cells = scatter_cells(world, x, y, count);
    while cells.len() < count as usize {
        cells.push((x, y));
    }
    for i in 0..count {
        let (cx, cy) = cells[i as usize];
        let new_id = world.spawn(&type_name, cx, cy);
        if let Some(e) = world.entities.get_mut(&new_id) {
            e.scatter_timer = 5;
        }
    }
}

fn try_merge_herd(world: &mut WorldState, id: EntityId, x: u8, y: u8) -> bool {
    let Some(entity) = world.entities.get(&id).cloned() else {
        return false;
    };
    if entity.is_corpse || entity.profile.social_structure == SocialStructure::None {
        return false;
    }
    let type_name = entity.type_name.clone();
    let mut members: Vec<EntityId> = world
        .entities_at(x, y)
        .into_iter()
        .filter(|mid| {
            world.entities.get(mid).is_some_and(|e| {
                !e.is_corpse && e.type_name == type_name && e.herd_count <= 1
            })
        })
        .collect();
    members.sort_by_key(|mid| mid.0);
    if members.len() < 2 {
        return false;
    }
    let total = members.len() as u8;
    for mid in members {
        world.remove_entity(mid);
    }
    world.spawn_herd(&type_name, x, y, total);
    true
}

fn try_join_adjacent_herd(
    world: &mut WorldState,
    id: EntityId,
    x: u8,
    y: u8,
    profile: &EntityProfile,
) -> bool {
    let Some(entity) = world.entities.get(&id).cloned() else {
        return false;
    };
    if entity.herd_count > 1 || entity.is_corpse {
        return false;
    }
    let neighbors = world.query_near_filtered(x, y, &profile.type_name, 1, id);
    for nid in neighbors {
        let Some(herd) = world.entities.get(&nid).cloned() else {
            continue;
        };
        if herd.herd_count < 2 || herd.type_name != entity.type_name {
            continue;
        }
        if herd.herd_count >= profile.herd_max {
            continue;
        }
        world.remove_entity(id);
        if let Some(h) = world.entities.get_mut(&nid) {
            h.herd_count += 1;
        }
        return true;
    }
    false
}

fn execute_herd_drive(
    world: &mut WorldState,
    id: EntityId,
    x: u8,
    y: u8,
    profile: &EntityProfile,
    drive: &ActiveDrive,
) {
    if try_merge_herd(world, id, x, y) {
        return;
    }
    if try_join_adjacent_herd(world, id, x, y, profile) {
        return;
    }

    if world
        .entities
        .get(&id)
        .is_some_and(|e| e.scatter_timer > 0)
    {
        world.next_move_speed = Some(profile.move_speed);
        wander(world, id, x, y, world.tick_count);
        return;
    }

    if let Some((_, tx, ty)) = drive.target {
        world.next_move_speed = Some(profile.move_speed);
        move_toward(world, id, x, y, tx, ty);
    }
}

fn execute_drive(
    world: &mut WorldState,
    id: EntityId,
    x: u8,
    y: u8,
    drive: &ActiveDrive,
    _profile: &EntityProfile,
    def: &CardDef,
    _delta: f32,
) {
    match drive.behavior {
        DriveBehavior::Flock => {
            if _profile.social_structure != SocialStructure::None && _profile.herd_range > 0 {
                execute_herd_drive(world, id, x, y, _profile, drive);
            } else if let Some((target_id, tx, ty)) = drive.target {
                let dist = chebyshev_distance(x, y, tx, ty);
                if dist <= 1 && target_id != EntityId(0) {
                    try_interact_at(world, id, target_id, def);
                } else {
                    move_toward(world, id, x, y, tx, ty);
                }
            }
            if let Some(e) = world.entities.get_mut(&id) {
                e.ecology_state = EcologyState::SeekingFood;
            }
        }
        DriveBehavior::Seek | DriveBehavior::ReturnDen => {
            if let Some((target_id, tx, ty)) = drive.target {
                let dist = chebyshev_distance(x, y, tx, ty);
                if drive.behavior == DriveBehavior::Seek && dist <= 1 {
                    try_interact_at(world, id, target_id, def);
                } else if drive.behavior == DriveBehavior::ReturnDen && x == tx && y == ty {
                    // arrived at den
                } else {
                    if drive.behavior == DriveBehavior::Seek
                        && (card_has_tag(def, "predator") || card_has_tag(def, "mesopredator"))
                    {
                        if world
                            .entities
                            .get(&id)
                            .is_some_and(|e| e.hunt_cooldown > 0.0)
                        {
                            wander(world, id, x, y, world.tick_count);
                            if let Some(e) = world.entities.get_mut(&id) {
                                e.ecology_state = EcologyState::Patrolling;
                            }
                            return;
                        }
                        let _ =
                            crate::rule_index::dual_track_hunt(crate::rule_index::rule_index(), world, id);
                        let _ = crate::rule_index::dual_track_stalk(
                            crate::rule_index::rule_index(),
                            world,
                            id,
                        );
                        if let Some(e) = world.entities.get_mut(&id) {
                            e.ecology_state = EcologyState::Hunting;
                        }
                    }
                    world.next_move_speed = Some(_profile.sprint_speed);
                    move_toward(world, id, x, y, tx, ty);
                    if let Some(e) = world.entities.get_mut(&id) {
                        if e.ecology_state != EcologyState::Hunting {
                            e.ecology_state = EcologyState::SeekingFood;
                        }
                    }
                }
            }
        }
        DriveBehavior::Flee => {
            if let Some((_, tx, ty)) = drive.target {
                world.next_move_speed = Some(_profile.sprint_speed);
                flee_from(world, id, x, y, tx, ty);
                if let Some(e) = world.entities.get_mut(&id) {
                    e.ecology_state = EcologyState::Fleeing;
                }
            }
        }
        DriveBehavior::Hide => {
            if let Some(e) = world.entities.get_mut(&id) {
                if drive.hide_tag == "grass" {
                    e.hidden_in_grass = true;
                    e.ecology_state = EcologyState::Fleeing;
                } else {
                    e.in_burrow = true;
                    e.ecology_state = EcologyState::Burrowed;
                }
            }
        }
        DriveBehavior::Scavenge => {
            if let Some((corpse_id, tx, ty)) = drive.target {
                if chebyshev_distance(x, y, tx, ty) <= 1 {
                    world.remove_entity(corpse_id);
                    let today = world
                        .entities
                        .get(&id)
                        .map(|f| f.scavenge_today + 1)
                        .unwrap_or(1);
                    if let Some(fox) = world.entities.get_mut(&id) {
                        mark_ecology_fed(fox, def);
                        fox.scavenge_today = today;
                        fox.ecology_state = EcologyState::Scavenging;
                    }
                    eco_log(
                        world,
                        format!(
                            "狐狸从尸体取肉回窝（今日 {today}/{FOX_SCAVENGE_PER_DAY}）"
                        ),
                    );
                } else {
                    move_toward(world, id, x, y, tx, ty);
                }
            }
        }
        DriveBehavior::Wander => {
            wander(world, id, x, y, world.tick_count);
            if let Some(e) = world.entities.get_mut(&id) {
                e.ecology_state = EcologyState::Wandering;
            }
        }
        DriveBehavior::Idle => {
            if let Some(e) = world.entities.get_mut(&id) {
                e.ecology_state = EcologyState::Idle;
            }
        }
    }
}

fn try_interact_at(world: &mut WorldState, actor_id: EntityId, target_id: EntityId, def: &CardDef) {
    let target_type = world
        .entities
        .get(&target_id)
        .map(|e| e.type_name.clone());
    let Some(target_type) = target_type else {
        return;
    };

    if target_type.ends_with("Corpse") || world.entities.get(&target_id).is_some_and(|e| e.is_corpse)
    {
        return;
    }

    let actor_type = def.type_name.clone();

    if (card_has_tag(def, "predator") || card_has_tag(def, "mesopredator"))
        && world.entities.get(&target_id).is_some_and(|t| {
            !t.is_corpse
                && world
                    .card_defs
                    .get(&t.type_name)
                    .map(|d| {
                        card_has_tag(d, "herbivore") || card_has_tag(d, "smallPrey")
                    })
                    .unwrap_or(false)
        })
    {
        hunt_kill(world, actor_id, target_id, def);
        return;
    }

    if actor_type == "fieldMouse" || actor_type == "fieldMousePup" {
        let (x, y) = {
            let e = &world.entities[&actor_id];
            (e.x, e.y)
        };
        let micro = world.bush_microfauna.get(&(x, y)).copied().unwrap_or(0);
        if world.has_tag_at(x, y, "bush") && micro >= FIELD_MOUSE_REPRODUCE_MIN_MICRO {
            if let Some(mouse) = world.entities.get_mut(&actor_id) {
                mark_ecology_fed(mouse, def);
                mouse.ecology_state = EcologyState::SeekingFood;
            }
            world
                .bush_microfauna
                .entry((x, y))
                .and_modify(|m| *m = (*m - 1).max(0));
            return;
        }
    }

    if actor_type == "bambooRat" && world.has_tag_at(
        world.entities[&actor_id].x,
        world.entities[&actor_id].y,
        "underground",
    ) {
        if let Some(rat) = world.entities.get_mut(&actor_id) {
            mark_ecology_fed(rat, def);
            rat.ecology_state = EcologyState::SeekingFood;
        }
        return;
    }

    let _ = crate::rule_index::dual_track_graze(crate::rule_index::rule_index(), world, actor_id);
    let _ = crate::rule_index::dual_track_eat(crate::rule_index::rule_index(), world, actor_id);
    try_consume(world, actor_id, target_id, def);
}

fn try_consume(
    world: &mut WorldState,
    eater_id: EntityId,
    target_id: EntityId,
    def: &CardDef,
) {
    let consumed = world
        .entities
        .get(&target_id)
        .map(|g| g.consumed)
        .unwrap_or(true);
    if consumed {
        return;
    }
    let (src_profile, eater_profile, ex, ey, tx, ty) = {
        let grass = world.entities.get(&target_id);
        let eater = world.entities.get(&eater_id);
        match (grass, eater) {
            (Some(g), Some(e)) => (
                g.profile.clone(),
                e.profile.clone(),
                e.x,
                e.y,
                g.x,
                g.y,
            ),
            _ => return,
        }
    };
    if chebyshev_distance(ex, ey, tx, ty) > 1 {
        return;
    }
    let result = AxiomEngine::transform(&src_profile, &eater_profile, TransformAction::Eat);
    if let Some(target) = world.entities.get_mut(&target_id) {
        target.consumed = true;
        target.hp = 0;
    }
    world.remove_entity(target_id);
    if let Some(eater) = world.entities.get_mut(&eater_id) {
        mark_ecology_fed(eater, def);
        eater.profile.energy = eater.profile.energy.saturating_add(result.energy_received);
        eater.ecology_state = EcologyState::Idle;
    }
}

fn hunt_kill(
    world: &mut WorldState,
    hunter_id: EntityId,
    prey_id: EntityId,
    hunter_def: &CardDef,
) {
    let prey_type = world.entities.get(&prey_id).map(|p| p.type_name.clone());
    let Some(prey_type) = prey_type else {
        return;
    };
    let (prey_profile, hunter_profile, herd_count, hx, hy, px, py) = {
        let prey = world.entities.get(&prey_id);
        let hunter = world.entities.get(&hunter_id);
        match (prey, hunter) {
            (Some(p), Some(h)) => (
                p.profile.clone(),
                h.profile.clone(),
                p.herd_count,
                h.x,
                h.y,
                p.x,
                p.y,
            ),
            _ => return,
        }
    };
    if chebyshev_distance(hx, hy, px, py) > 1 {
        return;
    }
    let corpse_type = corpse_type_for(&prey_type);
    if herd_count > 1 {
        let mut single_profile = prey_profile.clone();
        single_profile.energy = (prey_profile.energy / herd_count as u32).max(1);
        let _kill =
            AxiomEngine::transform(&single_profile, &hunter_profile, TransformAction::Kill);
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
        if let Some(prey) = world.entities.get_mut(&prey_id) {
            prey.herd_count -= 1;
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
        return;
    }
    let _kill = AxiomEngine::transform(&prey_profile, &hunter_profile, TransformAction::Kill);
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
    if scavenge_cap >= FOX_SCAVENGE_PER_DAY {
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
    if chebyshev_distance(x, y, cx, cy) <= 1 {
        world.remove_entity(*corpse_id);
        let today = world.entities.get(&id).map(|f| f.scavenge_today + 1).unwrap_or(1);
        if let Some(fox) = world.entities.get_mut(&id) {
            mark_ecology_fed(fox, def);
            fox.scavenge_today = today;
            fox.ecology_state = EcologyState::Scavenging;
        }
        eco_log(
            world,
            format!("狐狸从尸体取肉回窝（今日 {today}/{FOX_SCAVENGE_PER_DAY}）"),
        );
        return true;
    }
    move_toward(world, id, x, y, cx, cy);
    true
}
