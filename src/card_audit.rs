//! Static card-definition audit — mirrors Godot `card_rule_audit.gd` checklist.

use crate::capabilities::{all_capability_cards, card_capabilities};
use crate::card_def::{load_card_defs, CardDef};
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

static TAG_DIMENSIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    let identity = [
        "being", "actor", "animal", "human", "herbivore", "carnivore", "predator", "mesopredator",
        "omnivore", "omnivore.small", "grazer", "scavenger", "burrower", "filter_feeder", "aquatic",
        "volant", "elder", "youth", "forager", "worker", "observer", "customer", "autonomous",
        "basic", "smallHerbivore", "smallPrey", "largePrey", "wildPrey", "tiny", "small",
        "underground",
    ];
    for t in identity {
        m.insert(t, "identity");
    }
    let material = [
        "material", "material.lumber", "material.shard", "material.stone", "material.tool_head",
        "wood", "hard", "sharp", "blunt", "tough", "fiber", "fuel", "fuel.fire", "commodity",
        "copper", "cooked", "perishable", "dry", "corpse", "food.edible", "tuber", "floating",
        "sessile", "rooted", "structure", "weapon", "tool", "container", "container.water",
        "businessUnit", "mushroomFarm", "craft", "currency",
    ];
    for t in material {
        m.insert(t, "material_form");
    }
    let relation = [
        "camp.anchor", "camp.fire_bond", "camp.storable", "den", "den.candidate.fox", "fire_bond",
        "taoyuan", "home", "shelter", "animalHome", "forest", "bush", "grass", "water", "heat",
        "anchor", "nest", "foodSource", "berry.source", "nut_producer", "cone_producer",
        "source.lumber", "source.stone", "source.twig", "fertile", "environment", "terrain",
        "barren", "primary_producer",
    ];
    for t in relation {
        m.insert(t, "relation_domain");
    }
    let action = [
        "forager", "scavenger", "migratory", "pack_hunter", "flocking", "cover_user",
    ];
    for t in action {
        m.insert(t, "action");
    }
    let rule_mod = [
        "body.large", "body.medium", "body.small", "body.tiny", "organize.locked", "prolific",
        "juvenile", "tool_dependent", "opportunistic", "cell.overlay",
    ];
    for t in rule_mod {
        m.insert(t, "rule_modifier");
    }
    m
});

const KNOWN_DIMENSIONS: &[&str] = &[
    "identity",
    "material_form",
    "capability",
    "relation_domain",
    "action",
    "rule_modifier",
];

pub fn tag_dimension(tag: &str) -> Option<&'static str> {
    if let Some(dim) = TAG_DIMENSIONS.get(tag) {
        return Some(dim);
    }
    if tag.starts_with("body.") || tag.starts_with("organize.") {
        return Some("rule_modifier");
    }
    if tag.starts_with("material.") || tag.starts_with("fuel.") {
        return Some("material_form");
    }
    if tag.starts_with("camp.")
        || tag.starts_with("den.")
        || tag.starts_with("source.")
        || tag.starts_with("berry.")
    {
        return Some("relation_domain");
    }
    if tag.starts_with("container.") || tag.starts_with("cover.") || tag.starts_with("food.") {
        return Some("material_form");
    }
    if tag.starts_with("capability.") {
        return Some("capability");
    }
    if tag.starts_with("drive:")
        || tag.starts_with("move_speed:")
        || tag.starts_with("sprint:")
        || tag.starts_with("social_structure:")
        || tag.starts_with("flock_")
        || tag.starts_with("herd_")
    {
        return Some("action");
    }
    None
}

pub fn tag_is_registered(tag: &str) -> bool {
    crate::tag_zh::tag_has_zh_mapping(tag)
}

pub fn cap_is_registered(cap: &str) -> bool {
    crate::tag_zh::cap_has_zh_mapping(cap)
}

pub fn card_color_valid(color: (u8, u8, u8, u8)) -> bool {
    let (r, g, b, _) = color;
    (r, g, b) != (0, 0, 0)
}

pub fn audit_defs(defs: &[CardDef]) -> Vec<String> {
    let mut errors = Vec::new();
    for def in defs {
        if def.tags.is_empty() {
            errors.push(format!("{}: no tags", def.type_name));
        }
        if !card_color_valid(def.color) {
            errors.push(format!("{}: color RGB all zero", def.type_name));
        }
        for tag in &def.tags {
            if tag_dimension(tag).is_none() {
                errors.push(format!("{}: unknown tag dimension for `{tag}`", def.type_name));
            }
            if !tag_is_registered(tag) {
                errors.push(format!("{}: unregistered tag `{tag}`", def.type_name));
            }
        }
        if def.display_name.trim().is_empty() {
            errors.push(format!("{}: empty display_name", def.type_name));
        }
        if def.icon.trim().is_empty() {
            errors.push(format!("{}: empty icon", def.type_name));
        }
    }
    for name in all_capability_cards() {
        let caps = card_capabilities(name);
        if name == "humus" {
            continue;
        }
        if caps.is_empty() {
            errors.push(format!("{name}: no capabilities in CARD_CAPABILITIES"));
        }
        for cap in caps {
            if !cap_is_registered(cap) {
                errors.push(format!("{name}: unregistered capability `{cap}`"));
            }
        }
    }
    for def in defs {
        if def.type_name == "humus" {
            continue;
        }
        let caps = card_capabilities(&def.type_name);
        if caps.is_empty() {
            errors.push(format!(
                "{}: card_defs entry missing capabilities",
                def.type_name
            ));
        }
    }
    errors
}

pub fn load_and_audit(path: impl AsRef<Path>) -> Vec<String> {
    audit_defs(&load_card_defs(path))
}

pub fn known_dimensions() -> &'static [&'static str] {
    KNOWN_DIMENSIONS
}
