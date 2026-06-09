//! Grid slide animations — orthogonal steps, speed from `move_speed` tags.

use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{
    lens::TransformPositionLens, Animator, EaseFunction, Tween, TweenCompleted,
};

use crate::card_visual::{stack_indices, CardVisual};
use crate::coords::card_world_pos;
use crate::grid_render::SimWorld;
use crate::sim_events::{
    MoveAnimEvent, MoveAnimPlayback, MoveAnimationsComplete,
};
use crate::spatial_index::EntityId;
use crate::visual_config::STACK_OFFSET_Y;

pub const MOVE_ANIM_COMPLETED: u64 = 0x4D4F5645; // "MOVE"

#[derive(Component)]
pub struct MoveAnimating;

pub fn process_move_queue(
    mut commands: Commands,
    mut move_events: EventReader<MoveAnimEvent>,
    mut playback: ResMut<MoveAnimPlayback>,
    sim: Res<SimWorld>,
    card_visuals: Query<(Entity, &CardVisual, &Transform)>,
) {
    let stacks = stack_indices(&sim.0);
    let mut batch_count = 0u32;

    for event in move_events.read() {
        let Some(bevy_entity) = find_bevy_entity(&card_visuals, event.entity_id) else {
            continue;
        };
        let Ok((_, _, transform)) = card_visuals.get(bevy_entity) else {
            continue;
        };

        let start_pos = transform.translation;
        let end_pos = visual_pos(
            event.to_x,
            event.to_y,
            event.entity_id.0,
            &stacks,
            &sim.0,
        );
        let dur = Duration::from_secs_f32(event.duration_per_step);

        let dx = (event.to_x as i16 - event.from_x as i16).abs();
        let dy = (event.to_y as i16 - event.from_y as i16).abs();

        let tweenable = if dx > 0 && dy > 0 {
            let step1_target = visual_pos(
                event.to_x,
                event.from_y,
                event.entity_id.0,
                &stacks,
                &sim.0,
            );
            let tween1 = Tween::new(
                EaseFunction::CubicOut,
                dur,
                TransformPositionLens {
                    start: start_pos,
                    end: step1_target,
                },
            );
            let tween2 = Tween::new(
                EaseFunction::CubicOut,
                dur,
                TransformPositionLens {
                    start: step1_target,
                    end: end_pos,
                },
            )
            .with_completed_event(MOVE_ANIM_COMPLETED);
            tween1.then(tween2)
        } else {
            Tween::new(
                EaseFunction::CubicOut,
                dur,
                TransformPositionLens {
                    start: start_pos,
                    end: end_pos,
                },
            )
            .with_completed_event(MOVE_ANIM_COMPLETED)
        };

        commands
            .entity(bevy_entity)
            .insert((Animator::new(tweenable), MoveAnimating));
        batch_count += 1;
    }

    if batch_count > 0 {
        playback.begin_batch(batch_count);
    }
}

pub fn on_move_anim_completed(
    mut reader: EventReader<TweenCompleted>,
    mut complete_writer: EventWriter<MoveAnimationsComplete>,
    mut playback: ResMut<MoveAnimPlayback>,
    mut commands: Commands,
) {
    for event in reader.read() {
        if event.user_data != MOVE_ANIM_COMPLETED {
            continue;
        }
        commands.entity(event.entity).remove::<MoveAnimating>();
        commands.entity(event.entity).remove::<Animator<Transform>>();
        if playback.note_completion() {
            complete_writer.send(MoveAnimationsComplete);
        }
    }
}

fn find_bevy_entity(
    card_visuals: &Query<(Entity, &CardVisual, &Transform)>,
    entity_id: EntityId,
) -> Option<Entity> {
    card_visuals
        .iter()
        .find(|(_, cv, _)| cv.entity_id == entity_id.0)
        .map(|(e, _, _)| e)
}

fn visual_pos(
    x: u8,
    y: u8,
    entity_id: u64,
    stacks: &std::collections::HashMap<(u8, u8), Vec<u64>>,
    world: &crate::world_state::WorldState,
) -> Vec3 {
    let stack_index = stacks
        .get(&(x, y))
        .and_then(|list| list.iter().position(|id| *id == entity_id))
        .unwrap_or(0) as f32;
    let mut pos = card_world_pos(x, y, entity_id, Some(world));
    pos.y += stack_index * STACK_OFFSET_Y;
    pos
}
