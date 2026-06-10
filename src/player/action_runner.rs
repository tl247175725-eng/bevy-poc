//! Action runner — maps intention → task FSM.

use crate::interaction::InteractionState;
use crate::sim_events::SimEventQueue;
use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

use super::brain_tags::has_tag;
use super::intention::priority_rank;
use super::state::{tick_cooldowns, PlayerMind};
use super::behavior::sync_player_ecology_state;
use super::survival_tasks::{
    flee_from_threat, force_forage_needed, predator_threat_near, satisfy_hunger,
};
use super::tool_tasks::{advance_knife_task, plan_craft_knife};
use super::shelter_tasks::plan_build_hut;

pub struct ActionRunner;

impl ActionRunner {
    pub fn tick(
        world: &mut WorldState,
        player_id: EntityId,
        mind: &mut PlayerMind,
        delta: f32,
        interaction: &mut InteractionState,
        events: &mut SimEventQueue,
    ) {
        tick_cooldowns(mind, delta);

        if let Some(task) = &mind.task {
            if task.task_type == "makeKnife" {
                advance_knife_task(world, player_id, mind, interaction, events);
                return;
            }
        }

        let (px, py) = world
            .entities
            .get(&player_id)
            .map(|e| (e.x, e.y))
            .unwrap_or((0, 0));
        let predator_near = has_tag(mind, "predator_nearby_unsafe")
            || predator_threat_near(world, player_id, px, py).is_some();

        if predator_near {
            flee_from_threat(world, player_id, mind);
            sync_player_ecology_state(world, player_id, mind);
            return;
        }

        if force_forage_needed(world, player_id, mind) {
            let _ = satisfy_hunger(world, player_id, mind);
        }

        match mind.top_desire.as_str() {
            "craft_knife" if mind.task.is_none() => {
                let _ = plan_craft_knife(world, player_id, mind);
            }
            "forage" => {
                let _ = satisfy_hunger(world, player_id, mind);
            }
            "flee_threat" => {
                let _ = flee_from_threat(world, player_id, mind);
            }
            "build_hut" => {
                let _ = plan_build_hut(world, player_id, mind);
            }
            _ => {}
        }

        sync_player_ecology_state(world, player_id, mind);
    }

    pub fn is_active(mind: &PlayerMind) -> bool {
        mind.task.is_some()
    }
}

pub fn threat_beats_survival_beats_forage(threat: &str, survival: &str, forage: &str) -> bool {
    priority_rank(threat) < priority_rank(survival)
        && priority_rank(survival) < priority_rank(forage)
}
