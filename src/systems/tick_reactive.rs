use crate::axioms::{
    profile::{score_need_for_drive, NeedState},
    AxiomEngine, DriveBehavior, DriveDef, EntityProfile, TransformAction,
};
use crate::bulletin::seek_target_channel;
use crate::card_def::CardDef;
use crate::ecology_log::eco_log;
use crate::game_constants::{
    FIELD_MOUSE_REPRODUCE_MIN_MICRO, FOX_SCAVENGE_PER_DAY, WOLF_DEN_DELIVERY_RANGE,
};
use crate::spatial_index::EntityId;
use crate::systems::movement::{flee_from, move_toward, wander};
use crate::world_rules::{
    card_has_capability, card_has_tag, chebyshev_distance, ecology_was_fed_today,
    hunt_target_score, is_hunt_target_for_pack, mark_ecology_fed, parse_max_starve,
    HUNT_RANGE,
    HUNT_SCORE_INF, GRID_HEIGHT, GRID_WIDTH,
};
use crate::world_state::{EcologyState, WorldState};

pub const SLEEP_DURATION_TICKS: u64 = 5;
pub const WAKE_ACTIVITY_RANGE: u8 = 6;
const SLEEP_HUNGER_RATIO: f32 = 0.3;

pub fn wake_autonomous_near(world: &mut WorldState, x: u8, y: u8, range: u8) {
    let tick = world.tick_count;
    let min_x = x.saturating_sub(range);
    let max_x = (x as u16 + range as u16).min(GRID_WIDTH as u16 - 1) as u8;
    let min_y = y.saturating_sub(range);
    let max_y = (y as u16 + range as u16).min(GRID_HEIGHT as u16 - 1) as u8;
    for gy in min_y..=max_y {
        for gx in min_x..=max_x {
            for id in world.entities_at(gx, gy) {
                let Some(entity) = world.entities.get_mut(&id) else {
                    continue;
                };
                if entity.is_sleeping(tick)
                    && entity.is_autonomous(&world.card_defs)
                {
                    entity.wake();
                }
            }
        }
    }
}

/// End-of-tick reactive pass for all autonomous entities.
pub fn mark_baseline_reactive_tick(world: &mut WorldState) {
    let tick = world.tick_count;
    for entity in world.entities.values_mut() {
        if entity.is_corpse || entity.in_den || entity.in_burrow {
            continue;
        }
        if entity.is_sleeping(tick) {
            continue;
        }
        if entity.is_autonomous(&world.card_defs) {
            entity.needs_grazing_tick = true;
        }
    }
}

pub fn flush_reactive_tick(world: &mut WorldState, delta: f32) {
    let tick = world.tick_count;
    let mut ids: Vec<EntityId> = world
        .entities
        .iter()
        .filter(|(_, e)| {
            (e.needs_grazing_tick || e.needs_patrol) && !e.is_sleeping(tick)
        })
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

fn utility_drive_key(drive: &ActiveDrive) -> (DriveBehavior, String) {
    let tag = if drive.behavior == DriveBehavior::Hide && !drive.hide_tag.is_empty() {
        drive.hide_tag.clone()
    } else {
        drive.target_tag.clone()
    };
    (drive.behavior, tag)
}

fn nearest_threat_distance(world: &WorldState, x: u8, y: u8, id: EntityId) -> Option<u8> {
    let mut best: Option<u8> = None;
    for tag in ["predator", "mesopredator"] {
        for tid in world.query_near_filtered(x, y, tag, 6, id) {
            if let Some((tx, ty)) = world.spatial_index.position(tid) {
                let d = chebyshev_distance(x, y, tx, ty);
                best = Some(best.map(|b| b.min(d)).unwrap_or(d));
            }
        }
    }
    best
}

fn tick_entity_needs(world: &mut WorldState, id: EntityId, def: &CardDef) {
    let (x, y, starve_days, hungry) = {
        let Some(e) = world.entities.get(&id) else {
            return;
        };
        (
            e.x,
            e.y,
            e.starve_days,
            !ecology_was_fed_today(e, def),
        )
    };
    let threat_dist = nearest_threat_distance(world, x, y, id);
    let max_starve = parse_max_starve(def);
    let hunger_ratio = if max_starve > 0 {
        starve_days as f32 / max_starve as f32
    } else {
        0.0
    };

    if let Some(entity) = world.entities.get_mut(&id) {
        for need in &mut entity.profile.needs {
            need.current = (need.current + need.decay_rate).min(100.0);
        }
        if hungry {
            if let Some(need) = entity.profile.needs.iter_mut().find(|n| n.kind == "eat") {
                need.current = need.current.max(50.0 + hunger_ratio * 50.0);
            }
        } else if let Some(need) = entity.profile.needs.iter_mut().find(|n| n.kind == "eat") {
            need.current = need.current.min(15.0);
        }
        if let Some(dist) = threat_dist {
            if let Some(need) = entity.profile.needs.iter_mut().find(|n| n.kind == "safety") {
                need.current =
                    (need.current + (6.0_f32 - dist as f32).max(0.0) * 3.0).min(100.0);
                if dist <= 4 {
                    need.current = need.current.max(85.0 - dist as f32 * 10.0);
                }
            }
        }
        let _ = (x, y);
    }
}

fn select_utility_drive<'a>(
    drives: &'a [ActiveDrive],
    needs: &[NeedState],
    x: u8,
    y: u8,
    last: Option<&(DriveBehavior, String)>,
) -> Option<&'a ActiveDrive> {
    if drives.is_empty() {
        return None;
    }
    let scored: Vec<(&ActiveDrive, f32)> = drives
        .iter()
        .map(|d| {
            let dist = d
                .target
                .map(|(_, tx, ty)| chebyshev_distance(x, y, tx, ty))
                .unwrap_or(0);
            let score = needs
                .iter()
                .find(|n| n.kind == d.need_kind)
                .map(|n| score_need_for_drive(n, d.range, dist))
                .unwrap_or(0.0);
            (d, score)
        })
        .collect();
    let (best_drive, best_score) = scored
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))?;
    if best_score <= &0.0 {
        return None;
    }
    if let Some(last_key) = last {
        if let Some((last_drive, last_score)) = scored
            .iter()
            .find(|(d, _)| utility_drive_key(d) == *last_key)
        {
            let margin = 0.1 * best_score.max(0.001);
            if best_score - last_score < margin {
                return Some(last_drive);
            }
        }
    }
    Some(best_drive)
}

/// OnMove prey notify — defer hunt to end-of-tick reactive flush.
pub fn mark_predators_near_prey_needs_patrol(
    world: &mut WorldState,
    prey_id: EntityId,
    x: u8,
    y: u8,
) {
    let mut predators: Vec<EntityId> = world
        .query_near_filtered(x, y, "predator", HUNT_RANGE, prey_id)
        .into_iter()
        .chain(world.query_near_filtered(x, y, "mesopredator", HUNT_RANGE, prey_id))
        .collect();
    predators.sort_unstable_by_key(|id| id.0);
    predators.dedup();
    for pid in predators {
        if let Some(entity) = world.entities.get_mut(&pid) {
            if !entity.is_corpse && !entity.in_den {
                entity.wake();
            }
        }
    }
}

/// Unified reactive layer — all finite-intelligence entities tick through here.
pub fn tick_reactive(world: &mut WorldState, id: EntityId, delta: f32) {
    let Some(e) = world.entities.get(&id) else {
        return;
    };
    let Some(def) = world.card_defs.get(&e.type_name).cloned() else {
        return;
    };

    if card_has_tag(&def, "juvenile") {
        return;
    }

    if e.is_sleeping(world.tick_count) {
        return;
    }

    tick_entity_needs(world, id, &def);

    let (x, y, profile, in_den, in_burrow, last_drive) = {
        let e = world.entities.get(&id).expect("entity");
        (
            e.x,
            e.y,
            e.profile.clone(),
            e.in_den,
            e.in_burrow,
            e.last_utility_drive.clone(),
        )
    };

    if in_den || in_burrow {
        return;
    }

    if world.entities.get(&id).is_some_and(|e| e.in_cover) {
        let threat_near = !world
            .query_near_filtered(x, y, "predator", 5, id)
            .is_empty()
            || !world
                .query_near_filtered(x, y, "mesopredator", 5, id)
                .is_empty();
        let hungry = world.entities.get(&id).is_some_and(|e| {
            world
                .card_defs
                .get(&e.type_name)
                .is_some_and(|d| !ecology_was_fed_today(e, d) || e.starve_days > 0)
        });
        if !threat_near && hungry {
            exit_cover(world, id);
        } else {
            if let Some(e) = world.entities.get_mut(&id) {
                e.ecology_state = if threat_near {
                    EcologyState::Fleeing
                } else {
                    EcologyState::Idle
                };
            }
            return;
        }
    }

    if profile.native_medium == "water" && !world.pool_cells.contains(&(x, y)) {
        return;
    }

    if card_has_tag(&def, "predator") || card_has_tag(&def, "mesopredator") {
        if flee_fire(world, id, x, y) {
            return;
        }
        if card_has_tag(&def, "pack_hunter") && card_has_capability(&def, "capability.return_home") {
            if den_work_wolf(world, id, x, y, delta) {
                return;
            }
        }
        if card_has_tag(&def, "mesopredator") && card_has_tag(&def, "scavenger") {
            if den_work_fox(world, id, x, y) {
                return;
            }
            if try_scavenge(world, id, x, y, &def) {
                return;
            }
        }
    }

    let drives = active_drives(world, id, x, y, &profile, &def);
    let chosen = select_utility_drive(&drives, &profile.needs, x, y, last_drive.as_ref());

    let executed_drive = if let Some(drive) = chosen {
        execute_drive(world, id, x, y, drive, &profile, &def, delta);
        if let Some(e) = world.entities.get_mut(&id) {
            e.last_utility_drive = Some(utility_drive_key(drive));
        }
        true
    } else if card_has_tag(&def, "predator") || card_has_tag(&def, "mesopredator") {
        wander(world, id, x, y, world.tick_count);
        if let Some(e) = world.entities.get_mut(&id) {
            e.ecology_state = EcologyState::Patrolling;
        }
        true
    } else {
        if let Some(e) = world.entities.get_mut(&id) {
            e.ecology_state = EcologyState::Idle;
        }
        false
    };

    // Idle cover-user: auto-hide if on grass/bush when no other drive active
    if let Some(e) = world.entities.get(&id) {
        if e.ecology_state == EcologyState::Idle
            && !e.in_cover
            && crate::world_rules::card_has_tag(&def, "cover_user")
        {
            try_auto_hide_after_flee(world, id, &profile);
        }
    }

    if !executed_drive {
        try_enter_sleep(world, id, &def);
    }
}

fn try_enter_sleep(world: &mut WorldState, id: EntityId, def: &CardDef) {
    let tick = world.tick_count;
    let Some(entity) = world.entities.get(&id) else {
        return;
    };
    if !entity.is_autonomous(&world.card_defs) {
        return;
    }
    if !matches!(
        entity.ecology_state,
        EcologyState::Idle | EcologyState::Wandering
    ) {
        return;
    }
    if entity.in_cover || entity.in_den || entity.in_burrow {
        return;
    }
    let max_starve = parse_max_starve(def);
    if max_starve <= 0 {
        return;
    }
    let hunger_ratio = entity.starve_days as f32 / max_starve as f32;
    if hunger_ratio >= SLEEP_HUNGER_RATIO {
        return;
    }
    if let Some(entity) = world.entities.get_mut(&id) {
        entity.sleep_until_tick = tick + SLEEP_DURATION_TICKS;
    }
}

struct ActiveDrive {
    behavior: DriveBehavior,
    target_tag: String,
    need_kind: String,
    target: Option<(EntityId, u8, u8)>,
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
                        target_tag: drive_def.target_tag.clone(),
                        need_kind: drive_def.need_kind.clone(),
                        target: Some((tid, tx, ty)),
                        range: drive_def.range,
                        hide_tag: String::new(),
                    });
                } else if let Some((zx, zy)) = bulletin_seek_fallback(
                    world,
                    profile,
                    x,
                    y,
                    &drive_def.target_tag,
                ) {
                    drives.push(ActiveDrive {
                        behavior: DriveBehavior::Seek,
                        target_tag: drive_def.target_tag.clone(),
                        need_kind: drive_def.need_kind.clone(),
                        target: Some((EntityId(0), zx, zy)),
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
                            target_tag: drive_def.target_tag.clone(),
                            need_kind: drive_def.need_kind.clone(),
                            target: Some((tid, pos.0, pos.1)),
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
                        target_tag: drive_def.target_tag.clone(),
                        need_kind: drive_def.need_kind.clone(),
                        target: Some((EntityId(0), avg.0, avg.1)),
                        range: drive_def.range,
                        hide_tag: String::new(),
                    });
                }
            }
            DriveBehavior::Hide => {
                if entity.in_cover {
                    drives.push(ActiveDrive {
                        behavior: DriveBehavior::Idle,
                        target_tag: drive_def.target_tag.clone(),
                        need_kind: drive_def.need_kind.clone(),
                        target: None,
                        range: 0,
                        hide_tag: String::new(),
                    });
                    continue;
                }
                if !world.has_tag_at(x, y, &drive_def.target_tag) {
                    continue;
                }
                let range = if drive_def.range > 0 {
                    drive_def.range
                } else {
                    4
                };
                let predator_near = !world
                    .query_near_filtered(x, y, "predator", range, id)
                    .is_empty();
                let fleeing = entity.ecology_state == EcologyState::Fleeing;
                if fleeing || predator_near {
                    drives.push(ActiveDrive {
                        behavior: DriveBehavior::Hide,
                        target_tag: drive_def.target_tag.clone(),
                        need_kind: drive_def.need_kind.clone(),
                        target: None,
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
                                    target_tag: drive_def.target_tag.clone(),
                                    need_kind: drive_def.need_kind.clone(),
                                    target: Some((den_id, pos.0, pos.1)),
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
                            target_tag: drive_def.target_tag.clone(),
                            need_kind: drive_def.need_kind.clone(),
                            target: Some((cid, pos.0, pos.1)),
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

fn bulletin_seek_fallback(
    world: &WorldState,
    profile: &EntityProfile,
    x: u8,
    y: u8,
    target_tag: &str,
) -> Option<(u8, u8)> {
    let channel = seek_target_channel(target_tag);
    let (zx, zy) = world.bulletin_board.nearest_zone_center(profile, channel, x, y)?;
    if chebyshev_distance(x, y, zx, zy) <= 1 {
        return None;
    }
    Some((zx, zy))
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
        let pack_size = if card_has_tag(hunter_def, "pack_hunter") {
            world.count_by_tag("pack_hunter").max(1)
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
            if prey.in_cover && chebyshev_distance(x, y, prey.x, prey.y) > 1 {
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
            if let Some((_, tx, ty)) = drive.target {
                world.next_move_speed = Some(_profile.move_speed);
                move_toward(world, id, x, y, tx, ty);
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
                if let Some((hx, hy)) =
                    crate::systems::movement::hunting_predator_adjacent(world, x, y, id)
                {
                    crate::systems::movement::flee_pathfind(world, id, x, y, hx, hy);
                } else {
                    flee_from(world, id, x, y, tx, ty);
                }
                if let Some(e) = world.entities.get_mut(&id) {
                    e.ecology_state = EcologyState::Fleeing;
                }
                try_auto_hide_after_flee(world, id, _profile);
            }
        }
        DriveBehavior::Hide => {
            try_hide_in_cover_at(world, id, &drive.hide_tag);
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

    let prey_def = world
        .entities
        .get(&target_id)
        .and_then(|t| world.card_defs.get(&t.type_name));
    let is_prey = prey_def.is_some_and(|d| {
        card_has_tag(d, "herbivore") || card_has_tag(d, "smallPrey")
    });
    let can_hunt_prey = card_has_tag(def, "predator") || card_has_tag(def, "mesopredator");
    let can_forage_prey = card_has_tag(def, "aquatic")
        && card_has_capability(def, "capability.forage")
        && prey_def.is_some_and(|d| card_has_tag(d, "smallPrey"));
    if (can_hunt_prey || can_forage_prey)
        && world
            .entities
            .get(&target_id)
            .is_some_and(|t| !t.is_corpse && is_prey)
    {
        crate::interaction::apply_hunt_smash(world, actor_id, target_id, def);
        return;
    }

    if card_has_tag(def, "forages:bush") {
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

    if card_has_tag(def, "forages:underground") && world.has_tag_at(
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
    let target_type = world
        .entities
        .get(&target_id)
        .map(|e| e.type_name.clone())
        .unwrap_or_default();
    if target_type == "grass" {
        let remove = {
            if let Some(target) = world.entities.get_mut(&target_id) {
                target.hp = (target.hp - 1).max(0);
                target.hp <= 0
            } else {
                false
            }
        };
        if remove {
            world.remove_entity(target_id);
        }
        if let Some(eater) = world.entities.get_mut(&eater_id) {
            mark_ecology_fed(eater, def);
            eater.ecology_state = EcologyState::Idle;
        }
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
        if world.count_by_tag("den.wolf") == 0 {
            world.spawn("wolfDen", x, y);
            eco_log(world, "狼在远离草棚和火源的边缘形成狼穴");
            if let Some(den) = world
                .entities
                .values()
                .find(|e| {
                    e.x == x
                        && e.y == y
                        && world
                            .card_defs
                            .get(&e.type_name)
                            .is_some_and(|d| card_has_tag(d, "den.wolf"))
                })
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
        .find(|e| {
            chebyshev_distance(x, y, e.x, e.y) <= 1
                && world
                    .card_defs
                    .get(&e.type_name)
                    .is_some_and(|d| card_has_tag(d, "bush"))
        })
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

/// Leave cover and reclaim the grid cell (same as in_tree vacate/occupy pattern).
pub fn exit_cover(world: &mut WorldState, id: EntityId) {
    let Some(entity) = world.entities.get(&id).cloned() else {
        return;
    };
    if !entity.in_cover {
        return;
    }
    let (x, y) = (entity.x, entity.y);
    if let Some(e) = world.entities.get_mut(&id) {
        e.in_cover = false;
        e.hidden_in_grass = false;
        e.host_cover_id = None;
        e.wake();
    }
    if let Some(e) = world.entities.get(&id) {
        world.cell_composition.occupy_entity(x, y, e);
    }
}

/// Cover destroyed or forced out — exit and step to a nearby open cell if needed.
pub fn eject_from_cover(world: &mut WorldState, id: EntityId) {
    let Some(entity) = world.entities.get(&id).cloned() else {
        return;
    };
    if !entity.in_cover {
        return;
    }
    let (x, y) = (entity.x, entity.y);
    if let Some(e) = world.entities.get_mut(&id) {
        e.in_cover = false;
        e.hidden_in_grass = false;
        e.host_cover_id = None;
        e.wake();
    }
    let profile = world.entities.get(&id).map(|e| e.profile.clone());
    let Some(profile) = profile else {
        return;
    };
    if world.cell_composition.can_occupy(x, y, &profile) {
        if let Some(e) = world.entities.get(&id) {
            world.cell_composition.occupy_entity(x, y, e);
        }
        return;
    }
    if let Some((nx, ny)) = crate::systems::movement::find_safe_land_near(world, x, y) {
        let _ = world.move_entity(id, nx, ny);
    } else if let Some(e) = world.entities.get(&id) {
        world.cell_composition.occupy_entity(x, y, e);
    }
}

fn try_hide_in_cover_at(world: &mut WorldState, id: EntityId, hide_tag: &str) {
    if hide_tag.is_empty() {
        return;
    }
    let Some(entity) = world.entities.get(&id).cloned() else {
        return;
    };
    if entity.in_cover {
        return;
    }
    let (x, y) = (entity.x, entity.y);
    let Some(cover_id) =
        crate::systems::grass_regen::cover_at_cell(world, x, y, hide_tag)
    else {
        return;
    };
    world.cell_composition.vacate_entity(x, y, &entity);
    if let Some(e) = world.entities.get_mut(&id) {
        e.in_cover = true;
        e.hidden_in_grass = hide_tag == "grass";
        e.host_cover_id = Some(cover_id);
        e.ecology_state = EcologyState::Fleeing;
    }
}

fn try_auto_hide_after_flee(world: &mut WorldState, id: EntityId, profile: &EntityProfile) {
    for drive in &profile.drives {
        if drive.behavior == DriveBehavior::Hide && !drive.target_tag.is_empty() {
            try_hide_in_cover_at(world, id, &drive.target_tag);
            if world.entities.get(&id).is_some_and(|e| e.in_cover) {
                return;
            }
        }
    }
}

#[cfg(test)]
mod hide_tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn hungry_sheep_grazes_via_utility_seek() {
        let mut world = empty_world();
        let grass = world.spawn("grass", 8, 8);
        let sheep = world.spawn("sheep", 8, 7);
        mark_baseline_reactive_tick(&mut world);
        flush_reactive_tick(&mut world, 1.0);
        assert!(world.entities[&sheep].fed_today);
        assert_eq!(world.entities[&grass].hp, 3);
    }

    #[test]
    fn hide_in_cover_vacates_cell_slot() {
        let mut world = empty_world();
        world.spawn("grass", 8, 8);
        let rabbit = world.spawn("rabbit", 8, 8);
        let before = world.cell_composition.slot(8, 8).living_count;
        world.spawn("wolf", 9, 8);
        tick_reactive(&mut world, rabbit, 1.0);
        let rabbit_ent = &world.entities[&rabbit];
        assert!(
            rabbit_ent.in_cover || rabbit_ent.ecology_state == EcologyState::Fleeing,
            "utility should pick hide or flee when predator is adjacent"
        );
        if rabbit_ent.in_cover {
            assert_eq!(world.cell_composition.slot(8, 8).living_count, before - 1);
        }
    }

    #[test]
    fn exit_cover_reoccupies_cell() {
        let mut world = empty_world();
        world.spawn("grass", 8, 8);
        let rabbit = world.spawn("rabbit", 8, 8);
        try_hide_in_cover_at(&mut world, rabbit, "grass");
        let hidden = world.cell_composition.slot(8, 8).living_count;
        exit_cover(&mut world, rabbit);
        assert!(!world.entities[&rabbit].in_cover);
        assert_eq!(
            world.cell_composition.slot(8, 8).living_count,
            hidden + 1
        );
    }
}

#[cfg(test)]
mod sleep_tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn sleeping_entity_skips_reactive_tick() {
        let mut world = empty_world();
        world.tick_count = 10;
        let sheep = world.spawn("sheep", 5, 5);
        world.entities.get_mut(&sheep).unwrap().sleep_until_tick = 20;
        let before = world.entities[&sheep].ecology_state;
        tick_reactive(&mut world, sheep, 1.0);
        assert_eq!(world.entities[&sheep].ecology_state, before);
        assert!(world.entities[&sheep].is_sleeping(10));
    }

    #[test]
    fn fed_idle_entity_enters_sleep_after_tick() {
        let mut world = empty_world();
        world.tick_count = 10;
        let sheep = world.spawn("sheep", 5, 5);
        {
            let e = world.entities.get_mut(&sheep).unwrap();
            e.starve_days = 0;
            e.ecology_state = EcologyState::Idle;
        }
        tick_reactive(&mut world, sheep, 1.0);
        assert!(world.entities[&sheep].is_sleeping(10));
        assert_eq!(world.entities[&sheep].sleep_until_tick, 15);
    }

    #[test]
    fn nearby_movement_wakes_sleeping_entity() {
        let mut world = empty_world();
        world.tick_count = 10;
        let sheep = world.spawn("sheep", 5, 5);
        world.entities.get_mut(&sheep).unwrap().sleep_until_tick = 20;
        wake_autonomous_near(&mut world, 6, 5, WAKE_ACTIVITY_RANGE);
        assert!(!world.entities[&sheep].is_sleeping(10));
    }
}
