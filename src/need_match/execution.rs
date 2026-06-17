//! 执行分解 + 失败处理 — HTN 文献验证的三层容错
//!
//! 依据 design-philosophy-v5.md §3

use crate::meta_actions::MetaAction;
use crate::spatial_index::EntityId;

use super::data::{
    CandidateAction, DecompositionStep, KnowledgeGraph, KnowledgeId, PropertyRequirement,
};
pub use super::search::MaterialProperties;

// ===== 执行状态 =====

/// 执行状态——当前正在执行的计划
#[derive(Debug, Clone)]
pub struct ExecutionState {
    /// 当前意图
    pub intention: Option<KnowledgeId>,
    /// 未执行的步骤
    pub steps: Vec<DecompositionStep>,
    /// 当前步骤索引
    pub step_index: usize,
    /// 原意图的总分（用于仲裁阈值比较）
    pub intention_score: f32,
    /// 已获取的物体
    pub acquired_objects: Vec<AcquiredObject>,
    /// 当前步骤重试次数
    pub retry_count: u32,
}

#[derive(Debug, Clone)]
pub struct AcquiredObject {
    /// 世界中的实体 ID
    pub entity_id: u32,
    /// 该物体的属性
    pub properties: MaterialProperties,
}

impl ExecutionState {
    pub fn new() -> Self {
        Self {
            intention: None,
            steps: Vec::new(),
            step_index: 0,
            intention_score: 0.0,
            acquired_objects: Vec::new(),
            retry_count: 0,
        }
    }

    /// 是否还有剩余步骤
    pub fn has_steps(&self) -> bool {
        self.step_index < self.steps.len()
    }

    /// 当前步骤是否需要执行前复查
    pub fn current_step(&self) -> Option<&DecompositionStep> {
        self.steps.get(self.step_index)
    }
}

/// 最大重试次数——超过后触发 Decide 重跑
pub const MAX_RETRY_PER_STEP: u32 = 3;

// ===== 计划构建 =====

/// 根据 Candidate 构建执行计划
pub fn build_plan(candidate: &CandidateAction, knowledge: &KnowledgeGraph) -> ExecutionState {
    let entry = knowledge.entries.get(&candidate.knowledge_id);
    let steps = entry.map(|e| e.decomposition.clone()).unwrap_or_default();
    let score = candidate.matched_needs.len() as f32 * 0.5
        * candidate.achievability
        * (1.0 - candidate.risk);
    ExecutionState {
        intention: Some(candidate.knowledge_id.clone()),
        steps,
        step_index: 0,
        intention_score: score,
        acquired_objects: Vec::new(),
        retry_count: 0,
    }
}

// ===== Acquire 执行 =====

#[derive(Debug)]
pub enum AcquireError {
    /// 没有任何物体满足需求
    NoMatchingObject,
    /// 物体存在但不可获取（被占用/在封闭空间）
    ObjectUnavailable,
}

/// 执行一个 Acquire 步骤——从环境中查找满足属性需求的物体
pub fn execute_acquire_step(
    properties: &[PropertyRequirement],
    environment_objects: &[(u32, (u8, u8), MaterialProperties)],
    already_acquired: &[AcquiredObject],
) -> Result<Vec<(u32, (u8, u8), MaterialProperties)>, AcquireError> {
    let mut candidates = Vec::new();
    for &(entity_id, pos, ref obj_props) in environment_objects {
        if already_acquired.iter().any(|a| a.entity_id == entity_id) {
            continue;
        }
        if properties.iter().all(|req| obj_props.satisfies(req)) {
            candidates.push((entity_id, pos, obj_props.clone()));
        }
    }
    if candidates.is_empty() {
        Err(AcquireError::NoMatchingObject)
    } else {
        Ok(candidates)
    }
}

// ===== 前提复查 =====

/// 执行前条件复查——物体还在原位置吗？仍满足属性吗？
pub fn verify_prerequisites(
    entity_id: u32,
    required_properties: &[PropertyRequirement],
    current_environment: &[(u32, (u8, u8), MaterialProperties)],
) -> bool {
    current_environment
        .iter()
        .any(|(id, _, props)| *id == entity_id && required_properties.iter().all(|req| props.satisfies(req)))
}

// ===== 主执行循环 =====

/// 从当前位置向目标位置走一步（曼哈顿单步）
fn move_toward(from: (u8, u8), target: (u8, u8)) -> MetaAction {
    let dx = (target.0 as i16 - from.0 as i16).signum();
    let dy = (target.1 as i16 - from.1 as i16).signum();
    if dx.abs() >= dy.abs() && dx != 0 {
        MetaAction::Move { dx, dy: 0 }
    } else if dy != 0 {
        MetaAction::Move { dx: 0, dy }
    } else {
        MetaAction::Pause { ticks: 1 }
    }
}

/// 执行步骤的主循环——返回下一帧应该执行的元动作
///
/// 失败处理逻辑（HTN 文献验证）：
/// 1. 执行前复查前提（物体是否还在）
/// 2. 前提失败 → 从备选列表中找下一个
/// 3. 备选用完 → 重试计数 +1，重新搜索环境
/// 4. 超过重试上限 → 清空计划，返回 None → 调用方应触发 Decide
pub fn tick_execution(
    state: &mut ExecutionState,
    environment: &[(u32, (u8, u8), MaterialProperties)],
    entity_pos: (u8, u8),
) -> Option<MetaAction> {
    if !state.has_steps() {
        return None;
    }

    let step = state.current_step().unwrap().clone();

    match &step {
        DecompositionStep::Acquire { requirements } => {
            // 1. 执行前复查（非首次尝试时）
            if state.retry_count > 0 && !state.acquired_objects.is_empty() {
                let last = state.acquired_objects.last().unwrap();
                if !verify_prerequisites(last.entity_id, requirements, environment) {
                    state.acquired_objects.pop();
                }
            }

            // 2. 搜索候选
            match execute_acquire_step(requirements, environment, &state.acquired_objects) {
                Ok(candidates) => {
                    let (target_id, target_pos, _) = &candidates[0];
                    state.acquired_objects.push(AcquiredObject {
                        entity_id: *target_id,
                        properties: candidates[0].2.clone(),
                    });
                    state.retry_count = 0;
                    state.step_index += 1;
                    Some(move_toward(entity_pos, *target_pos))
                }
                Err(AcquireError::NoMatchingObject) => {
                    state.retry_count += 1;
                    if state.retry_count > MAX_RETRY_PER_STEP {
                        state.intention = None;
                        state.steps.clear();
                        return None;
                    }
                    Some(MetaAction::Pause { ticks: 1 })
                }
                Err(AcquireError::ObjectUnavailable) => {
                    Some(MetaAction::Pause { ticks: 1 })
                }
            }
        }

        DecompositionStep::Act { action, target } => {
            state.step_index += 1;
            state.retry_count = 0;
            // 从 target 字段解析索引（如 "acquired_0" → 第一个已获取物体）
            let acquired_target_id = target
                .as_ref()
                .and_then(|t| {
                    if let Some(idx_str) = t.strip_prefix("acquired_") {
                        idx_str.parse::<usize>().ok()
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    // 无 explicit target 时取第一个已获取物体
                    if !state.acquired_objects.is_empty() {
                        Some(0)
                    } else {
                        None
                    }
                })
                .and_then(|idx| state.acquired_objects.get(idx).map(|o| o.entity_id));
            match action.as_str() {
                "Strike" => Some(MetaAction::Strike {
                    target: EntityId(acquired_target_id.unwrap_or(0) as u64),
                }),
                "Consume" => Some(MetaAction::Consume {
                    target: EntityId(acquired_target_id.unwrap_or(0) as u64),
                }),
                _ => Some(MetaAction::Pause { ticks: 1 }),
            }
        }

        DecompositionStep::Combine { ingredient_indices } => {
            if ingredient_indices.len() >= 2 {
                let target_id = state
                    .acquired_objects
                    .get(ingredient_indices[1])
                    .map(|o| o.entity_id)
                    .unwrap_or(0);
                state.step_index += 1;
                state.retry_count = 0;
                Some(MetaAction::Combine {
                    ingredient: EntityId(target_id as u64),
                })
            } else {
                state.step_index += 1;
                Some(MetaAction::Pause { ticks: 1 })
            }
        }
    }
}

// ===== 状态查询 =====

/// 计划是否已完成
pub fn is_plan_complete(state: &ExecutionState) -> bool {
    state.intention.is_some() && !state.has_steps()
}

/// 计划是否已失败（需要 Decide 重新选择）
pub fn is_plan_failed(state: &ExecutionState) -> bool {
    state.intention.is_none() && state.steps.is_empty()
}

// ===== 测试 =====

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::data::CompareOp;

    fn make_stone_props() -> MaterialProperties {
        MaterialProperties {
            hardness: Some(3.0),
            density: Some(2700.0),
            flammability: None,
            spark_on_strike: true,
            edge_present: false,
            mass_kg: 5.0,
            tags: vec![],
        }
    }

    fn make_stick_props() -> MaterialProperties {
        MaterialProperties {
            hardness: Some(1.0),
            density: Some(700.0),
            flammability: Some(0.8),
            spark_on_strike: false,
            edge_present: false,
            mass_kg: 0.3,
            tags: vec![],
        }
    }

    #[test]
    fn execute_acquire_step_finds_matching_object() {
        let env = vec![(1, (5, 5), make_stone_props()), (2, (8, 5), make_stick_props())];
        let reqs = vec![PropertyRequirement {
            property: "hardness".into(),
            operator: CompareOp::GreaterThan,
            threshold: 2.0,
            quantity_needed: 1.0,
            tag_value: None,
        }];
        let result = execute_acquire_step(&reqs, &env, &[]);
        assert!(result.is_ok());
        let candidates = result.unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].0, 1); // stone has hardness 3.0 > 2.0
    }

    #[test]
    fn execute_acquire_step_skips_already_acquired() {
        let env = vec![(1, (5, 5), make_stone_props())];
        let reqs = vec![PropertyRequirement {
            property: "hardness".into(),
            operator: CompareOp::GreaterOrEqual,
            threshold: 3.0,
            quantity_needed: 1.0,
            tag_value: None,
        }];
        let already = vec![AcquiredObject {
            entity_id: 1,
            properties: make_stone_props(),
        }];
        let result = execute_acquire_step(&reqs, &env, &already);
        assert!(result.is_err());
    }

    #[test]
    fn execute_acquire_step_returns_error_when_no_match() {
        let env = vec![(1, (5, 5), make_stone_props())];
        let reqs = vec![PropertyRequirement {
            property: "flammability".into(),
            operator: CompareOp::Present,
            threshold: 0.0,
            quantity_needed: 1.0,
            tag_value: None,
        }];
        let result = execute_acquire_step(&reqs, &env, &[]);
        assert!(matches!(result, Err(AcquireError::NoMatchingObject)));
    }

    #[test]
    fn verify_prerequisites_detects_changed_object() {
        let env_before = vec![(1, (5, 5), make_stone_props())];
        let reqs = vec![PropertyRequirement {
            property: "hardness".into(),
            operator: CompareOp::GreaterThan,
            threshold: 2.0,
            quantity_needed: 1.0,
            tag_value: None,
        }];
        assert!(verify_prerequisites(1, &reqs, &env_before));

        // Object changed — no longer satisfies
        let env_after = vec![(1, (5, 5), make_stick_props())]; // hardness 1.0 < 2.0
        assert!(!verify_prerequisites(1, &reqs, &env_after));
    }

    #[test]
    fn retry_exhaustion_fails_plan() {
        let mut state = ExecutionState {
            intention: Some(KnowledgeId(1)),
            steps: vec![DecompositionStep::Acquire {
                requirements: vec![PropertyRequirement {
                    property: "hardness".into(),
                    operator: CompareOp::GreaterThan,
                    threshold: 100.0, // impossible
                    quantity_needed: 1.0,
                    tag_value: None,
                }],
            }],
            step_index: 0,
            intention_score: 0.5,
            acquired_objects: vec![],
            retry_count: 0,
        };
        let env = vec![(1, (5, 5), make_stone_props())];

        // Tick 4 times (MAX_RETRY_PER_STEP = 3, so 4th exhausts)
        for _ in 0..=MAX_RETRY_PER_STEP {
            tick_execution(&mut state, &env, (5, 5));
        }
        assert!(is_plan_failed(&state));
    }

    #[test]
    fn is_plan_complete_when_all_steps_done() {
        let state = ExecutionState {
            intention: Some(KnowledgeId(1)),
            steps: vec![DecompositionStep::Act {
                action: "Strike".into(),
                target: None,
            }],
            step_index: 1, // past the last step
            intention_score: 0.5,
            acquired_objects: vec![],
            retry_count: 0,
        };
        assert!(is_plan_complete(&state));
        assert!(!is_plan_failed(&state));
    }

    #[test]
    fn material_satisfies_present_check() {
        let stone = make_stone_props();
        assert!(stone.satisfies(&PropertyRequirement {
            property: "spark".into(),
            operator: CompareOp::Present,
            threshold: 0.0,
            quantity_needed: 1.0,
            tag_value: None,
        }));
        assert!(!stone.satisfies(&PropertyRequirement {
            property: "edge".into(),
            operator: CompareOp::Present,
            threshold: 0.0,
            quantity_needed: 1.0,
            tag_value: None,
        }));
    }
}
