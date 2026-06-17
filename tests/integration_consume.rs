//! 集成测试：验证 Consume 公理端到端工作

use std::collections::HashMap;
use bevy_poc::card_def::{load_card_defs_with_tags, card_defs_map};
use bevy_poc::need_match::data::{
    NeedKind, NeedState, KnowledgeGraph, KnowledgeEntry, KnowledgeId,
    KnowledgeSource, EffectDescriptor, DecompositionStep, PropertyRequirement, CompareOp,
};
use bevy_poc::world_rules;
use bevy_poc::world_state::WorldState;

#[test]
fn consume_pipeline_end_to_end() {
    world_rules::init_tag_registry();
    let defs = load_card_defs_with_tags(bevy_poc::assets_util::card_defs_path());
    let card_map = card_defs_map(&defs);
    let mut world = WorldState::new(card_map);

    world.spawn("miscanthus", 5, 5);
    world.spawn("sambar_deer", 6, 5);

    let deer_id = world.entities.values()
        .find(|e| e.type_name == "sambar_deer").unwrap().id;

    if let Some(deer) = world.entities.get_mut(&deer_id) {
        deer.needs = vec![NeedState {
            kind: NeedKind::Nutrition, current: 1.0, baseline: 0.3,
            urgency: 1.0, blocked: false, decay_rate: 0.0,
        }];
        let mut entries = HashMap::new();
        entries.insert(KnowledgeId(1), KnowledgeEntry {
            id: KnowledgeId(1), name: "觅食".into(),
            functional_prerequisites: vec![PropertyRequirement {
                property: "has_tag".into(), operator: CompareOp::Present,
                threshold: 0.0, quantity_needed: 1.0, tag_value: Some("plant".into()),
            }],
            decomposition: vec![
                DecompositionStep::Acquire { requirements: vec![PropertyRequirement {
                    property: "has_tag".into(), operator: CompareOp::Present,
                    threshold: 0.0, quantity_needed: 1.0, tag_value: Some("plant".into()),
                }]},
                DecompositionStep::Act { action: "Consume".into(), target: Some("acquired_0".into()) },
            ],
            effects: vec![EffectDescriptor { satisfies: NeedKind::Nutrition, magnitude: 0.8 }],
            source: KnowledgeSource::CommonSense,
        });
        deer.knowledge = KnowledgeGraph { entries, next_id: 2 };
    }
    world.set_causal_mode(true);

    let initial_x = world.entities[&deer_id].x;
    let initial_y = world.entities[&deer_id].y;

    // 运行 tick
    for _ in 0..10 {
        world.tick_once();
        world.drain_pending_events();
    }

    let deer = &world.entities[&deer_id];
    assert!(
        deer.x != initial_x || deer.y != initial_y,
        "鹿应在 10 tick 内移动（初始: {},{}, 最终: {},{}）", initial_x, initial_y, deer.x, deer.y
    );

    let eaten = world.entities.values().filter(|e| e.consumed).count();
    assert!(eaten > 0, "应有植物被消耗（实际: {}）", eaten);
}
