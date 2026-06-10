use crate::ecology_log::{card_display_name, eco_log};
use crate::game_constants::{
    BUG_CARCASS_ATTRACT_RANGE, CORPSE_DECAY_SECONDS, HUMUS_DURATION_SECONDS,
    HUMUS_MAX_LAYERS, LAND_BUG_POP_CAP, LIVING_GRASS_CAP, PERISHABLE_TICKS, PLAYER_CORPSE_DECAY,
    RIPARIAN_GRASS_INTERVAL, WILDYAM_POP_CAP, WOLF_CORPSE_DECAY, YAM_SPREAD_INTERVAL,
};
use crate::systems::movement::find_safe_land_near;
use crate::terrain::{base_terrain_at, terrain_at};
use crate::world_rules::{count_living_grasses, count_living_grasses_near_xy, is_being, is_camp_fire_anchor, GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;

pub fn tick_environment(world: &mut WorldState, delta: f32) {
    crate::systems::grass_regen::tick_grass_regen(world);
    crate::systems::grass_regen::tick_bush_regen(world);
    crate::systems::tick_starvation::tick_starvation(world);
    tick_river_bounce(world);
    tick_fire_on_water(world);
    tick_riparian_grass(world, delta);
    tick_perishable(world, delta);
    tick_corpses(world, delta);
    tick_land_bugs(world, delta);
    tick_underground_spread(world, delta);
    crate::systems::tick_containment::tick_contained_producers(world, delta);
    crate::systems::tick_aquatic::tick_aquatic(world, delta);
}

/// Godot `_river_bounce` — non-beings on river/ford bounce to nearby land.
fn tick_river_bounce(world: &mut WorldState) {
    let mut moves: Vec<(crate::spatial_index::EntityId, u8, u8)> = Vec::new();
    for entity in world.entities.values() {
        if entity.in_pool || entity.in_tree || entity.in_ground || entity.in_den {
            continue;
        }
        let terrain = terrain_at(world, entity.x, entity.y);
        if !matches!(terrain, "river" | "ford") {
            continue;
        }
        if let Some(def) = world.card_defs.get(&entity.type_name) {
            if is_being(def) || is_camp_fire_anchor(def) {
                continue;
            }
        }
        if let Some((nx, ny)) = find_safe_land_near(world, entity.x, entity.y) {
            moves.push((entity.id, nx, ny));
        }
    }
    for (id, nx, ny) in moves {
        if let Some(name) = world.entities.get(&id).map(|e| card_display_name(world, &e.type_name)) {
            eco_log(
                world,
                format!("{name}落入水域，弹到陆地({nx},{ny})"),
            );
        }
        if world.move_entity(id, nx, ny) != crate::world_state::MoveResult::Moved {
            continue;
        }
    }
}

/// Godot `_fire_on_water` — camp fire anchor on river/ford becomes charcoal.
fn tick_fire_on_water(world: &mut WorldState) {
    let fires: Vec<_> = world
        .spatial_index
        .query_tag("camp.anchor")
        .into_iter()
        .filter_map(|id| world.entities.get(&id).map(|e| (e.id, e.x, e.y, e.type_name.clone())))
        .collect();
    for (id, x, y, type_name) in fires {
        let Some(def) = world.card_defs.get(&type_name) else {
            continue;
        };
        if !is_camp_fire_anchor(def) {
            continue;
        }
        let terrain = base_terrain_at(world, x, y);
        if matches!(terrain, "river" | "ford") {
            world.remove_entity(id);
            world.fire_cells.remove(&(x, y));
            world.spawn("charcoal", x, y);
            eco_log(world, "篝火接触水域 → 熄灭变炭");
        }
    }
}

/// Godot `_update_riparian_grass` — bank/wetland cells sprout grass periodically.
fn tick_riparian_grass(world: &mut WorldState, delta: f32) {
    if !world.ecology.ready {
        return;
    }
    world.riparian_timer += delta;
    if world.riparian_timer < RIPARIAN_GRASS_INTERVAL {
        return;
    }
    world.riparian_timer = 0.0;
    if count_living_grasses(world) >= LIVING_GRASS_CAP as usize {
        return;
    }
    let mut bank_spots: Vec<(u8, u8, usize)> = Vec::new();
    for y in 1..GRID_HEIGHT - 1 {
        for x in 1..GRID_WIDTH - 1 {
            if !world.ecology.is_riparian_grass_cell(x, y) {
                continue;
            }
            if !world.entities_at(x, y).is_empty() {
                continue;
            }
            let near = count_living_grasses_near_xy(world, x, y, 1);
            bank_spots.push((x, y, near));
        }
    }
    if bank_spots.is_empty() {
        return;
    }
    bank_spots.sort_by_key(|(_, _, near)| *near);
    if bank_spots[0].2 > 1 {
        return;
    }
    let (x, y, _) = bank_spots[0];
    world.spawn("grass", x, y);
    eco_log(world, "河岸湿润地自然长出草皮");
}

fn tick_perishable(world: &mut WorldState, delta: f32) {
    let ids: Vec<_> = world
        .entities
        .values()
        .filter(|e| e.perish_ticks >= 0)
        .map(|e| e.id)
        .collect();
    for id in ids {
        if let Some(e) = world.entities.get_mut(&id) {
            if e.perish_ticks == 0 {
                e.perish_ticks = PERISHABLE_TICKS;
            }
            e.perish_ticks -= delta as i32;
            if e.perish_ticks <= 0 {
                world.remove_entity(id);
            }
        }
    }
}

fn tick_corpses(world: &mut WorldState, delta: f32) {
    let corpses: Vec<_> = world
        .entities
        .values()
        .filter(|e| e.is_corpse || e.type_name.ends_with("Corpse"))
        .map(|e| (e.id, e.type_name.clone(), e.x, e.y, e.decay_timer))
        .collect();
    for (id, type_name, x, y, mut decay) in corpses {
        let land_bug_bonus = if world.count_type("landBug") > 0
            && world.has_tag_at(x, y, "landBug")
        {
            2.0
        } else {
            1.0
        };
        let max_decay = match type_name.as_str() {
            "wolfCorpse" => WOLF_CORPSE_DECAY,
            "playerCorpse" => PLAYER_CORPSE_DECAY,
            _ => CORPSE_DECAY_SECONDS,
        };
        decay += delta * land_bug_bonus;
        if decay >= max_decay {
            world.remove_entity(id);
            let had_layers = world.humus_layers.get(&(x, y)).copied().unwrap_or(0);
            try_spawn_humus(world, x, y);
            if world.humus_layers.get(&(x, y)).copied().unwrap_or(0) > had_layers {
                eco_log(world, "尸体腐解 → 腐殖土（humus）回到土壤循环");
            } else {
                eco_log(world, "尸体腐解 → 该格腐殖层已满");
            }
        } else if let Some(e) = world.entities.get_mut(&id) {
            e.decay_timer = decay;
        }
    }
}

fn try_spawn_humus(world: &mut WorldState, x: u8, y: u8) {
    let layers = world.humus_layers.get(&(x, y)).copied().unwrap_or(0);
    if layers >= HUMUS_MAX_LAYERS {
        return;
    }
    world.humus_layers.insert((x, y), layers + 1);
    world.humus_age.insert((x, y), 0.0);
    world.spawn("humus", x, y);
}

fn tick_land_bugs(world: &mut WorldState, delta: f32) {
    let _ = delta;
    if (world.count_type("landBug") as i32) >= LAND_BUG_POP_CAP {
        return;
    }
    let corpses: Vec<(u8, u8)> = world
        .entities
        .values()
        .filter(|e| e.is_corpse)
        .map(|e| (e.x, e.y))
        .collect();
    for (cx, cy) in corpses {
        if !world.has_tag_at(cx, cy, "landBug") {
            world.spawn("landBug", cx, cy);
            break;
        }
        let near_corpse = world
            .entities
            .values()
            .any(|e| {
                e.is_corpse
                    && crate::world_rules::chebyshev_distance(cx, cy, e.x, e.y)
                        <= BUG_CARCASS_ATTRACT_RANGE
            });
        if near_corpse && world.count_type("landBug") == 0 {
            world.spawn("landBug", cx, cy);
            break;
        }
    }
}

fn tick_underground_spread(world: &mut WorldState, delta: f32) {
    world.yam_timer += delta;
    if world.yam_timer < YAM_SPREAD_INTERVAL {
        return;
    }
    world.yam_timer = 0.0;
    if (world.count_type("wildYam") as i32) >= WILDYAM_POP_CAP {
        return;
    }
    let yams: Vec<(u8, u8)> = world
        .entities
        .values()
        .filter(|e| e.type_name == "wildYam")
        .map(|e| (e.x, e.y))
        .collect();
    for (x, y) in yams {
        let nx = (x + 1).min(crate::world_rules::GRID_WIDTH - 1);
        if !world.has_tag_at(nx, y, "underground") {
            world.spawn("wildYam", nx, y);
            break;
        }
    }
    let humus_keys: Vec<(u8, u8)> = world.humus_age.keys().copied().collect();
    for key in humus_keys {
        let age = {
            let age = world.humus_age.get_mut(&key).unwrap();
            *age += delta;
            *age
        };
        if age >= crate::game_constants::HUMUS_GRASS_SECONDS
            && !world.has_tag_at(key.0, key.1, "grass")
        {
            world.spawn("grass", key.0, key.1);
            eco_log(world, "腐殖土 → 草芽");
        }
        if age >= HUMUS_DURATION_SECONDS {
            world.humus_age.remove(&key);
            world.humus_layers.remove(&key);
            let to_remove: Vec<_> = world
                .entities
                .values()
                .filter(|e| e.type_name == "humus" && e.x == key.0 && e.y == key.1)
                .map(|e| e.id)
                .collect();
            for id in to_remove {
                world.remove_entity(id);
            }
        }
    }
}
