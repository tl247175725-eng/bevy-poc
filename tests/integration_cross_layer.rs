//! Cross-layer integration — coords, ecology tick, card stack, selection.

use bevy_poc::{
    card_world_pos, cell_center, empty_world, flush_herbivore_tick, mark_baseline_herbivore_tick,
    stack_indices, world_to_grid, world_width,
};

#[test]
fn cross_layer_cell_center_round_trips_to_grid() {
    let (x, y) = (5u8, 3u8);
    let center = cell_center(x, y).truncate();
    assert_eq!(world_to_grid(center), Some((x, y)));
}

#[test]
fn cross_layer_herbivore_baseline_grazing_tick() {
    let mut w = empty_world();
    let grass = w.spawn("grass", 8, 8);
    let sheep = w.spawn("sheep", 8, 8);
    mark_baseline_herbivore_tick(&mut w);
    assert!(w.entities[&sheep].needs_grazing_tick);
    flush_herbivore_tick(&mut w, 1.0);
    assert!(!w.entities[&sheep].needs_grazing_tick);
    assert!(!w.entities.contains_key(&grass));
}

#[test]
fn cross_layer_card_world_pos_x_within_world_width() {
    let pos = card_world_pos(0, 0, 1, None);
    assert!(pos.x >= 0.0);
    assert!(pos.x <= world_width());
}

#[test]
fn cross_layer_stack_indices_orders_three_surface_cards() {
    let mut w = empty_world();
    let a = w.spawn("stone", 4, 4);
    let b = w.spawn("twig", 4, 4);
    let c = w.spawn("wood", 4, 4);
    let stacks = stack_indices(&w);
    assert_eq!(
        stacks.get(&(4, 4)).map(Vec::as_slice),
        Some(&[a.0, b.0, c.0][..])
    );
}

#[test]
fn cross_layer_in_tree_entity_excluded_from_stack_indices() {
    let mut w = empty_world();
    let stone = w.spawn("stone", 6, 6);
    let twig = w.spawn("twig", 6, 6);
    let oak = w.spawn("oak", 6, 6);
    if let Some(e) = w.entities.get_mut(&oak) {
        e.in_tree = true;
    }
    let stacks = stack_indices(&w);
    let cell = stacks.get(&(6, 6)).expect("surface stack");
    assert_eq!(cell, &[stone.0, twig.0]);
    assert!(!cell.contains(&oak.0));
}
