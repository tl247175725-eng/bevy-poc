//! Cross-layer integration — coords, ecology tick, card stack, selection.

use bevy_poc::{
    card_world_pos, cell_center, empty_world, flush_herbivore_tick, mark_baseline_herbivore_tick,
    spawn_initial_world, stack_indices, ui_containment_entries, world_to_grid, world_width,
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
    let sheep = w.spawn("sheep", 8, 7);
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
fn cross_layer_stack_indices_one_living_card_per_cell() {
    let mut w = empty_world();
    let stone = w.spawn("stone", 4, 4);
    let twig = w.spawn("twig", 4, 4);
    let wood = w.spawn("wood", 4, 4);
    let stacks = stack_indices(&w);
    assert_eq!(stacks.get(&(4, 4)), Some(&vec![stone.0]));
    assert_ne!((w.entities[&twig].x, w.entities[&twig].y), (4, 4));
    assert_ne!((w.entities[&wood].x, w.entities[&wood].y), (4, 4));
}

#[test]
fn cross_layer_in_tree_entity_excluded_from_stack_indices() {
    let mut w = empty_world();
    let stone = w.spawn("stone", 6, 6);
    let tree = w.spawn("tree", 7, 7);
    let oak = w.spawn("oak", 7, 7);
    if let Some(e) = w.entities.get_mut(&oak) {
        e.in_tree = true;
        e.host_tree_id = Some(tree);
        e.x = 7;
        e.y = 7;
    }
    let stacks = stack_indices(&w);
    assert_eq!(stacks.get(&(6, 6)), Some(&vec![stone.0]));
    assert_eq!(stacks.get(&(7, 7)), Some(&vec![tree.0]));
    assert!(!stacks.values().any(|ids| ids.contains(&oak.0)));
}

#[test]
fn cross_layer_initial_bird_nest_contained_in_tree() {
    let w = spawn_initial_world();
    let nest = w
        .entities
        .values()
        .find(|e| e.type_name == "birdNest")
        .expect("birdNest spawned");
    assert!(nest.in_tree);
    let tree_id = nest.host_tree_id.expect("birdNest host_tree_id");
    let tree = &w.entities[&tree_id];
    assert_eq!((nest.x, nest.y), (tree.x, tree.y));

    let stacks = stack_indices(&w);
    let tree_stack = stacks.get(&(tree.x, tree.y)).expect("tree on grid");
    assert!(tree_stack.contains(&tree_id.0));
    assert!(!tree_stack.contains(&nest.id.0));

    let entries = ui_containment_entries(&w, tree.x, tree.y, Some(tree_id));
    assert!(entries.iter().any(|e| e.entity_id == nest.id));
}
