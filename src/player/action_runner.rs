//! Action runner — maps intention → task FSM.

use crate::interaction::InteractionState;
use crate::sim_events::SimEventQueue;
use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

use super::brain_tags::has_tag;
use super::intention::priority_rank;
use super::state::{tick_cooldowns, PlayerMind};
use super::survival_tasks;
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

        if has_tag(mind, "predator_nearby_unsafe") {
            mind.state_label = "躲避威胁".into();
            return;
        }

        match mind.top_desire.as_str() {
            "craft_knife" if mind.task.is_none() => {
                let _ = plan_craft_knife(world, player_id, mind);
            }
            "forage" => {
                let _ = survival_tasks::satisfy_hunger(world, player_id, mind);
            }
            "build_hut" => {
                let _ = plan_build_hut(world, player_id, mind);
            }
            _ => {}
        }
    }

    pub fn is_active(mind: &PlayerMind) -> bool {
        mind.task.is_some()
    }
}

pub fn threat_beats_survival_beats_forage(threat: &str, survival: &str, forage: &str) -> bool {
    priority_rank(threat) < priority_rank(survival)
        && priority_rank(survival) < priority_rank(forage)
}
