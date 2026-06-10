//! Grid slide animations — lerp `CardVisual::visual_pos` toward `target_pos`.

use bevy::prelude::*;

use crate::card_visual::{CardVisual, CARD_SLIDE_SPEED, CARD_SPRINT_SLIDE_SPEED};
use crate::sim_events::{MoveAnimEvent, MoveAnimPlayback, MoveAnimationsComplete};
use crate::spatial_index::EntityId;

const ARRIVED_THRESHOLD: f32 = 0.1;

#[derive(Component)]
pub struct MoveAnimating {
    pub lerp_speed: f32,
}

pub fn lerp_speed_for_step(duration_per_step: f32) -> f32 {
    if duration_per_step < 0.2 {
        CARD_SPRINT_SLIDE_SPEED
    } else {
        CARD_SLIDE_SPEED
    }
}

pub fn process_move_queue(
    mut commands: Commands,
    mut move_events: EventReader<MoveAnimEvent>,
    mut playback: ResMut<MoveAnimPlayback>,
    card_visuals: Query<(Entity, &CardVisual)>,
) {
    let mut batch_count = 0u32;

    for event in move_events.read() {
        let Some(bevy_entity) = find_bevy_entity(&card_visuals, event.entity_id) else {
            continue;
        };
        commands.entity(bevy_entity).insert(MoveAnimating {
            lerp_speed: lerp_speed_for_step(event.duration_per_step),
        });
        batch_count += 1;
    }

    if batch_count > 0 {
        playback.begin_batch(batch_count);
    }
}

pub fn check_move_anim_completion(
    mut commands: Commands,
    mut playback: ResMut<MoveAnimPlayback>,
    mut complete_writer: EventWriter<MoveAnimationsComplete>,
    cards: Query<(Entity, &CardVisual, Option<&MoveAnimating>)>,
) {
    if !playback.in_progress {
        return;
    }
    for (entity, cv, animating) in &cards {
        if animating.is_none() {
            continue;
        }
        if cv.visual_pos.distance(cv.target_pos) >= ARRIVED_THRESHOLD {
            continue;
        }
        commands.entity(entity).remove::<MoveAnimating>();
        if playback.note_completion() {
            complete_writer.send(MoveAnimationsComplete);
        }
    }
}

fn find_bevy_entity(
    card_visuals: &Query<(Entity, &CardVisual)>,
    entity_id: EntityId,
) -> Option<Entity> {
    card_visuals
        .iter()
        .find(|(_, cv)| cv.entity_id == entity_id.0)
        .map(|(e, _)| e)
}
