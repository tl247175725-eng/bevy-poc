use crate::visual_config::{START_PLAYER_X, START_PLAYER_Y};
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;

/// Godot `WorldManager._spawn_initial_cards` — deterministic subset for ~123 cards.
pub fn spawn_initial_world() -> WorldState {
    let mut w = WorldState::from_card_defs_file(crate::assets_util::card_defs_path());
    w.ensure_map_ecology();
    let px = START_PLAYER_X;
    let py = START_PLAYER_Y;

    for x in 0..GRID_WIDTH {
        w.spawn("mountain", x, 0);
        w.spawn("mountain", x, GRID_HEIGHT - 1);
    }
    for y in 1..GRID_HEIGHT - 1 {
        w.spawn("mountain", 0, y);
        w.spawn("mountain", GRID_WIDTH - 1, y);
    }

    w.spawn("player", px, py);
    w.spawn("stone", px - 4, py);
    w.spawn("stone", px - 3, py);
    w.spawn("twig", px - 2, py - 1);
    w.spawn("wood", px - 8, py - 2);
    w.spawn("wood", px + 6, py - 4);
    let tree1 = w.spawn("tree", px - 10, py);
    let tree2 = w.spawn("tree", px - 12, py - 8);
    spawn_tree_flora(&mut w, tree1, "oak");
    spawn_tree_flora(&mut w, tree2, "pine");
    spawn_tree_guest(&mut w, tree1, "birdNest");
    w.spawn("bucket", px, py + 2);
    w.spawn("mountain", px - 14, py - 4);
    w.spawn("mountain", px + 10, py - 6);

    for &(gx, gy) in &initial_grass_positions(px, py) {
        w.spawn("grass", gx, gy);
    }

    spawn_sheep_pack(&mut w, px - 10, py - 6);
    w.spawn_with_sex("wolf", px - 4, py - 6, Some("male".into()));
    w.spawn_with_sex("wolf", px - 2, py - 6, Some("female".into()));

    for i in 0..6 {
        let sex = if i % 2 == 0 { "male" } else { "female" };
        w.spawn_with_sex("rabbit", px - 12 + i, py + 2, Some(sex.into()));
    }
    for i in 0..5 {
        let sex = if i < 2 { "male" } else { "female" };
        w.spawn_with_sex("deer", px - 14 - i, py + 4 + (i / 2), Some(sex.into()));
    }
    w.spawn_with_sex("waterBuffalo", px + 11, py + 3, Some("male".into()));
    w.spawn_with_sex("waterBuffalo", px + 12, py + 3, Some("female".into()));
    w.spawn_with_sex("waterBuffalo", px + 13, py + 3, Some("female".into()));

    spawn_taoyuan(&mut w);
    spawn_step2_ecology(&mut w, px, py);
    spawn_water_ecology(&mut w);
    spawn_wild_yams(&mut w);

    w
}

fn initial_grass_positions(px: u8, py: u8) -> [(u8, u8); 10] {
    [
        (px - 1, py + 1),
        (px - 8, py + 2),
        (px + 2, py + 2),
        (px - 2, py - 2),
        (px + 4, py + 1),
        (px - 6, py + 3),
        (px + 1, py - 3),
        (px - 4, py + 4),
        (px + 6, py + 2),
        (px - 9, py - 1),
    ]
}

fn spawn_sheep_pack(w: &mut WorldState, x: u8, y: u8) {
    let sexes = ["male", "male", "female", "female", "female"];
    for (i, sex) in sexes.iter().enumerate() {
        w.spawn_with_sex("sheep", x + i as u8, y, Some((*sex).into()));
    }
}

fn spawn_tree_flora(w: &mut WorldState, tree_id: crate::spatial_index::EntityId, kind: &str) {
    spawn_tree_guest(w, tree_id, kind);
}

/// Spawn flora/nest inside a tree — does not occupy the grid cell (one-card-per-cell).
fn spawn_tree_guest(w: &mut WorldState, tree_id: crate::spatial_index::EntityId, type_name: &str) {
    let Some(tree) = w.entities.get(&tree_id).cloned() else {
        return;
    };
    let id = w.spawn(type_name, tree.x, tree.y);
    let Some(e) = w.entities.get_mut(&id) else {
        return;
    };
    let guest = e.clone();
    w.cell_composition.vacate_entity(e.x, e.y, &guest);
    e.x = tree.x;
    e.y = tree.y;
    e.in_tree = true;
    e.host_tree_id = Some(tree_id);
    w.spatial_index.move_entity(id, tree.x, tree.y);
}

fn spawn_taoyuan(w: &mut WorldState) {
    let types = ["taoyuanElder", "taoyuanForager", "taoyuanYouth"];
    let mut slots = Vec::new();
    for y in 2..GRID_HEIGHT - 2 {
        for x in GRID_WIDTH - 12..GRID_WIDTH - 2 {
            if w.entities_at(x, y).is_empty() && !w.pool_cells.contains(&(x, y)) {
                slots.push((x, y));
            }
        }
    }
    for (i, type_name) in types.iter().enumerate() {
        if let Some(&(x, y)) = slots.get(i) {
            w.spawn(type_name, x, y);
        }
    }
}

fn spawn_step2_ecology(w: &mut WorldState, px: u8, py: u8) {
    let bush_cells = [
        (px - 5, py + 5),
        (px + 3, py + 6),
        (px - 8, py + 1),
        (px + 7, py + 2),
        (px - 3, py + 8),
        (px + 5, py + 1),
        (px - 11, py + 5),
        (px + 9, py + 5),
    ];
    for &(x, y) in &bush_cells {
        w.spawn("bush", x, y);
    }
    let mouse_cells = [
        (px - 4, py + 6),
        (px + 4, py + 7),
        (px - 7, py + 2),
        (px + 8, py + 3),
        (px - 2, py + 9),
        (px + 6, py + 2),
        (px - 10, py + 6),
        (px + 10, py + 6),
    ];
    for (i, &(x, y)) in mouse_cells.iter().enumerate() {
        let sex = if i % 2 == 0 { "male" } else { "female" };
        w.spawn_with_sex("fieldMouse", x, y, Some(sex.into()));
    }
    for i in 0..4 {
        w.spawn("fieldMousePup", px - 3 + i, py + 7);
    }
    for i in 0..5 {
        let sex = if i < 2 { "male" } else { "female" };
        w.spawn_with_sex("pheasant", px - 13 + i, py + 2, Some(sex.into()));
    }
    for &(x, y) in &[(px + 5, py + 4), (px + 6, py + 4), (px + 5, py + 5), (px + 6, py + 5)] {
        w.spawn_with_sex("bambooRat", x, y, Some("female".into()));
    }
    w.spawn_with_sex("fox", px - 6, py - 4, Some("male".into()));
    w.spawn_with_sex("fox", px - 5, py - 3, Some("female".into()));
    w.spawn("foxCub", px - 5, py - 4);
}

fn spawn_water_ecology(w: &mut WorldState) {
    let pools: Vec<_> = w.pool_cells.iter().copied().collect();
    for (i, &(x, y)) in pools.iter().enumerate() {
        if i < pools.len() * 55 / 100 {
            let id = w.spawn("algae", x, y);
            if let Some(e) = w.entities.get_mut(&id) {
                e.in_pool = true;
            }
        }
    }
    for (i, &(x, y)) in pools.iter().enumerate() {
        if i < 3 {
            let id = w.spawn("waterBug", x, y);
            if let Some(e) = w.entities.get_mut(&id) {
                e.in_pool = true;
            }
        }
    }
    for (i, &(x, y)) in pools.iter().enumerate() {
        if i < 2 {
            let id = w.spawn("fish", x, y);
            if let Some(e) = w.entities.get_mut(&id) {
                e.in_pool = true;
            }
        }
    }
    for (i, &(x, y)) in pools.iter().enumerate() {
        if i >= 2 && i < 4 {
            let id = w.spawn("shellfish", x, y);
            if let Some(e) = w.entities.get_mut(&id) {
                e.in_pool = true;
            }
        }
    }
}

fn spawn_wild_yams(w: &mut WorldState) {
    let mut slots = Vec::new();
    for entity in w.entities.values() {
        if entity.type_name == "tree" || entity.type_name == "oak" || entity.type_name == "pine" {
            for (dx, dy) in [(1i16, 0), (-1, 0), (0, 1), (0, -1)] {
                let x = (entity.x as i16 + dx) as u8;
                let y = (entity.y as i16 + dy) as u8;
                if w.pool_cells.contains(&(x, y)) {
                    continue;
                }
                if w.entities_at(x, y).is_empty() {
                    slots.push((x, y));
                }
            }
        }
    }
    slots.sort();
    slots.dedup();
    for &(x, y) in slots.iter().take(4) {
        let id = w.spawn("wildYam", x, y);
        if let Some(e) = w.entities.get_mut(&id) {
            e.in_ground = true;
        }
    }
}

pub fn initial_card_count() -> usize {
    spawn_initial_world().entities.len()
}
