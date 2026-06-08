//! Phase 4 — selection panel fields + ecology log stream.

use bevy_poc::{
    build_cell_panel, build_panel_with_stress, demo_world, eco_log,
    empty_world, SimEvent, SimEventQueue, SelectionTarget,
};

#[test]
fn phase4_cell_panel_water_stress_and_cover() {
    let mut w = empty_world();
    w.mark_river(10, 5);
    let panel = build_cell_panel(&w, 10, 5, 72.0);
    assert!(panel.lines.iter().any(|l| l.contains("水势：紧")));
    assert!(panel.title.contains("河沟"));
}

#[test]
fn phase4_wolf_panel_shows_meat_counter() {
    let mut w = empty_world();
    let wolf = w.spawn("wolf", 5, 5);
    w.entities.get_mut(&wolf).unwrap().meat_fed_today = 1;
    let panel = bevy_poc::build_card_panel(&w, wolf, 5, 5);
    assert!(panel.lines.iter().any(|l| l.contains("今日肉：1/2")));
}

#[test]
fn phase4_ecology_log_drains_to_sim_events() {
    let mut w = empty_world();
    eco_log(&mut w, "测试生态日志");
    assert_eq!(
        w.drain_pending_events(),
        vec![SimEvent::Generic("测试生态日志".to_string())]
    );

    let mut w = demo_world();
    w.drain_pending_events(); // demo setup spawns are internal, not UI log
    w.run_ticks(50);
    let events = w.drain_pending_events();
    assert!(!events.is_empty(), "expected ecology activity events, got none");

    let mut queue = SimEventQueue::default();
    for event in events {
        queue.push(event);
    }
    let drained = queue.drain();
    assert!(!drained.is_empty());
}

#[test]
fn phase4_empty_cell_panel_via_target() {
    let w = empty_world();
    let panel = build_panel_with_stress(
        &w,
        &SelectionTarget {
            cell_x: 4,
            cell_y: 6,
            card_id: None,
        },
        10.0,
    );
    assert!(panel.title.contains("(4, 6)"));
    assert!(panel.lines.iter().any(|l| l.contains("水势：稳")));
}
