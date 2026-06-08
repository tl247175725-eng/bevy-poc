use bevy_poc::{
    cell_center, grid_from_cursor, grid_round_trip, grid_to_world, world_to_grid,
    zoom_anchor_invariant, WorldView, GRID_HEIGHT, GRID_WIDTH,
};
use bevy_poc::viewport_layout::ViewportLayout;

#[test]
fn grid_world_round_trip_center() {
    let max_x = GRID_WIDTH - 1;
    let max_y = GRID_HEIGHT - 1;
    for y in [0u8, 5, 12, max_y] {
        for x in [0u8, 10, max_x / 2, max_x] {
            let rt = grid_round_trip(x, y).expect("in bounds");
            assert_eq!(rt, (x, y), "round-trip at ({x},{y})");
        }
    }
}

#[test]
fn grid_to_world_matches_cell_center() {
    let (x, y) = (18u8, 22u8);
    let center = cell_center(x, y);
    let from_card = grid_to_world(x, y);
    assert!((center.x - from_card.x).abs() < 0.01);
    assert!((center.y - from_card.y).abs() < 1.0);
}

#[test]
fn world_to_grid_corners() {
    assert_eq!(world_to_grid(bevy::math::Vec2::ZERO), Some((0, 0)));
    let max_x = GRID_WIDTH - 1;
    let max_y = GRID_HEIGHT - 1;
    let edge = world_to_grid(bevy::math::Vec2::new(
        56.0 * (max_x as f32 + 0.9),
        56.0 * (max_y as f32 + 0.9),
    ));
    assert_eq!(edge, Some((max_x, max_y)));
}

#[test]
fn zoom_anchor_keeps_world_point_fixed() {
    let area = bevy::math::Vec2::new(800.0, 600.0);
    let mouse = bevy::math::Vec2::new(400.0, 300.0);
    let mut view = WorldView::default();
    let delta = zoom_anchor_invariant(&mut view, mouse, area, 1);
    assert!(
        delta.length() < 0.05,
        "zoom anchor drift {delta:?} zoom={}",
        view.zoom
    );
    let delta2 = zoom_anchor_invariant(&mut view, mouse, area, -1);
    assert!(delta2.length() < 0.05);
}

#[test]
fn cursor_grid_mapping_in_world_area() {
    let layout = ViewportLayout {
        window_width: 1280.0,
        window_height: 720.0,
        scale_factor: 1.0,
    };
    let view = WorldView::default();
    let cursor = bevy::math::Vec2::new(100.0, 100.0);
    let grid = grid_from_cursor(cursor, &layout, &view).expect("in area");
    assert!(grid.0 < GRID_WIDTH && grid.1 < GRID_HEIGHT);
}
