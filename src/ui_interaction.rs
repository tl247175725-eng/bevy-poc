use crate::card_visual::CardVisual;
use crate::coords::{cursor_to_world, grid_from_cursor, grid_to_world_in};
use crate::grid_render::SimWorld;
use crate::interaction::{try_ghost_drop, try_harvest, try_impact, InteractionState};
use crate::selection_info::{resolve_selection_card, SelectionTarget};
use crate::sim_events::SimEventQueue;
use crate::spatial_index::EntityId;
use crate::terrain::is_blocked_terrain;
use crate::terrain::terrain_at;
use crate::viewport_layout::ViewportLayout;
use crate::world_state::WorldState;
use crate::world_view::WorldView;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Resource, Default)]
pub struct SelectionState {
    pub target: Option<SelectionTarget>,
    pub border_visible: bool,
    pub revision: u64,
}

#[derive(Resource, Default)]
pub struct DragState {
    pub dragging: bool,
    pub entity_id: Option<EntityId>,
    pub origin_x: u8,
    pub origin_y: u8,
    pub cursor_offset: Vec2,
    pub hit_target: Option<EntityId>,
}

#[derive(Resource, Default)]
pub struct GhostPlaceMode {
    pub dragging: bool,
    pub entity_id: Option<EntityId>,
    pub origin_x: u8,
    pub origin_y: u8,
    pub preview_cell: Option<(u8, u8)>,
    pub cursor_offset: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceResult {
    Moved,
    Reverted,
    Stacked,
    Impacted,
}

pub fn can_drag_entity(world: &WorldState, id: EntityId) -> bool {
    let Some(entity) = world.entities.get(&id) else {
        return false;
    };
    if entity.is_corpse || entity.in_den || entity.in_burrow {
        return false;
    }
    if let Some(def) = world.card_defs.get(&entity.type_name) {
        if def.is_rooted {
            return false;
        }
    }
    true
}

pub fn try_place_entity(world: &mut WorldState, id: EntityId, x: u8, y: u8) -> PlaceResult {
    let terrain = terrain_at(world, x, y);
    if is_blocked_terrain(terrain) {
        return PlaceResult::Reverted;
    }

    let Some(entity) = world.entities.get(&id).cloned() else {
        return PlaceResult::Reverted;
    };

    if entity.x == x && entity.y == y {
        return PlaceResult::Moved;
    }

    world.move_entity(id, x, y);
    if let Some(e) = world.entities.get_mut(&id) {
        e.in_pool = terrain == "pool"
            && matches!(
                e.type_name.as_str(),
                "algae" | "waterBug" | "fish" | "shellfish" | "waterCaltrop" | "lotus"
            );
        e.hidden_in_grass = false;
    }
    world.reindex_entity(id);
    PlaceResult::Moved
}

pub fn revert_drag(world: &mut WorldState, id: EntityId, origin_x: u8, origin_y: u8) {
    world.move_entity(id, origin_x, origin_y);
    world.reindex_entity(id);
}

pub fn handle_selection_click(
    world: &WorldState,
    gx: u8,
    gy: u8,
    selection: &mut SelectionState,
) {
    let card = resolve_selection_card(world, gx, gy);
    selection.target = Some(SelectionTarget {
        cell_x: gx,
        cell_y: gy,
        card_id: card,
    });
    selection.border_visible = card.is_some();
    selection.revision += 1;
}

pub fn select_containment_entry(
    world: &WorldState,
    entity_id: EntityId,
    selection: &mut SelectionState,
) {
    if let Some(entity) = world.entities.get(&entity_id) {
        selection.target = Some(SelectionTarget {
            cell_x: entity.x,
            cell_y: entity.y,
            card_id: Some(entity_id),
        });
        selection.border_visible = true;
        selection.revision += 1;
    }
}

fn pointer_blocked(cursor: Vec2, layout: &ViewportLayout) -> bool {
    !layout.cursor_in_world_area(cursor)
}

fn find_impact_target(world: &WorldState, gx: u8, gy: u8, source: EntityId) -> Option<EntityId> {
    world
        .entities_at(gx, gy)
        .into_iter()
        .find(|id| *id != source)
}

pub fn handle_pointer_input(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut selection: ResMut<SelectionState>,
    mut drag: ResMut<DragState>,
    mut ghost: ResMut<GhostPlaceMode>,
    mut sim: ResMut<SimWorld>,
    mut interaction: ResMut<InteractionState>,
    mut events: ResMut<SimEventQueue>,
    layout: Res<ViewportLayout>,
    view: Res<WorldView>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        return;
    };

    if pointer_blocked(cursor, &layout) {
        return;
    }

    if let Some((gx, gy)) = grid_from_cursor(cursor, &layout, &view) {
        if buttons.just_pressed(MouseButton::Right) && !drag.dragging && !ghost.dragging {
            if let Some(card_id) = resolve_selection_card(&sim.0, gx, gy) {
                if can_drag_entity(&sim.0, card_id) {
                    ghost.dragging = true;
                    ghost.entity_id = Some(card_id);
                    ghost.origin_x = gx;
                    ghost.origin_y = gy;
                    ghost.preview_cell = Some((gx, gy));
                    let world_pos = grid_to_world_in(&sim.0, gx, gy, card_id.0);
                    if let Some(world_cursor) = cursor_to_world(cursor, &layout, &view) {
                        ghost.cursor_offset = world_cursor - world_pos.truncate();
                    }
                    handle_selection_click(&sim.0, gx, gy, &mut selection);
                    return;
                }
            }
            ghost.preview_cell = Some((gx, gy));
            handle_selection_click(&sim.0, gx, gy, &mut selection);
            return;
        }

        if buttons.just_pressed(MouseButton::Left) && !drag.dragging && !ghost.dragging {
            if try_harvest(&mut sim.0, gx, gy, &mut *events).is_some() {
                handle_selection_click(&sim.0, gx, gy, &mut selection);
                return;
            }
            if let Some(card_id) = resolve_selection_card(&sim.0, gx, gy) {
                if can_drag_entity(&sim.0, card_id) {
                    drag.dragging = true;
                    drag.entity_id = Some(card_id);
                    drag.origin_x = gx;
                    drag.origin_y = gy;
                    drag.hit_target = None;
                    let world_pos = grid_to_world_in(&sim.0, gx, gy, card_id.0);
                    if let Some(world_cursor) = cursor_to_world(cursor, &layout, &view) {
                        drag.cursor_offset = world_cursor - world_pos.truncate();
                    }
                    handle_selection_click(&sim.0, gx, gy, &mut selection);
                    return;
                }
            }
            handle_selection_click(&sim.0, gx, gy, &mut selection);
        }
    }

    if ghost.dragging {
        if let Some((gx, gy)) = grid_from_cursor(cursor, &layout, &view) {
            ghost.preview_cell = Some((gx, gy));
        }
        if buttons.just_released(MouseButton::Right) {
            if let Some(id) = ghost.entity_id.take() {
                ghost.dragging = false;
                if let Some((gx, gy)) = grid_from_cursor(cursor, &layout, &view) {
                    try_ghost_drop(
                        &mut sim.0,
                        id,
                        gx,
                        gy,
                        ghost.origin_x,
                        ghost.origin_y,
                        &*interaction,
                        &mut *events,
                    );
                    handle_selection_click(&sim.0, gx, gy, &mut selection);
                } else {
                    revert_drag(&mut sim.0, id, ghost.origin_x, ghost.origin_y);
                }
            }
            ghost.preview_cell = None;
        }
    }

    if drag.dragging {
        if let Some((gx, gy)) = grid_from_cursor(cursor, &layout, &view) {
            if let Some(source) = drag.entity_id {
                drag.hit_target = find_impact_target(&sim.0, gx, gy, source);
            }
        }
        if buttons.just_released(MouseButton::Left) {
            if let Some(id) = drag.entity_id.take() {
                drag.dragging = false;
                if let Some((gx, gy)) = grid_from_cursor(cursor, &layout, &view) {
                    if let Some(target) = drag.hit_target.take() {
                        if try_impact(&mut sim.0, id, target, &mut *interaction, &mut *events) {
                            handle_selection_click(&sim.0, gx, gy, &mut selection);
                            return;
                        }
                    }
                    let result = try_place_entity(&mut sim.0, id, gx, gy);
                    if result == PlaceResult::Reverted {
                        revert_drag(&mut sim.0, id, drag.origin_x, drag.origin_y);
                    } else {
                        handle_selection_click(&sim.0, gx, gy, &mut selection);
                    }
                } else {
                    revert_drag(&mut sim.0, id, drag.origin_x, drag.origin_y);
                }
            }
        }
    }
}

pub fn update_ghost_follow(
    ghost: Res<GhostPlaceMode>,
    layout: Res<ViewportLayout>,
    view: Res<WorldView>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cards: Query<(&CardVisual, &mut Transform)>,
) {
    if !ghost.dragging {
        return;
    }
    let Some(id) = ghost.entity_id else {
        return;
    };
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Some(world) = cursor_to_world(cursor, &layout, &view) else {
        return;
    };
    let pos = world - ghost.cursor_offset;
    for (card, mut transform) in &mut cards {
        if card.entity_id == id.0 {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

pub fn update_drag_follow(
    drag: Res<DragState>,
    layout: Res<ViewportLayout>,
    view: Res<WorldView>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cards: Query<(&CardVisual, &mut Transform)>,
) {
    if !drag.dragging {
        return;
    }
    let Some(id) = drag.entity_id else {
        return;
    };
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Some(world) = cursor_to_world(cursor, &layout, &view) else {
        return;
    };
    let pos = world - drag.cursor_offset;
    for (card, mut transform) in &mut cards {
        if card.entity_id == id.0 {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
            transform.translation.z = 10.0 + id.0 as f32 * 0.001;
        }
    }
}

pub fn handle_view_zoom(
    mut scroll: EventReader<MouseWheel>,
    mut view: ResMut<WorldView>,
    layout: Res<ViewportLayout>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if pointer_blocked(cursor, &layout) {
        return;
    }

    let mut wheel_dir = 0i32;
    for ev in scroll.read() {
        let y = match ev.unit {
            MouseScrollUnit::Line => ev.y,
            MouseScrollUnit::Pixel => ev.y * 0.1,
        };
        if y > 0.0 {
            wheel_dir += 1;
        } else if y < 0.0 {
            wheel_dir -= 1;
        }
    }
    if wheel_dir != 0 {
        view.zoom_wheel(wheel_dir, cursor, layout.world_area_size());
    }
}

#[derive(Resource, Default)]
pub struct CameraPanState {
    pub panning: bool,
    pub last_cursor: Vec2,
}

pub fn handle_view_pan(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut pan: ResMut<CameraPanState>,
    mut view: ResMut<WorldView>,
    layout: Res<ViewportLayout>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    if buttons.just_pressed(MouseButton::Middle) {
        if !pointer_blocked(cursor, &layout) {
            pan.panning = true;
            pan.last_cursor = cursor;
        }
    }
    if buttons.just_released(MouseButton::Middle) {
        pan.panning = false;
    }

    if pan.panning && cursor != pan.last_cursor {
        let delta = cursor - pan.last_cursor;
        pan.last_cursor = cursor;
        view.pan_by(delta, layout.world_area_size());
    }
}

pub fn apply_camera_zoom(scale: f32) -> f32 {
    WorldView::clamp_zoom(scale)
}
