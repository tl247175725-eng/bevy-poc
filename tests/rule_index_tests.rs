//! RuleIndex — tag_index 与 shared_node 去重断言。

use bevy_poc::rule_index::{rule_index, EcologyAction, SharedCondition};
use bevy_poc::world_rules::{ecosystem_behavior_key, ecosystem_behavior_key_legacy, HUNT_RANGE};
use bevy_poc::world_state::empty_world;

#[test]
fn rule_index_tag_index_lists_predator_rules() {
    let index = rule_index();
    let predator_rules = index.rules_for_tag("predator");
    assert!(!predator_rules.is_empty(), "predator 标签应索引到规则");

    let actions: Vec<EcologyAction> = predator_rules
        .iter()
        .filter_map(|rid| index.rule(*rid).map(|r| r.action))
        .collect();
    assert!(
        actions.contains(&EcologyAction::Hunt),
        "predator 应包含 Hunt 规则"
    );
}

#[test]
fn rule_index_shared_in_range_node_deduped() {
    let index = rule_index();
    let hunt_range = SharedCondition::InRange {
        radius: HUNT_RANGE,
    };
    let graze_range = SharedCondition::InRange { radius: 6 };

    let hunt_nodes: Vec<_> = index
        .shared_nodes
        .iter()
        .filter(|n| n.condition == hunt_range)
        .collect();
    assert_eq!(hunt_nodes.len(), 1, "InRange(hunt) 应合并为单个共享节点");

    let graze_nodes: Vec<_> = index
        .shared_nodes
        .iter()
        .filter(|n| n.condition == graze_range)
        .collect();
    assert_eq!(graze_nodes.len(), 1, "InRange(6) 应合并为单个共享节点");

    let hunt_node = hunt_nodes[0];
    assert!(
        hunt_node.children.len() >= 1,
        "hunt 共享节点应有 prey 子节点"
    );
    assert!(
        !hunt_node.rules.is_empty(),
        "共享节点应挂载规则 id"
    );
}

#[test]
fn ecosystem_behavior_key_dual_track_matches_legacy_on_wolf() {
    let w = empty_world();
    let def = w.card_defs.get("wolf").unwrap();
    assert_eq!(
        ecosystem_behavior_key(def, "wolf"),
        ecosystem_behavior_key_legacy(def, "wolf")
    );
}
