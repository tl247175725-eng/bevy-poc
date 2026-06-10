//! Sprint trail ghosts and dust puffs.

use bevy::prelude::*;

use crate::card_visual::CARD_SPRINT_SLIDE_SPEED;
use crate::render::move_animation::MoveAnimating;
use crate::visual_config::CARD_SIZE;

const TRAIL_TTL: f32 = 0.2;
const DUST_TTL: f32 = 0.3;

#[derive(Component, Default)]
pub struct SprintFxState {
    pub was_sprinting: bool,
}

#[derive(Component)]
pub struct SprintTrail {
    pub ttl: f32,
}

#[derive(Component)]
pub struct SprintDust {
    pub ttl: f32,
    pub start_scale: f32,
}

pub fn sync_sprint_fx(
    time: Res<Time>,
    mut commands: Commands,
    mut trails: Query<(Entity, &mut SprintTrail, &mut Sprite, &mut Transform)>,
    mut dust: Query<(Entity, &mut SprintDust, &mut Sprite, &mut Transform), Without<SprintTrail>>,
    mut cards: Query<(Entity, &Transform, Option<&MoveAnimating>, &mut SprintFxState)>,
) {
    let dt = time.delta_secs();

    for (entity, mut trail, mut sprite, mut transform) in &mut trails {
        trail.ttl -= dt;
        let t = (trail.ttl / TRAIL_TTL).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(0.5 * t);
        if trail.ttl <= 0.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            transform.scale = Vec3::splat(0.9 + (1.0 - t) * 0.05);
        }
    }

    for (entity, mut puff, mut sprite, mut transform) in &mut dust {
        puff.ttl -= dt;
        let t = 1.0 - (puff.ttl / DUST_TTL).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(0.6 * (1.0 - t));
        let scale = puff.start_scale + t * 1.0;
        transform.scale = Vec3::splat(scale);
        if puff.ttl <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }

    for (card, transform, animating, mut state) in &mut cards {
        let sprinting = animating
            .map(|a| a.lerp_speed >= CARD_SPRINT_SLIDE_SPEED - 1.0)
            .unwrap_or(false);
        if sprinting {
            let _pos = transform.translation;
            let size = CARD_SIZE * 0.85;
            for offset in [0.0_f32, -3.0] {
                commands.entity(card).with_children(|parent| {
                    parent.spawn((
                        SprintTrail { ttl: TRAIL_TTL },
                        Sprite {
                            color: Color::srgba(0.85, 0.82, 0.75, 0.5),
                            custom_size: Some(Vec2::splat(size)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, offset, -0.05),
                    ));
                });
            }
            if !state.was_sprinting {
                commands.entity(card).with_children(|parent| {
                    parent.spawn((
                        SprintDust {
                            ttl: DUST_TTL,
                            start_scale: 0.5,
                        },
                        Sprite {
                            color: Color::srgba(0.55, 0.45, 0.32, 0.6),
                            custom_size: Some(Vec2::splat(10.0)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, -CARD_SIZE * 0.35, 0.05),
                    ));
                });
            }
        }
        state.was_sprinting = sprinting;
    }
}
