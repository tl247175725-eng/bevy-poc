//! Card-definition static audit — 8 automatic checks (engineering tools).

use bevy_poc::{
    all_capability_cards, card_capabilities, card_color_valid, cap_is_registered, load_card_defs,
    tag_dimension, tag_is_registered,
};
use bevy_poc::assets_util::card_defs_path;

fn defs() -> Vec<bevy_poc::CardDef> {
    load_card_defs(card_defs_path())
}

#[test]
fn audit_01_every_card_has_at_least_one_tag() {
    for def in defs() {
        assert!(
            !def.tags.is_empty(),
            "{} must have at least one tag",
            def.type_name
        );
    }
}

#[test]
fn audit_02_every_card_has_nonzero_rgb_color() {
    for def in defs() {
        assert!(
            card_color_valid(def.color),
            "{} color RGB must not be all zero",
            def.type_name
        );
    }
}

#[test]
fn audit_03_every_tag_in_known_dimension() {
    for def in defs() {
        for tag in &def.tags {
            assert!(
                tag_dimension(tag).is_some(),
                "{} tag `{tag}` has no known dimension",
                def.type_name
            );
        }
    }
}

#[test]
fn audit_04_every_capability_card_has_at_least_one_capability() {
    for name in all_capability_cards() {
        if name == "humus" {
            continue;
        }
        let caps = card_capabilities(name);
        assert!(
            !caps.is_empty(),
            "{name} must have at least one capability in CARD_CAPABILITIES"
        );
    }
    for def in defs() {
        if def.type_name == "humus" {
            continue;
        }
        assert!(
            !card_capabilities(&def.type_name).is_empty(),
            "{} must have capabilities entry",
            def.type_name
        );
    }
}

#[test]
fn audit_05_every_capability_registered_in_cap_zh() {
    for name in all_capability_cards() {
        for cap in card_capabilities(name) {
            assert!(
                cap_is_registered(cap),
                "{name} capability `{cap}` missing from CAP_ZH"
            );
        }
    }
}

#[test]
fn audit_06_every_tag_registered_in_tag_zh() {
    for def in defs() {
        for tag in &def.tags {
            assert!(
                tag_is_registered(tag),
                "{} tag `{tag}` missing from TAG_ZH rules",
                def.type_name
            );
        }
    }
}

#[test]
fn audit_07_every_card_has_display_name() {
    for def in defs() {
        assert!(
            !def.display_name.trim().is_empty(),
            "{} display_name must be non-empty",
            def.type_name
        );
    }
}

#[test]
fn audit_08_every_card_has_icon() {
    for def in defs() {
        assert!(
            !def.icon.trim().is_empty(),
            "{} icon must be non-empty",
            def.type_name
        );
    }
}
