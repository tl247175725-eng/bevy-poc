use bevy_poc::{
    apply_camera_zoom, build_card_panel, build_cell_panel, build_panel, can_hunt_target,
    contains_english_tag, demo_world, empty_world, entity_state_label, flocking_blocks_reproduction,
    handle_selection_click, panel_text_joined, select_containment_entry, try_place_entity,
    EcologyState, PlaceResult, SelectionState, SelectionTarget,
};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn tw() -> bevy_poc::WorldState {
    let mut w = empty_world();
    for x in 0..8 {
        w.mark_river(x, 0);
    }
    w
}

fn select_at(w: &bevy_poc::WorldState, x: u8, y: u8) -> SelectionState {
    let mut s = SelectionState::default();
    handle_selection_click(w, x, y, &mut s);
    s
}

// --- M3 20 ---

#[test]
fn m3_01_click_grass_shows_name() {
    let mut w = tw();
    let g = w.spawn("grass", 5, 5);
    let s = select_at(&w, 5, 5);
    let panel = build_card_panel(&w, g, 5, 5);
    assert!(panel.title.contains("草皮"));
    assert_eq!(s.target.unwrap().card_id, Some(g));
}

#[test]
fn m3_02_click_sheep_shows_tags() {
    let mut w = tw();
    let id = w.spawn("sheep", 6, 6);
    let panel = build_card_panel(&w, id, 6, 6);
    let text = panel_text_joined(&panel);
    assert!(text.contains("动物"));
    assert!(text.contains("食草"));
    assert!(text.contains("集群"));
}

#[test]
fn m3_03_click_wolf_shows_tags() {
    let mut w = tw();
    let id = w.spawn("wolf", 7, 7);
    let panel = build_card_panel(&w, id, 7, 7);
    let text = panel_text_joined(&panel);
    assert!(text.contains("捕食者"));
    assert!(text.contains("群猎"));
}

#[test]
fn m3_04_tree_shows_containment() {
    let mut w = tw();
    let tree = w.spawn("oak", 10, 10);
    let pine = w.spawn("pine", 10, 10);
    if let Some(e) = w.entities.get_mut(&pine) {
        e.in_tree = true;
        e.host_tree_id = Some(tree);
    }
    let panel = build_card_panel(&w, tree, 10, 10);
    let names: Vec<_> = panel.containment.iter().map(|e| e.display_name.as_str()).collect();
    assert!(names.iter().any(|n| n.contains("栎") || n.contains("松")));
}

#[test]
fn m3_05_pool_shows_containment() {
    let mut w = tw();
    w.mark_pool(8, 12);
    w.spawn("fish", 8, 12);
    w.spawn("shellfish", 8, 12);
    w.spawn("algae", 8, 12);
    let panel = build_cell_panel(&w, 8, 12, 0.0);
    assert!(!panel.containment.is_empty());
    let names: Vec<_> = panel
        .containment
        .iter()
        .map(|e| e.display_name.as_str())
        .collect();
    assert!(names.iter().any(|n| *n == "鱼" || *n == "贝" || *n == "水藻"));
}

#[test]
fn m3_06_selection_border_visible() {
    let mut w = tw();
    w.spawn("sheep", 4, 4);
    let s = select_at(&w, 4, 4);
    assert!(s.border_visible);
}

#[test]
fn m3_07_empty_cell_no_border() {
    let w = tw();
    let s = select_at(&w, 3, 3);
    assert!(!s.border_visible);
    let panel = build_panel(
        &w,
        &SelectionTarget {
            cell_x: 3,
            cell_y: 3,
            card_id: None,
        },
    );
    assert!(panel.title.contains('3'));
}

#[test]
fn m3_08_drag_sheep_to_empty() {
    let mut w = tw();
    let id = w.spawn("sheep", 5, 5);
    let result = try_place_entity(&mut w, id, 8, 8);
    assert_eq!(result, PlaceResult::Moved);
    assert_eq!(w.entities[&id].x, 8);
    assert_eq!(w.entities[&id].y, 8);
}

#[test]
fn m3_09_drag_to_water_reverts() {
    let mut w = tw();
    w.mark_pool(9, 9);
    let id = w.spawn("sheep", 5, 5);
    let result = try_place_entity(&mut w, id, 9, 9);
    assert_eq!(result, PlaceResult::Reverted);
}

#[test]
fn m3_10_tags_fully_chinese() {
    let mut w = tw();
    for name in ["sheep", "wolf", "grass", "oak", "fish", "deer", "rabbit"] {
        let id = w.spawn(name, 2, 2);
        let panel = build_card_panel(&w, id, 2, 2);
        let text = panel_text_joined(&panel);
        assert!(
            !contains_english_tag(&text),
            "english in panel for {name}: {text}"
        );
    }
}

#[test]
fn m3_11_camera_zoom_clamps() {
    assert!((apply_camera_zoom(1.0) - 1.0).abs() < f32::EPSILON);
    assert_eq!(apply_camera_zoom(0.1), 0.4);
    assert_eq!(apply_camera_zoom(10.0), 3.0);
}

#[test]
fn m3_12_camera_pan_state() {
    let mut pan = bevy_poc::CameraPanState::default();
    assert!(!pan.panning);
    pan.panning = true;
    assert!(pan.panning);
}

#[test]
fn m3_13_panel_hp() {
    let mut w = tw();
    let id = w.spawn("sheep", 5, 5);
    w.entities.get_mut(&id).unwrap().hp = 2;
    let panel = build_card_panel(&w, id, 5, 5);
    assert!(panel.lines.iter().any(|l| l.contains("HP：2")));
}

#[test]
fn m3_14_panel_sex() {
    let mut w = tw();
    let id = w.spawn_with_sex("sheep", 5, 5, Some("male".into()));
    let panel = build_card_panel(&w, id, 5, 5);
    assert!(panel.lines.iter().any(|l| l.contains("性别：公")));

    let id2 = w.spawn_with_sex("sheep", 6, 6, Some("female".into()));
    let panel2 = build_card_panel(&w, id2, 6, 6);
    assert!(panel2.lines.iter().any(|l| l.contains("性别：母")));
}

#[test]
fn m3_15_containment_entry_jumps() {
    let mut w = tw();
    let tree = w.spawn("oak", 10, 10);
    let pine = w.spawn("pine", 10, 10);
    if let Some(e) = w.entities.get_mut(&pine) {
        e.in_tree = true;
        e.host_tree_id = Some(tree);
    }
    let mut s = select_at(&w, 10, 10);
    select_containment_entry(&w, pine, &mut s);
    assert_eq!(s.target.as_ref().unwrap().card_id, Some(pine));
    let panel = build_card_panel(&w, pine, 10, 10);
    assert!(panel.title.contains("松"));
}

#[test]
fn m3_16_panel_state_labels() {
    let mut w = tw();
    let sheep = w.spawn("sheep", 5, 5);
    if let Some(e) = w.entities.get_mut(&sheep) {
        e.ecology_state = EcologyState::SeekingFood;
    }
    let def = w.card_defs.get("sheep").unwrap().clone();
    let entity = w.entities.get(&sheep).unwrap();
    assert_eq!(entity_state_label(entity, &def), "吃草");

    if let Some(e) = w.entities.get_mut(&sheep) {
        e.ecology_state = EcologyState::Wandering;
    }
    let entity = w.entities.get(&sheep).unwrap();
    assert_eq!(entity_state_label(entity, &def), "游荡");

    if let Some(e) = w.entities.get_mut(&sheep) {
        e.ecology_state = EcologyState::Fleeing;
    }
    let entity = w.entities.get(&sheep).unwrap();
    assert_eq!(entity_state_label(entity, &def), "逃跑");
}

#[test]
fn m3_17_food_web_still_passes() {
    let mut w = tw();
    w.spawn("grass", 8, 8);
    w.spawn("sheep", 8, 7);
    let before = w.sheep_count();
    w.tick_once();
    assert_eq!(w.sheep_count(), before);
    let world = empty_world();
    let sheep = &world.card_defs["sheep"];
    let adults = vec![sheep, sheep];
    assert!(flocking_blocks_reproduction(&adults));
    let wolf = &world.card_defs["wolf"];
    let rabbit = &world.card_defs["rabbit"];
    assert!(can_hunt_target(1, wolf, rabbit));
}

#[test]
fn m3_18_bench_avg_tick_under_1ms() {
    let mut world = demo_world();
    let start = Instant::now();
    let ticks: u64 = 10_000;
    for _ in 0..ticks {
        world.tick_once();
    }
    let avg_ms = start.elapsed().as_secs_f64() * 1000.0 / ticks as f64;
    assert!(
        avg_ms < 1.0,
        "avg_tick_ms={avg_ms:.4} expected < 1.0"
    );
}

#[test]
fn m3_19_design_docs_use_rust_paths() {
    let overview = fs::read_to_string("docs/design/game-design-overview.md")
        .expect("game-design-overview.md");
    assert!(
        !overview.contains(".gd"),
        "game-design-overview still references .gd files"
    );
    assert!(
        overview.contains("Bevy") || overview.contains(".rs"),
        "game-design-overview should mention Bevy/Rust implementation"
    );
    let fix_log = fs::read_to_string("FIX_LOG.md").expect("FIX_LOG.md");
    assert!(fix_log.contains("bevy-poc"));
}

#[test]
fn m3_20_release_build_no_panic() {
    if !Path::new("Cargo.toml").exists() {
        return;
    }
    let status = Command::new("cargo")
        .args(["build", "--release", "--quiet"])
        .status()
        .expect("cargo build --release");
    assert!(status.success(), "release build failed");
}
