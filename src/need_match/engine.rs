//! 需求匹配引擎 — 模块串联管道
//!
//! 把 activation / search / execution / social 串联成可调用的单 tick 管道。

use crate::meta_actions::MetaAction;

use super::activation::{apply_safety_block, tick_need};
use super::data::{KnowledgeGraph, NeedState};
use super::execution::{build_plan, tick_execution, ExecutionState};
use super::search::{arbitrate, search_by_needs, MaterialProperties};

/// 每个 tick：更新需求 → 安全阻断 → 匹配 → 执行
///
/// 管道顺序：
/// 1. tick_need + apply_safety_block — 每个 tick 必然执行
/// 2. search_by_needs + arbitrate — 仅在无计划或计划失败时执行
/// 3. tick_execution — 每个 tick 执行
pub fn tick_need_engine(
    needs: &mut [NeedState],
    state: &mut ExecutionState,
    knowledge: &KnowledgeGraph,
    environment: &[(u32, (u8, u8), MaterialProperties)],
    delta: f32,
    entity_pos: (u8, u8),
) -> Option<MetaAction> {
    // 1. 需求衰减 + 计算紧迫度
    for need in needs.iter_mut() {
        tick_need(need, delta);
    }

    // 2. 安全阻断
    apply_safety_block(needs);

    // 3. 如果没有当前计划 → Decide
    if state.intention.is_none() {
        let candidates = search_by_needs(needs, knowledge);
        if let Some(best) = arbitrate(candidates, needs, state.intention_score) {
            *state = build_plan(&best, knowledge);
        }
    }

    // 4. 执行下一步（无步骤时默认 Pause）
    tick_execution(state, environment, entity_pos)
        .or_else(|| Some(MetaAction::Pause { ticks: 1 }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::data::{
        DecompositionStep, EffectDescriptor, KnowledgeEntry, KnowledgeId, KnowledgeSource, NeedKind,
    };
    use std::collections::HashMap;

    fn make_need(kind: NeedKind, current: f32, baseline: f32, urgency: f32, blocked: bool) -> NeedState {
        NeedState {
            kind,
            current,
            baseline,
            urgency,
            blocked,
            decay_rate: 0.5,
        }
    }

    fn make_knowledge_graph() -> KnowledgeGraph {
        let mut entries = HashMap::new();
        entries.insert(
            KnowledgeId(1),
            KnowledgeEntry {
                id: KnowledgeId(1),
                name: "rest".into(),
                functional_prerequisites: vec![],
                decomposition: vec![DecompositionStep::Act {
                    action: "sleep".into(),
                    target: None,
                }],
                effects: vec![EffectDescriptor {
                    satisfies: NeedKind::Rest,
                    magnitude: 0.8,
                }],
                source: KnowledgeSource::CommonSense,
            },
        );
        KnowledgeGraph {
            entries,
            next_id: 2,
        }
    }

    #[test]
    fn tick_need_engine_returns_pause_when_no_candidates() {
        let mut needs = vec![
            make_need(NeedKind::Nutrition, 1.0, 0.5, 0.9, false), // activated but no matching knowledge
        ];
        let mut state = ExecutionState::new();
        let kg = make_knowledge_graph();
        let env: Vec<(u32, (u8, u8), MaterialProperties)> = vec![];

        let action = tick_need_engine(&mut needs, &mut state, &kg, &env, 0.0, (5, 5));

        // No knowledge matches Nutrition → no plan built → returns Pause
        assert!(matches!(action, Some(MetaAction::Pause { .. })));
    }

    #[test]
    fn tick_need_engine_builds_plan_and_ticks() {
        let mut needs = vec![
            make_need(NeedKind::Rest, 1.0, 0.5, 0.9, false), // activated, matches "rest" knowledge
        ];
        let mut state = ExecutionState::new();
        let kg = make_knowledge_graph();
        let env: Vec<(u32, (u8, u8), MaterialProperties)> = vec![];

        let action = tick_need_engine(&mut needs, &mut state, &kg, &env, 0.0, (5, 5));

        // Plan built for "rest" knowledge → Act step "sleep" → Pause
        assert!(matches!(action, Some(MetaAction::Pause { .. })));
        assert!(state.intention.is_some());
    }
}
