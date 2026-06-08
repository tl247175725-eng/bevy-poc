//! Card drag/ghost overlays — Godot `card_base.gd` frame modes.

use bevy::prelude::*;

use crate::card_visual::{CardVisual, stack_indices};
use crate::grid_render::SimWorld;
use crate::ui_interaction::{DragState, GhostPlaceMode};
use crate::visual_config::{CARD_BORDER_PAD, CARD_SIZE};

const DRAG_Z: f32 = 100.0;

#[derive(Component)]
pub struct CardDragOutline;

pub fn sync_card_overlays(
    sim: Res<SimWorld>,
    drag: Res<DragState>,
    ghost: Res<GhostPlaceMode>,
    mut commands: Commands,
    outlines: Query<Entity, With<CardDragOutline>>,
    mut cards: Query<(Entity, &CardVisual, &mut Transform, &Children)>,
    mut sprites: Query<&mut Sprite>,
) {
    for e in &outlines {
        commands.entity(e).despawn_recursive();
    }

    let stacks = stack_indices(&sim.0);
    for (entity, visual, mut transform, children) in &mut cards {
        let Some(sim_entity) = sim
            .0
            .entities
            .get(&crate::spatial_index::EntityId(visual.entity_id))
        else {
            continue;
        };
        let base_z = 10.0 + visual.entity_id as f32 * 0.001;
        let stack_boost = stacks
            .get(&(sim_entity.x, sim_entity.y))
            .and_then(|list| list.iter().position(|id| *id == visual.entity_id))
            .unwrap_or(0) as f32
            * 0.01;

        let is_left_drag = drag.dragging && drag.entity_id.map(|id| id.0) == Some(visual.entity_id);
        let is_ghost_drag =
            ghost.dragging && ghost.entity_id.map(|id| id.0) == Some(visual.entity_id);

        if is_left_drag || is_ghost_drag {
            transform.translation.z = DRAG_Z;
        } else {
            transform.translation.z = base_z + stack_boost;
        }

        for child in children.iter() {
            if let Ok(mut sprite) = sprites.get_mut(*child) {
                let a = if is_ghost_drag {
                    0.55
                } else if is_left_drag {
                    0.92
                } else {
                    1.0
                };
                sprite.color = sprite.color.with_alpha(a);
            }
        }

        if is_left_drag {
            spawn_outline(&mut commands, entity, Color::srgba(0.9, 0.35, 0.15, 0.85));
        } else if is_ghost_drag {
            spawn_outline(&mut commands, entity, Color::srgba(0.3, 0.55, 0.95, 0.75));
        }
    }
}

fn spawn_outline(commands: &mut Commands, card: Entity, color: Color) {
    let size = CARD_SIZE + CARD_BORDER_PAD * 2.0 + 4.0;
    commands.entity(card).with_children(|parent| {
        parent.spawn((
            CardDragOutline,
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, -0.02),
                ..default()
            },
        ));
    });
}
