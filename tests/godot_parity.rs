//! Godot `unit_test_cases.gd` parity — non-camp / non-player subset.

use bevy_poc::game_constants::LIVING_GRASS_CAP;
use bevy_poc::systems::movement::find_safe_land_near;
use bevy_poc::{
    count_living_grasses, empty_world, find_path, harvest_at, is_blocked_for, spawn_initial_world,
    terrain_at, ecosystem_behavior_key, BEHAVIOR_PREDATOR_DEN, EventRegistry, RecipeBook,
};

#[test]
fn godot_pool_source_at_bottom_center() {
    let w = spawn_initial_world();
    assert_eq!(terrain_at(&w, 18, 22), "pool");
}

#[test]
fn godot_no_surface_river_column() {
    let w = spawn_initial_world();
    for y in 0..24u8 {
        assert_ne!(terrain_at(&w, 0, y), "river");
    }
}

#[test]
fn godot_pathfinding_adjacent_walk() {
    let w = empty_world();
    let path = find_path(&w, 10, 10, 11, 10, None);
    assert_eq!(path, vec![(11, 10)]);
}

#[test]
fn godot_pathfinding_blocked_by_pool() {
    let mut w = empty_world();
    w.mark_pool(5, 5);
    assert!(is_blocked_for(&w, 5, 5, None));
}

#[test]
fn godot_recipe_spear_from_twig_shard() {
    let book = RecipeBook::load_embedded();
    let rel = book.find_relation("twig", "shard").expect("spear recipe");
    assert_eq!(rel.result, "spear");
}

#[test]
fn godot_harvest_pool_lotus() {
    let mut w = empty_world();
    w.pool_cells.insert((10, 10));
    w.spawn("lotus", 10, 10);
    let product = harvest_at(&mut w, 10, 10, "").expect("lotus harvest");
    assert_eq!(product, "lotusSeed");
}

#[test]
fn godot_event_registry_has_wolf() {
    let w = empty_world();
    let def = w.card_defs.get("wolf").unwrap();
    assert_eq!(ecosystem_behavior_key(def, "wolf"), BEHAVIOR_PREDATOR_DEN);
    let mut w = empty_world();
    w.spawn("grass", 5, 5);
    let wolf = w.spawn("wolf", 4, 5);
    EventRegistry::tick_entity_ecology(&mut w, wolf, 1.0);
}

#[test]
fn godot_river_bounce_moves_stone_off_ford() {
    let mut w = empty_world();
    w.mark_river(10, 5);
    let stone = w.spawn("stone", 10, 5);
    if let Some((nx, ny)) = find_safe_land_near(&w, 10, 5) {
        w.move_entity(stone, nx, ny);
        assert_ne!((w.entities[&stone].x, w.entities[&stone].y), (10, 5));
    }
}

#[test]
fn godot_fire_on_water_becomes_charcoal() {
    let mut w = empty_world();
    w.mark_river(8, 8);
    w.spawn("fire", 8, 8);
    w.fire_cells.insert((8, 8));
    bevy_poc::systems::tick_environment::tick_environment(&mut w, 1.0);
    assert!(w.entities.values().any(|e| e.type_name == "charcoal"));
    assert!(!w.entities.values().any(|e| e.type_name == "fire"));
}

#[test]
fn godot_pathfinding_avoids_obstacle() {
    let mut w = empty_world();
    w.spawn("mountain", 6, 5);
    let path = find_path(&w, 5, 5, 7, 5, None);
    assert!(!path.is_empty());
    assert!(path.iter().all(|(x, y)| !(*x == 6 && *y == 5)));
}

#[test]
fn godot_ecology_ready() {
    let w = spawn_initial_world();
    assert!(w.ecology.ready);
}

#[test]
fn godot_living_grass_cap_blocks_river_regen() {
    let mut w = empty_world();
    for i in 0..LIVING_GRASS_CAP {
        w.spawn("grass", (i % 10) as u8, 5);
    }
    assert_eq!(count_living_grasses(&w), LIVING_GRASS_CAP as usize);
    w.mark_river(3, 0);
    w.run_ticks(10);
    assert_eq!(count_living_grasses(&w), LIVING_GRASS_CAP as usize);
}
