//! 集成测试：验证 Consume 公理端到端工作
//! 用小世界验证：TagRegistry → 生成 → 需求 → 搜索 → 执行 → Consume → 状态更新

use bevy_poc::card_def::{load_card_defs_with_tags, card_defs_map};
use bevy_poc::need_match::data::NeedKind;
use bevy_poc::world_rules;
use bevy_poc::world_state::WorldState;

/// 构建仅含 1 食草动物 + 1 植物的最小世界
fn tiny_world() -> WorldState {
    world_rules::init_tag_registry();
    let defs = load_card_defs_with_tags(bevy_poc::assets_util::card_defs_path());
    let card_map = card_defs_map(&defs);
    let mut world = WorldState::new(card_map);
    // 放一株草在 (5,5)，一只鹿在 (6,5)
    world.spawn("miscanthus", 5, 5);
    world.spawn("sambar_deer", 6, 5);
    // 手动初始化鹿的需求和知识
    if let Some(deer_def) = world.card_defs.get("sambar_deer").cloned() {
        if let Some(deer) = world.entities.values_mut().find(|e| e.type_name == "sambar_deer") {
            deer.needs = bevy_poc::initial_spawn::init_animal_needs_public(&deer_def);
            deer.knowledge = bevy_poc::initial_spawn::init_animal_knowledge_public(&deer_def);
            // 确保营养需求已激活（设为高紧迫度）
            for need in &mut deer.needs {
                if matches!(need.kind, NeedKind::Nutrition) {
                    need.current = 0.9; // 非常饿
                    need.urgency = 0.9;
                }
            }
        }
    }
    world.set_causal_mode(true);
    world
}

#[test]
#[ignore = "已知管线不通——handoff 039修"]
fn consume_pipeline_end_to_end() {
    let mut world = tiny_world();

    let deer_id = world.entities.values()
        .find(|e| e.type_name == "sambar_deer")
        .expect("应有鹿")
        .id;

    let initial_x = world.entities[&deer_id].x;
    let initial_y = world.entities[&deer_id].y;

    // 运行 50 tick
    for _ in 0..50 {
        world.tick_once();
        world.drain_pending_events();
    }

    let deer = &world.entities[&deer_id];

    // 断言：鹿移动了（Acquire + Move 管道工作）
    assert!(
        deer.x != initial_x || deer.y != initial_y,
        "鹿应在 50 tick 内移动（初始: {initial_x},{initial_y} 最终: {},{}）",
        deer.x, deer.y
    );

    // 断言：有植物被消耗
    let eaten = world.entities.values().filter(|e| e.consumed).count();
    assert!(eaten > 0, "至少应有 1 株植物被消耗（实际: {eaten}）");
}
