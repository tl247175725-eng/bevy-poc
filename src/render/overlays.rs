//! Rain overlay, float text FX, hover/ghost rings.

use bevy::prelude::*;

use crate::coords::card_world_pos;
use crate::grid_render::SimWorld;
use crate::game_ui_panel::{spawn_text2d, UiFont};
use crate::sim_clock::SimClock;
use crate::sim_events::WorldFxQueue;
use crate::ui_interaction::{DragState, GhostPlaceMode, SelectionState};
use crate::visual_config::{SELECTION_RING, SELECTION_RING_SIZE, SELECTION_RING_WIDTH};
use crate::world_view::WorldRootEntity;

const RAIN_DROP_COUNT: usize = 120;

#[derive(Component)]
pub struct RainOverlay;

#[derive(Component)]
pub struct HoverRing;

#[derive(Component)]
pub struct GhostPreview;

#[derive(Component)]
pub struct FloatTextFx {
    pub ttl: f32,
}

pub fn setup_fx_layer(mut commands: Commands, root: Res<WorldRootEntity>) {
    commands.entity(root.0).with_children(|world| {
        world.spawn((
            RainOverlay,
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 50.0),
                ..default()
            },
        ));
    });
}

pub fn sync_rain_overlay(
    clock: Res<SimClock>,
    root: Res<WorldRootEntity>,
    rain: Query<Entity, With<RainOverlay>>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    let raining = clock.weather == "下雨";
    for entity in &rain {
        let child_count = children.get(entity).map(|c| c.len()).unwrap_or(0);
        let want = if raining { RAIN_DROP_COUNT } else { 0 };
        if child_count == want {
            continue;
        }
        commands.entity(entity).despawn_descendants();
        if raining {
            let w = crate::visual_config::world_width();
            let h = crate::visual_config::world_height();
            commands.entity(entity).with_children(|parent| {
                for i in 0..RAIN_DROP_COUNT {
                    let x = (i as f32 * 17.3) % w;
                    let y = (i as f32 * 23.7) % h;
                    parent.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(0.4, 0.55, 0.75, 0.35),
                            custom_size: Some(Vec2::new(1.5, 8.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(x, y, 0.0),
                        ..default()
                    });
                }
            });
        }
    }
}

pub fn sync_world_fx(
    time: Res<Time>,
    mut queue: ResMut<WorldFxQueue>,
    font: Res<UiFont>,
    root: Res<WorldRootEntity>,
    mut commands: Commands,
    mut fx: Query<(Entity, &mut FloatTextFx, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut ttl, mut transform) in &mut fx {
        ttl.ttl -= dt;
        transform.translation.y -= 20.0 * dt;
        if ttl.ttl <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }

    for msg in queue.drain() {
        let pos = card_world_pos(msg.x, msg.y, 0, None);
        commands.entity(root.0).with_children(|world| {
            world.spawn((
                FloatTextFx { ttl: 1.2 },
                spawn_text2d(
                    msg.text,
                    &font,
                    14.0,
                    Color::srgba(0.9, 0.2, 0.15, 0.95),
                    Transform::from_translation(pos + Vec3::new(0.0, 10.0, 30.0))
                        .with_scale(Vec3::new(1.0, -1.0, 1.0)),
                ),
            ));
        });
    }
}

pub fn sync_hover_ring(
    mut commands: Commands,
    selection: Res<SelectionState>,
    drag: Res<DragState>,
    sim: Res<SimWorld>,
    root: Res<WorldRootEntity>,
    rings: Query<Entity, With<HoverRing>>,
) {
    for e in &rings {
        commands.entity(e).despawn_recursive();
    }
    if drag.dragging {
        return;
    }
    let Some(target) = &selection.target else {
        return;
    };
    let Some(card_id) = target.card_id else {
        return;
    };
    spawn_ring(
        &mut commands,
        root.0,
        card_world_pos(target.cell_x, target.cell_y, card_id.0, Some(&sim.0)),
        Color::srgba(SELECTION_RING.0, SELECTION_RING.1, SELECTION_RING.2, 0.45),
        HoverRing,
    );
}

pub fn sync_ghost_preview(
    mut commands: Commands,
    ghost: Res<GhostPlaceMode>,
    sim: Res<SimWorld>,
    root: Res<WorldRootEntity>,
    previews: Query<Entity, With<GhostPreview>>,
) {
    for e in &previews {
        commands.entity(e).despawn_recursive();
    }
    if !ghost.dragging {
        return;
    }
    let Some((gx, gy)) = ghost.preview_cell else {
        return;
    };
    spawn_ring(
        &mut commands,
        root.0,
        card_world_pos(gx, gy, 0, Some(&sim.0)),
        Color::srgba(0.3, 0.5, 0.9, 0.6),
        GhostPreview,
    );
}

fn spawn_ring(
    commands: &mut Commands,
    root: Entity,
    center: Vec3,
    color: Color,
    marker: impl Bundle,
) {
    let size = SELECTION_RING_SIZE;
    let hw = size / 2.0;
    let t = SELECTION_RING_WIDTH;
    commands.entity(root).with_children(|world| {
        world
            .spawn((
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(center.x, center.y, 25.0)),
                    ..default()
                },
                marker,
            ))
            .with_children(|parent| {
                for (offset, sz) in [
                    (Vec2::new(0.0, hw - t / 2.0), Vec2::new(size, t)),
                    (Vec2::new(0.0, -hw + t / 2.0), Vec2::new(size, t)),
                    (Vec2::new(-hw + t / 2.0, 0.0), Vec2::new(t, size)),
                    (Vec2::new(hw - t / 2.0, 0.0), Vec2::new(t, size)),
                ] {
                    parent.spawn(SpriteBundle {
                        sprite: Sprite {
                            color,
                            custom_size: Some(sz),
                            ..default()
                        },
                        transform: Transform::from_translation(offset.extend(0.0)),
                        ..default()
                    });
                }
            });
    });
}
