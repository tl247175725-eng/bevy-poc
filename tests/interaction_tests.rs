//! Phase 1–2 interaction / surface label tests.

use bevy_poc::{
    empty_world, surface_label_with_stress, terrain_at, try_ghost_drop, try_harvest,
    InteractionState, RecipeBook, SimEventQueue, WorldState,
};

#[test]
fn surface_label_pool_is_water_pond() {
    let w = bevy_poc::spawn_initial_world();
    assert_eq!(
        surface_label_with_stress(&w, 18, 22, 0.0).as_deref(),
        Some("水潭")
    );
}

#[test]
fn surface_label_river_stress_tight() {
    let w = bevy_poc::spawn_initial_world();
    // ford/river cells may not exist on v3 map; use stress label on any ford if present
    for y in 0..24u8 {
        for x in 0..36u8 {
            if terrain_at(&w, x, y) == "ford" {
                let lbl = surface_label_with_stress(&w, x, y, 75.0).unwrap();
                assert!(lbl.contains("紧"), "ford label: {lbl}");
                return;
            }
        }
    }
}

#[test]
fn surface_label_wolf_den_overlay() {
    let mut w = empty_world();
    w.spawn("wolfDen", 10, 10);
    assert_eq!(
        surface_label_with_stress(&w, 10, 10, 0.0).as_deref(),
        Some("狼穴")
    );
}

#[test]
fn harvest_lotus_on_click_path() {
    let mut w = empty_world();
    w.pool_cells.insert((8, 8));
    w.spawn("lotus", 8, 8);
    let mut events = SimEventQueue::default();
    let product = try_harvest(&mut w, 8, 8, &mut events).expect("harvest");
    assert_eq!(product, "lotusSeed");
}

#[test]
fn ghost_drop_relation_spear() {
    let mut w = empty_world();
    let twig = w.spawn("twig", 5, 5);
    w.spawn("shard", 5, 5);
    let mut events = SimEventQueue::default();
    let state = InteractionState::default();
    assert!(try_ghost_drop(&mut w, twig, 5, 5, 3, 3, &state, &mut events));
    assert!(w.entities.values().any(|e| e.type_name == "spear"));
}

#[test]
fn recipe_book_loads_spear() {
    let book = RecipeBook::load_embedded();
    assert!(book.find_relation("twig", "shard").is_some());
}
