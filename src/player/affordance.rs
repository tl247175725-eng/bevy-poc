//! Gibson affordances — Godot `player_affordance.gd`.

use std::collections::HashMap;

use crate::capabilities::card_capabilities;
use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

use super::brain_tags::{has_tag, sync_player_tags};
use super::brain_world::check_all;
use super::state::{AffordanceEntry, PlayerMind};

const AFFORDANCE_HOLD_TICKS: u8 = 3;
const DEBOUNCE_KEYS: &[&str] = &["craft_knife", "craft_spear", "forage"];

struct AffordanceRule {
    key: &'static str,
    require_caps: &'static [&'static str],
    require_tags: &'static [&'static str],
    require_not_tags: &'static [&'static str],
    require_world: &'static [&'static str],
    need: &'static str,
    score: i32,
}

const AFFORDANCE_TABLE: &[AffordanceRule] = &[
    AffordanceRule {
        key: "hunt_armed",
        require_caps: &["capability.hunt"],
        require_tags: &["has_weapon"],
        require_not_tags: &[],
        require_world: &["prey_available", "hunt_quota_ok"],
        need: "competence",
        score: 7,
    },
    AffordanceRule {
        key: "hunt_bare",
        require_caps: &["capability.hunt"],
        require_tags: &[],
        require_not_tags: &["has_weapon"],
        require_world: &["prey_available", "hunt_quota_ok"],
        need: "competence",
        score: 2,
    },
    AffordanceRule {
        key: "collect_fuel",
        require_caps: &[],
        require_tags: &["fire_bond"],
        require_not_tags: &[],
        require_world: &["camp_fire_exists", "wood_nearby"],
        need: "relatedness",
        score: 6,
    },
    AffordanceRule {
        key: "craft_knife",
        require_caps: &["capability.craft"],
        require_tags: &[],
        require_not_tags: &[],
        require_world: &["stones_available", "no_knife_owned"],
        need: "competence",
        score: 8,
    },
    AffordanceRule {
        key: "craft_spear",
        require_caps: &["capability.craft"],
        require_tags: &[],
        require_not_tags: &[],
        require_world: &["stone_available", "wood_nearby", "no_spear_owned"],
        need: "competence",
        score: 7,
    },
    AffordanceRule {
        key: "build_hut",
        require_caps: &[],
        require_tags: &["tool_dependent"],
        require_not_tags: &[],
        require_world: &["no_shelter_exists"],
        need: "relatedness",
        score: 9,
    },
    AffordanceRule {
        key: "forage",
        require_caps: &["capability.forage"],
        require_tags: &["food_seek"],
        require_not_tags: &[],
        require_world: &["berry_bush_nearby"],
        need: "hunger",
        score: 4,
    },
];

pub fn compute_affordances(world: &WorldState, player_id: EntityId, mind: &mut PlayerMind) {
    sync_player_tags(world, player_id, mind);
    let raw = detect_raw(world, player_id, mind);
    mind.affordances = apply_debounce(mind, raw);
}

fn detect_raw(
    world: &WorldState,
    player_id: EntityId,
    mind: &PlayerMind,
) -> HashMap<String, AffordanceEntry> {
    let caps = card_capabilities("player");
    let mut affordances = HashMap::new();
    for rule in AFFORDANCE_TABLE {
        if !check_caps(caps, rule.require_caps) {
            continue;
        }
        if !check_tags(mind, rule.require_tags) {
            continue;
        }
        if !check_not_tags(mind, rule.require_not_tags) {
            continue;
        }
        if !check_all(world, player_id, mind, rule.require_world) {
            continue;
        }
        affordances.insert(
            rule.key.to_string(),
            AffordanceEntry {
                key: rule.key.to_string(),
                need: rule.need.to_string(),
                score: rule.score,
            },
        );
    }
    affordances
}

fn apply_debounce(
    mind: &mut PlayerMind,
    raw: HashMap<String, AffordanceEntry>,
) -> HashMap<String, AffordanceEntry> {
    let hold = mind.affordance_hold.clone();
    let mut next_hold = HashMap::new();
    let mut merged = raw.clone();

    for (key, entry) in &raw {
        mind.affordance_last.insert(key.clone(), entry.clone());
        if DEBOUNCE_KEYS.contains(&key.as_str()) {
            next_hold.insert(key.clone(), AFFORDANCE_HOLD_TICKS);
        }
    }

    for key in DEBOUNCE_KEYS {
        if raw.contains_key(*key) {
            continue;
        }
        let rem = hold.get(*key).copied().unwrap_or(0);
        if rem == 0 {
            continue;
        }
        next_hold.insert(key.to_string(), rem - 1);
        if let Some(last) = mind.affordance_last.get(*key) {
            merged.insert(key.to_string(), last.clone());
        } else if let Some(entry) = entry_from_table(key) {
            merged.insert(key.to_string(), entry);
        }
    }

    mind.affordance_hold = next_hold;
    merged
}

fn entry_from_table(key: &str) -> Option<AffordanceEntry> {
    AFFORDANCE_TABLE
        .iter()
        .find(|r| r.key == key)
        .map(|r| AffordanceEntry {
            key: r.key.to_string(),
            need: r.need.to_string(),
            score: r.score,
        })
}

fn check_caps(caps: &[&str], required: &[&str]) -> bool {
    required.iter().all(|c| caps.contains(c))
}

fn check_tags(mind: &PlayerMind, required: &[&str]) -> bool {
    required.iter().all(|t| has_tag(mind, t))
}

fn check_not_tags(mind: &PlayerMind, forbidden: &[&str]) -> bool {
    forbidden.iter().all(|t| !has_tag(mind, t))
}

pub fn allows_live_hunt(mind: &PlayerMind) -> bool {
    mind.affordances.contains_key("hunt_armed") || mind.affordances.contains_key("hunt_bare")
}
