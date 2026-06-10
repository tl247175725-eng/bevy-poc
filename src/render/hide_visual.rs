//! Purple "藏" badge when prey is concealed in grass or bush.

use bevy::prelude::*;

use crate::card_visual::CardVisual;
use crate::game_ui_panel::{spawn_text2d, UiFont};
use crate::grid_render::SimWorld;
use crate::visual_config::CARD_SIZE;

#[derive(Component)]
pub struct HideBadge;

#[derive(Component)]
pub struct HideBadgeBg;

pub fn sync_hide_badges(
    mut commands: Commands,
    sim: Res<SimWorld>,
    font: Res<UiFont>,
    cards: Query<(Entity, &CardVisual, &Children)>,
    badges: Query<Entity, With<HideBadge>>,
) {
    let mut want: std::collections::HashSet<u64> = std::collections::HashSet::new();
    for entity in sim.0.entities.values() {
        if entity.hidden_in_grass || entity.in_burrow {
            want.insert(entity.id.0);
        }
    }

    for (card, cv, children) in &cards {
        let concealed = want.contains(&cv.entity_id);
        let has_badge = children.iter().any(|c| badges.get(*c).is_ok());
        if concealed && !has_badge {
            let r = CARD_SIZE * 0.25;
            let hw = CARD_SIZE * 0.5;
            commands.entity(card).with_children(|parent| {
                parent.spawn((
                    HideBadge,
                    Sprite {
                        color: Color::srgba(0.55, 0.2, 0.75, 0.92),
                        custom_size: Some(Vec2::splat(r * 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(hw - r * 0.6, hw - r * 0.6, 0.6),
                    HideBadgeBg,
                ));
                parent.spawn((
                    HideBadge,
                    spawn_text2d(
                        "藏",
                        &font,
                        10.0,
                        Color::srgba(1.0, 1.0, 1.0, 0.95),
                        Transform::from_xyz(hw - r * 0.6, hw - r * 0.6, 0.65)
                            .with_scale(Vec3::new(1.0, -1.0, 1.0)),
                    ),
                ));
            });
        } else if !concealed && has_badge {
            for child in children.iter() {
                if badges.get(*child).is_ok() {
                    commands.entity(*child).despawn_recursive();
                }
            }
        }
    }
}
