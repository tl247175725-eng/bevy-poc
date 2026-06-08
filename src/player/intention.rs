//! BDI intention layer — Godot `player_intention.gd`.

use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

use super::affordance::allows_live_hunt;
use super::brain_tags::has_tag;
use super::state::{desire_label, AffordanceEntry, PlayerMind, SdtNeeds};

#[derive(Debug, Clone)]
pub struct Desire {
    pub key: String,
    pub urgency: i32,
    pub need: String,
    pub score: i32,
}

pub fn generate_desires(mind: &PlayerMind) -> Vec<Desire> {
    let mut desires: Vec<Desire> = mind
        .affordances
        .values()
        .filter_map(|aff| {
            let need_key = aff.need.as_str();
            if need_key.is_empty() {
                return None;
            }
            let urgency = if need_key == "hunger" {
                if has_tag(mind, "hungry") {
                    95
                } else {
                    55
                }
            } else if let Some(v) = need_value(&mind.needs, need_key) {
                100 - v
            } else {
                return None;
            };
            Some(Desire {
                key: aff.key.clone(),
                urgency: urgency + aff.score,
                need: need_key.to_string(),
                score: aff.score,
            })
        })
        .collect();
    desires.sort_by(|a, b| b.urgency.cmp(&a.urgency));
    desires
}

fn need_value(needs: &SdtNeeds, key: &str) -> Option<i32> {
    match key {
        "autonomy" => Some(needs.autonomy),
        "competence" => Some(needs.competence),
        "relatedness" => Some(needs.relatedness),
        _ => None,
    }
}

pub fn select_intention(world: &WorldState, player_id: EntityId, mind: &mut PlayerMind) {
    let desires = generate_desires(mind);
    if has_tag(mind, "predator_nearby_unsafe") {
        mind.top_desire = "flee_threat".into();
        mind.intent_key = "flee_threat".into();
        mind.goal_text = desire_label("flee_threat");
        mind.threat_level = 3;
        return;
    }
    mind.threat_level = 0;

    if !fire_bond_ok(world) && survival_priority_over_forage(mind) {
        mind.top_desire = "relight_fire".into();
        mind.intent_key = "relight_fire".into();
        mind.goal_text = "生火".into();
        return;
    }

    mind.top_desire = desires
        .first()
        .map(|d| d.key.clone())
        .unwrap_or_default();
    mind.intent_key = mind.top_desire.clone();
    if !mind.top_desire.is_empty() {
        mind.goal_text = desire_label(&mind.top_desire);
    }

    let _ = player_id;
    let _ = allows_live_hunt(mind);
}

fn fire_bond_ok(world: &WorldState) -> bool {
    super::brain_tags::fire_bond_satisfied(world)
}

fn survival_priority_over_forage(_mind: &PlayerMind) -> bool {
    true
}

pub fn top_desire_key(mind: &PlayerMind) -> &str {
    &mind.top_desire
}

pub fn priority_rank(key: &str) -> u8 {
    match key {
        "flee_threat" => 0,
        "relight_fire" | "build_hut" => 1,
        "forage" | "craft_knife" | "craft_spear" | "hunt_armed" | "hunt_bare" => 2,
        _ => 3,
    }
}

pub fn desire_from_affordance(aff: &AffordanceEntry, mind: &PlayerMind) -> Desire {
    let urgency = if aff.need == "hunger" && has_tag(mind, "hungry") {
        95 + aff.score
    } else {
        100 - need_value(&mind.needs, &aff.need).unwrap_or(50) + aff.score
    };
    Desire {
        key: aff.key.clone(),
        urgency,
        need: aff.need.clone(),
        score: aff.score,
    }
}
