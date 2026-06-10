//! Smash / stack interaction badges and target shake.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::card_visual::CardVisual;
use crate::grid_render::SimWorld;
use crate::panel_ui::UiFont;
use crate::spatial_index::EntityId;
use crate::ui_interaction::DragState;
use crate::visual_config::CARD_SIZE;

const SHAKE_FRAMES: u8 = 5;
const SHAKE_AMPLITUDE: f32 = 2.0;

#[derive(Resource, Default)]
pub struct SmashVisualState {
    pub badge_count: u32,
    pub target_id: Option<u64>,
    pub shake_frames: HashMap<u64, u8>,
}

#[derive(Component)]
pub struct SmashBadge;

#[derive(Component)]
pub struct SmashBadgeText;

pub fn bump_smash_shake(state: &mut SmashVisualState, target: EntityId, count: u32) {
    state.target_id = Some(target.0);
    state.badge_count = count;
    state.shake_frames.insert(target.0, SHAKE_FRAMES);
}

pub fn clear_smash_session(state: &mut SmashVisualState) {
    state.target_id = None;
    state.badge_count = 0;
}

pub fn sync_smash_badges(
    mut commands: Commands,
    mut state: ResMut<SmashVisualState>,
    font: Res<UiFont>,
    sim: Res<SimWorld>,
    drag: Res<DragState>,
    badges: Query<Entity, With<SmashBadge>>,
    mut cards: Query<(Entity, &CardVisual, &mut Transform)>,
) {
    for e in &badges {
        commands.entity(e).despawn_recursive();
    }

    for (_, frames) in state.shake_frames.iter_mut() {
        *frames = frames.saturating_sub(1);
    }
    state.shake_frames.retain(|_, f| *f > 0);

    for (entity, cv, mut transform) in &mut cards {
        if let Some(frames) = state.shake_frames.get(&cv.entity_id).copied() {
            if frames > 0 {
                let phase = (SHAKE_FRAMES - frames) as f32 * 30.0;
                transform.translation.x += phase.sin() * SHAKE_AMPLITUDE;
            }
        }

        if state.target_id == Some(cv.entity_id) && state.badge_count > 0 && drag.dragging {
            if sim.0.entities.contains_key(&EntityId(cv.entity_id)) {
                spawn_smash_badge(&mut commands, entity, &font, state.badge_count);
            }
        }
    }
}

fn spawn_smash_badge(commands: &mut Commands, card: Entity, font: &UiFont, count: u32) {
    let label = if count > 1 {
        format!("砸{count}")
    } else {
        "砸".into()
    };
    let hw = CARD_SIZE * 0.5;
    commands.entity(card).with_children(|parent| {
        parent
            .spawn((
                SmashBadge,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(0.85, 0.15, 0.12, 0.92),
                        custom_size: Some(Vec2::splat(22.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(hw - 6.0, hw - 6.0, 0.5),
                    ..default()
                },
            ))
            .with_children(|badge| {
                badge.spawn((
                    SmashBadgeText,
                    Text2dBundle {
                        text: Text::from_section(
                            label,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 11.0,
                                color: Color::srgba(1.0, 1.0, 1.0, 0.95),
                            },
                        ),
                        transform: Transform::from_xyz(0.0, 0.0, 0.1)
                            .with_scale(Vec3::new(1.0, -1.0, 1.0)),
                        ..default()
                    },
                ));
            });
    });
}
