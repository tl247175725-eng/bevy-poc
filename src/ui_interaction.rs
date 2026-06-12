use crate::card_visual::CardVisual;
use crate::coords::{card_world_pos, cursor_to_world, grid_from_cursor};
use crate::grid_render::SimWorld;
use crate::interaction::{try_ghost_drop, try_harvest, try_impact, InteractionState, SmashOutcome};
use crate::render::smash_visual::{bump_smash_shake, clear_smash_session, SmashVisualState};
use crate::visual_config::CARD_SIZE;
use crate::selection_info::{resolve_selection_card, SelectionTarget};
use crate::sim_events::SimEventQueue;
use crate::spatial_index::EntityId;
use crate::terrain::is_blocked_terrain;
use crate::terrain::terrain_at;
use crate::viewport_layout::ViewportLayout;
use crate::world_rules::card_has_tag;
use crate::world_state::{MoveResult, WorldState};
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
    /// While dragging: last target we smashed — must pull away before another hit.
    pub smash_contact: Option<EntityId>,
    pub smash_armed: bool,
    /// Target center to repulse from after a smash (edge clamp + brief bounce).
    pub smash_repulse_from: Option<Vec2>,
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
        if card_has_tag(def, "cell.overlay") {
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

    if !world.cell_composition.can_occupy(x, y, &entity.profile) {
        return PlaceResult::Reverted;
    }
    if world.move_entity(id, x, y) != MoveResult::Moved {
        return PlaceResult::Reverted;
    }
    if let Some(e) = world.entities.get_mut(&id) {
        e.hidden_in_grass = false;
        e.in_cover = false;
        e.host_cover_id = None;
    }
    world.reindex_entity(id);
    PlaceResult::Moved
}

pub fn revert_drag(world: &mut WorldState, id: EntityId, origin_x: u8, origin_y: u8) {
    if world.move_entity(id, origin_x, origin_y) != MoveResult::Moved {
        if let Some((nx, ny)) = crate::systems::movement::find_safe_land_near(world, origin_x, origin_y)
        {
            let _ = world.move_entity(id, nx, ny);
        }
    }
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
                    if let Some(entity) = sim.0.entities.get(&card_id) {
                        ghost.origin_x = entity.x;
                        ghost.origin_y = entity.y;
                        ghost.preview_cell = Some((entity.x, entity.y));
                        let world_pos = card_world_pos(entity.x, entity.y, card_id.0, Some(&sim.0));
                        if let Some(world_cursor) = cursor_to_world(cursor, &layout, &view) {
                            ghost.cursor_offset = world_cursor - world_pos.truncate();
                        }
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
                    drag.hit_target = None;
                    drag.smash_contact = None;
                    drag.smash_armed = false;
                    drag.smash_repulse_from = None;
                    if let Some(entity) = sim.0.entities.get(&card_id) {
                        drag.origin_x = entity.x;
                        drag.origin_y = entity.y;
                        let world_pos = card_world_pos(entity.x, entity.y, card_id.0, Some(&sim.0));
                        if let Some(world_cursor) = cursor_to_world(cursor, &layout, &view) {
                            drag.cursor_offset = world_cursor - world_pos.truncate();
                        }
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
            let smashed_this_drag = drag.smash_armed;
            if let Some(id) = drag.entity_id.take() {
                drag.dragging = false;
                drag.smash_contact = None;
                drag.smash_armed = false;
                drag.smash_repulse_from = None;
                if let Some((gx, gy)) = grid_from_cursor(cursor, &layout, &view) {
                    if let Some(target) = drag.hit_target.take() {
                        if !smashed_this_drag
                            && try_impact(&mut sim.0, id, target, &mut *interaction, &mut *events)
                        {
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
    mut cards: Query<(&mut CardVisual, &mut Transform)>,
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
    for (mut card, mut transform) in &mut cards {
        if card.entity_id == id.0 {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
            card.visual_pos = transform.translation;
            card.target_pos = transform.translation;
        }
    }
}

const SMASH_CONTACT_DIST: f32 = CARD_SIZE * 0.8;

fn clamp_outside_smash_radius(pos: Vec2, center: Vec2) -> Vec2 {
    let delta = pos - center;
    let dist = delta.length();
    if dist >= SMASH_CONTACT_DIST {
        return pos;
    }
    if dist > 0.001 {
        center + delta / dist * SMASH_CONTACT_DIST
    } else {
        center + Vec2::X * SMASH_CONTACT_DIST
    }
}

fn clamp_drag_pos(
    pos: Vec2,
    repulse_from: Option<Vec2>,
    other_centers: impl Iterator<Item = Vec2>,
) -> Vec2 {
    let mut clamped = pos;
    if let Some(center) = repulse_from {
        clamped = clamp_outside_smash_radius(clamped, center);
    }
    for center in other_centers {
        if clamped.distance(center) <= SMASH_CONTACT_DIST {
            clamped = clamp_outside_smash_radius(clamped, center);
        }
    }
    clamped
}

pub fn detect_drag_smash(
    mut drag: ResMut<DragState>,
    mut sim: ResMut<SimWorld>,
    mut interaction: ResMut<InteractionState>,
    mut events: ResMut<SimEventQueue>,
    mut smash_vis: ResMut<SmashVisualState>,
    mut cards: Query<(&CardVisual, &mut Transform)>,
) {
    if !drag.dragging {
        clear_smash_session(&mut smash_vis);
        return;
    }
    let Some(source_id) = drag.entity_id else {
        return;
    };

    let source_pos = cards
        .iter()
        .find(|(cv, _)| cv.entity_id == source_id.0)
        .map(|(_, t)| t.translation.truncate());

    let Some(source_pos) = source_pos else {
        return;
    };

    let mut nearest: Option<(EntityId, f32, Vec2)> = None;
    for (cv, transform) in &cards {
        if cv.entity_id == source_id.0 {
            continue;
        }
        if sim
            .0
            .entities
            .get(&EntityId(cv.entity_id))
            .is_some_and(|e| e.in_cover)
        {
            continue;
        }
        let target_pos = transform.translation.truncate();
        let dist = source_pos.distance(target_pos);
        if dist <= SMASH_CONTACT_DIST {
            let id = EntityId(cv.entity_id);
            if nearest.is_none_or(|(_, best, _)| dist < best) {
                nearest = Some((id, dist, target_pos));
            }
        }
    }

    let Some((target_id, _, target_pos)) = nearest else {
        drag.smash_contact = None;
        drag.smash_armed = false;
        drag.smash_repulse_from = None;
        clear_smash_session(&mut smash_vis);
        return;
    };

    if drag.smash_contact != Some(target_id) {
        drag.smash_contact = Some(target_id);
        drag.smash_armed = false;
    }

    if drag.smash_armed {
        let count = interaction
            .hit_counts
            .get(&(source_id, target_id))
            .copied()
            .unwrap_or(0);
        if count > 0 {
            bump_smash_shake(&mut smash_vis, target_id, count);
        }
        return;
    }

    let outcome = crate::interaction::apply_smash_hit(
        &mut sim.0,
        source_id,
        target_id,
        &mut interaction,
        &mut events,
    );
    if outcome != SmashOutcome::NoEffect {
        drag.smash_armed = true;
        drag.smash_repulse_from = Some(target_pos);
        let count = interaction
            .hit_counts
            .get(&(source_id, target_id))
            .copied()
            .unwrap_or(1);
        bump_smash_shake(&mut smash_vis, target_id, count);
        let bounced = clamp_outside_smash_radius(source_pos, target_pos);
        for (cv, mut transform) in &mut cards {
            if cv.entity_id == source_id.0 {
                transform.translation.x = bounced.x;
                transform.translation.y = bounced.y;
                transform.translation.z = 10.0 + source_id.0 as f32 * 0.001;
            }
        }
    }
}

pub fn update_drag_follow(
    drag: Res<DragState>,
    layout: Res<ViewportLayout>,
    view: Res<WorldView>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cards: Query<(&mut CardVisual, &mut Transform)>,
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
    let other_centers: Vec<Vec2> = cards
        .iter()
        .filter(|(cv, _)| cv.entity_id != id.0)
        .map(|(_, t)| t.translation.truncate())
        .collect();
    let pos = clamp_drag_pos(
        world - drag.cursor_offset,
        drag.smash_repulse_from,
        other_centers.into_iter(),
    );
    for (mut card, mut transform) in &mut cards {
        if card.entity_id == id.0 {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
            transform.translation.z = 10.0 + id.0 as f32 * 0.001;
            card.visual_pos = transform.translation;
            card.target_pos = transform.translation;
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
