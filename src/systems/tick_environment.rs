use crate::ecology_log::{card_display_name, eco_log};
use crate::game_constants::{
    BUG_CARCASS_ATTRACT_RANGE, HUMUS_DURATION_SECONDS, LAND_BUG_POP_CAP,
    LIVING_GRASS_CAP, PERISHABLE_TICKS, RIPARIAN_GRASS_INTERVAL, WILDYAM_POP_CAP,
    YAM_SPREAD_INTERVAL,
};
use crate::interaction::finalize_prey_kill;
use crate::sim_clock::{season_from_tick, Season};
use crate::spatial_index::EntityId;
use crate::systems::movement::find_safe_land_near;
use crate::terrain::{base_terrain_at, terrain_at};
use crate::world_rules::{
    card_has_tag, count_living_grasses, count_living_grasses_near_xy, is_being,
    is_camp_fire_anchor, GRID_HEIGHT, GRID_WIDTH,
};
use crate::world_state::WorldState;

pub fn tick_environment(world: &mut WorldState, delta: f32) {
    tick_season_effects(world);
    tick_entity_aging(world);
    crate::systems::grass_regen::tick_grass_regen(world);
    crate::systems::grass_regen::tick_bush_regen(world);
    crate::systems::tick_producer_spawn::tick_producer_spawn(world);
    crate::systems::tick_producer_spawn::tick_producer_growth(world);
    crate::systems::tick_decomposer::tick_decomposer(world);
    crate::systems::tick_starvation::tick_starvation(world);
    tick_river_bounce(world);
    tick_fire_on_water(world);
    tick_riparian_grass(world, delta);
    tick_perishable(world, delta);
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

fn tick_season_effects(world: &mut WorldState) {
    let season = season_from_tick(world.tick_count);
    if season.current == Season::Winter {
        let frozen: Vec<EntityId> = world
            .entities
            .values()
            .filter(|e| !e.is_corpse)
            .filter(|e| world.ecology.is_pool_cell(e.x, e.y) || e.in_pool)
            .filter(|e| {
                world
                    .card_defs
                    .get(&e.type_name)
                    .is_some_and(|d| card_has_tag(d, "aquatic") || e.in_pool)
            })
            .map(|e| e.id)
            .collect();
        for id in frozen {
            if let Some(name) = world
                .entities
                .get(&id)
                .map(|e| card_display_name(world, &e.type_name))
            {
                eco_log(world, format!("{name} 困于冰封水池 → 自然死亡"));
            }
            finalize_prey_kill(world, id, None, "natural");
        }
    }
}

fn tick_entity_aging(world: &mut WorldState) {
    let ids: Vec<EntityId> = world
        .entities
        .values()
        .filter(|e| !e.is_corpse && e.max_age > 0.0)
        .map(|e| e.id)
        .collect();
    for id in ids {
        let Some((age, max_age, type_name)) = world.entities.get(&id).map(|e| {
            (e.age, e.max_age, e.type_name.clone())
        }) else {
            continue;
        };
        let Some(def) = world.card_defs.get(&type_name).cloned() else {
            continue;
        };
        if age > max_age * 0.7 {
            if card_has_tag(&def, "trait:frail") {
                if let Some(e) = world.entities.get_mut(&id) {
                    e.profile.move_speed =
                        crate::axioms::profile::parse_move_speed(&def.tags) * 0.7;
                    e.profile.sprint_speed =
                        crate::axioms::profile::parse_sprint_speed(&def.tags) * 0.7;
                }
            }
        }
        if age > max_age {
            eco_log(
                world,
                format!(
                    "{} 寿终正寝（{} 游戏日）",
                    card_display_name(world, &type_name),
                    age.round() as i32
                ),
            );
            finalize_prey_kill(world, id, None, "natural");
        }
    }
}

/// Godot `_update_riparian_grass` — bank/wetland cells sprout grass periodically.
fn tick_riparian_grass(world: &mut WorldState, delta: f32) {
    if !world.ecology.ready {
        return;
    }
    let season = season_from_tick(world.tick_count);
    let interval = if season.current == Season::Summer {
        RIPARIAN_GRASS_INTERVAL * 0.5
    } else {
        RIPARIAN_GRASS_INTERVAL
    };
    world.riparian_timer += delta;
    if world.riparian_timer < interval {
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

#[cfg(test)]
mod environment_tests {
    use super::*;
    use crate::game_constants::TICKS_PER_DAY;
    use crate::world_state::empty_world;

    #[test]
    fn winter_freezes_pool_aquatic() {
        let mut world = empty_world();
        world.ensure_map_ecology();
        world.tick_count = 280 * TICKS_PER_DAY;
        let (px, py) = world.ecology.pool_source;
        let fish = world.spawn("fish", px, py);
        tick_season_effects(&mut world);
        assert!(!world.entities.contains_key(&fish));
    }

    #[test]
    fn natural_death_when_age_exceeds_max() {
        let mut world = empty_world();
        let sheep = world.spawn("sheep", 5, 5);
        world.entities.get_mut(&sheep).unwrap().age = 4000.0;
        tick_entity_aging(&mut world);
        assert!(!world.entities.contains_key(&sheep));
    }

    #[test]
    fn map_ecology_assigns_distinct_soil_tags() {
        let mut world = empty_world();
        world.ensure_map_ecology();
        let rich_cell = world
            .ecology
            .wetland_cells
            .iter()
            .copied()
            .find(|&(x, y)| world.ecology.pool_manhattan_dist(x, y) == 6)
            .expect("inner wetland rich soil");
        let near = world
            .cell_composition
            .slot(rich_cell.0, rich_cell.1)
            .tags
            .clone();
        let far = world.cell_composition.slot(5, 10).tags.clone();
        assert!(near.iter().any(|t| t.starts_with("soil:")));
        assert_ne!(near, far);
    }
}
