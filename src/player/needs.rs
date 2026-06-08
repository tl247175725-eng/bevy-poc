//! SDT needs layer — Godot `player_needs.gd`.

use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

use super::brain_tags::{has_tag, sync_player_tags};
use super::state::{PlayerMind, SdtNeeds};

struct NeedRule {
    tag: &'static str,
    need: &'static str,
    state: &'static str,
    delta: i32,
}

const NEED_TAG_RULES: &[NeedRule] = &[
    NeedRule {
        tag: "fire_bond",
        need: "autonomy",
        state: "unsatisfied",
        delta: -60,
    },
    NeedRule {
        tag: "predator",
        need: "autonomy",
        state: "nearby",
        delta: -80,
    },
    NeedRule {
        tag: "hungry",
        need: "autonomy",
        state: "active",
        delta: -40,
    },
    NeedRule {
        tag: "hungry",
        need: "competence",
        state: "active",
        delta: -15,
    },
    NeedRule {
        tag: "has_weapon",
        need: "competence",
        state: "false",
        delta: -30,
    },
    NeedRule {
        tag: "has_tools",
        need: "competence",
        state: "false",
        delta: -20,
    },
    NeedRule {
        tag: "fire_bond",
        need: "relatedness",
        state: "unsatisfied",
        delta: -70,
    },
    NeedRule {
        tag: "has_shelter",
        need: "relatedness",
        state: "false",
        delta: -30,
    },
    NeedRule {
        tag: "near_camp",
        need: "relatedness",
        state: "true",
        delta: 20,
    },
];

pub fn evaluate_needs(world: &WorldState, player_id: EntityId, mind: &mut PlayerMind) {
    sync_player_tags(world, player_id, mind);
    let mut needs = SdtNeeds::baseline();
    for rule in NEED_TAG_RULES {
        if tag_state_active(mind, rule.tag, rule.state) {
            match rule.need {
                "autonomy" => needs.autonomy += rule.delta,
                "competence" => needs.competence += rule.delta,
                "relatedness" => needs.relatedness += rule.delta,
                _ => {}
            }
        }
    }
    mind.needs = needs.clamp();
}

fn tag_state_active(mind: &PlayerMind, tag_key: &str, state: &str) -> bool {
    match tag_key {
        "fire_bond" => {
            let satisfied = fire_bond_satisfied_world(mind);
            match state {
                "satisfied" => satisfied,
                "unsatisfied" => !satisfied,
                "weak" => satisfied && !has_tag(mind, "near_camp"),
                _ => false,
            }
        }
        "predator" => match state {
            "nearby" => has_tag(mind, "predator_nearby_unsafe"),
            "distant" => !has_tag(mind, "predator_nearby_unsafe"),
            _ => false,
        },
        "hungry" | "has_weapon" | "has_tools" | "has_shelter" | "near_camp" => {
            let on = has_tag(mind, tag_key);
            match state {
                "active" | "true" => on,
                "inactive" | "false" => !on,
                _ => false,
            }
        }
        _ => false,
    }
}

fn fire_bond_satisfied_world(mind: &PlayerMind) -> bool {
    mind.runtime_tags
        .get("fire_bond_satisfied")
        .copied()
        .unwrap_or(false)
}

pub fn lowest_need_value(needs: &SdtNeeds) -> i32 {
    needs.autonomy.min(needs.competence).min(needs.relatedness)
}

pub fn evaluate_needs_for_test(mind: &mut PlayerMind, fire_exists: bool) {
    let mut needs = SdtNeeds::baseline();
    if !fire_exists {
        needs.autonomy -= 60;
        needs.relatedness -= 70;
    }
    if has_tag(mind, "predator_nearby_unsafe") {
        needs.autonomy -= 80;
    }
    if has_tag(mind, "hungry") {
        needs.autonomy -= 40;
        needs.competence -= 15;
    }
    if has_tag(mind, "near_camp") && fire_exists {
        needs.relatedness += 20;
    }
    mind.needs = needs.clamp();
}

pub fn sync_predator_tag_from_world(world: &WorldState, player_id: EntityId, mind: &mut PlayerMind) {
    sync_player_tags(world, player_id, mind);
    let _ = world;
}
