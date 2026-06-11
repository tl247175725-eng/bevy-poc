//! Player AI — five-layer brain (perception → affordance → needs → intention → execution).

mod action_runner;
mod affordance;
mod behavior;
mod brain_tags;
mod brain_world;
mod intention;
mod needs;
mod needs_manager;
pub mod plugin;
mod shelter_tasks;
mod state;
mod survival_tasks;
mod tool_tasks;

pub use action_runner::{threat_beats_survival_beats_forage, ActionRunner};
pub use affordance::compute_affordances;
pub use behavior::{detect_threat_level, tick_brain};
pub use brain_tags::{fire_bond_satisfied, has_tag, sync_player_tags};
pub use brain_world::{check, check_all, is_neighbor};
pub use intention::{generate_desires, priority_rank, select_intention};
pub use needs::evaluate_needs;
pub use needs_manager::{
    ensure_player_mind, player_mind, player_mind_mut, tick_player, tick_player_world,
};
pub use plugin::{display_player_mind, find_player_id, tick_player_in_sim, PlayerBrainResource, PlayerPlugin};
pub use shelter_tasks::{
    build_hut_affordable, craft_hut_relation, execute_build_hut, has_hut_materials,
    materials_near_player,
};
pub use state::{
    desire_label, task_on_cooldown, AffordanceEntry, PlayerMind, PlayerTask, SdtNeeds, TaskPhase,
};
pub use survival_tasks::{hunger_priority_when_starving, should_forage, should_not_forage_when_full};
pub use tool_tasks::{
    craft_axe_relation, craft_spear_relation, fsm_phase_sequence, knap_stones_to_shard,
    plan_craft_knife,
};
