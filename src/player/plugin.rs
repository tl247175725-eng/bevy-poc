//! Bevy PlayerPlugin — independent from ecosystem EventRegistry.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiSet};

use crate::grid_render::SimWorld;
use crate::interaction::InteractionState;
use crate::sim_events::SimEventQueue;
use crate::spatial_index::EntityId;

use super::needs_manager;
#[derive(Resource, Default)]
pub struct PlayerBrainResource {
    pub display_mind: bool,
}

#[derive(Resource, Default)]
pub struct PlayerMindPanel {
    pub last_goal: String,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerBrainResource>()
            .init_resource::<PlayerMindPanel>()
            .add_systems(
                Update,
                display_player_mind.after(EguiSet::InitContexts),
            );
    }
}

pub fn display_player_mind(
    mut contexts: EguiContexts,
    sim: Res<SimWorld>,
    brain: Res<PlayerBrainResource>,
) {
    if !brain.display_mind {
        return;
    }
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    let player_id = sim
        .0
        .entities
        .values()
        .find(|e| e.type_name == "player")
        .map(|e| e.id);
    let Some(pid) = player_id else {
        return;
    };
    let line = sim
        .0
        .player_minds
        .get(&pid)
        .map(|m| m.mind_display_line())
        .unwrap_or_else(|| "观望中".into());
    egui::Window::new("玩家思维")
        .default_width(220.0)
        .show(ctx, |ui| {
            ui.label(line);
        });
}

pub fn tick_player_in_sim(
    sim: &mut SimWorld,
    interaction: &mut InteractionState,
    events: &mut SimEventQueue,
    delta: f32,
) {
    let ids: Vec<EntityId> = sim
        .0
        .entities
        .values()
        .filter(|e| e.type_name == "player" && !e.is_corpse)
        .map(|e| e.id)
        .collect();
    for id in ids {
        needs_manager::tick_player(&mut sim.0, id, delta, interaction, events);
    }
}

pub fn find_player_id(world: &crate::world_state::WorldState) -> Option<EntityId> {
    world
        .entities
        .values()
        .find(|e| e.type_name == "player")
        .map(|e| e.id)
}
