//! 匹配搜索引擎 — 双方向搜索 + 冲突仲裁
//!
//! 依据 design-philosophy-v5.md §3.2-3.4

use super::activation::URGENCY_ACTIVATION_THRESHOLD;
use super::data::{
    CandidateAction, CandidateSource, CompareOp, KnowledgeGraph, NeedKind, NeedState, PropertyRequirement,
};

use crate::meta_values::DECISION_THRESHOLD_DEFAULT;
use crate::tags::TagBits;

// ===== 材质属性表 =====

/// 材质属性表（供本质查询使用）
#[derive(Debug, Clone)]
pub struct MaterialProperties {
    pub hardness: Option<f32>,
    pub density: Option<f32>,
    pub flammability: Option<f32>,
    pub spark_on_strike: bool,
    pub edge_present: bool,
    pub mass_kg: f32,
    /// Tag-based properties (e.g. "edible", "prey", "cover") derived from card tags
    pub tags: Vec<String>,
}

impl MaterialProperties {
    /// 检查单个属性需求是否满足
    pub fn satisfies(&self, req: &PropertyRequirement) -> bool {
        // Tag-based bridge: "is_plant" ↔ nutrition:autotroph 或 body_plan:plant
        if req.operator == CompareOp::Present && req.property == "is_plant" {
            return self.tags.contains(&"nutrition:autotroph".to_string())
                || self.tags.contains(&"body_plan:plant".to_string());
        }
        // Tag-based bridge: "is_animal" → 检查 animal 标签
        if req.operator == CompareOp::Present && req.property == "is_animal" {
            return self.tags.contains(&"animal".to_string());
        }
        // has_tag 检查：查目标实体 tags 是否包含 req.tag_value
        if req.operator == CompareOp::Present && req.property == "has_tag" {
            return req.tag_value.as_ref().map_or(false, |tv| self.tags.contains(tv));
        }
        // Tag-based properties (by convention: tag name used as property name)
        if req.operator == CompareOp::Present && self.tags.contains(&req.property) {
            return true;
        }
        let value = match req.property.as_str() {
            "hardness" => self.hardness,
            "density" => self.density,
            "flammability" => self.flammability,
            "mass" => Some(self.mass_kg),
            "edge" if req.operator == CompareOp::Present => return self.edge_present,
            "spark" if req.operator == CompareOp::Present => return self.spark_on_strike,
            _ => return false,
        };
        match (value, &req.operator) {
            (Some(v), CompareOp::GreaterThan) => v > req.threshold,
            (Some(v), CompareOp::LessThan) => v < req.threshold,
            (Some(v), CompareOp::GreaterOrEqual) => v >= req.threshold,
            (Some(v), CompareOp::Equal) => (v - req.threshold).abs() < f32::EPSILON,
            _ => false,
        }
    }
}

// ===== A 方向：需求 → 知识图 → 候选 =====

/// A 方向：激活需求 → 查知识图 → 返回匹配候选
pub fn search_by_needs(
    needs: &[NeedState],
    knowledge: &KnowledgeGraph,
) -> Vec<CandidateAction> {
    let activated: Vec<&NeedState> = needs
        .iter()
        .filter(|n| n.urgency > URGENCY_ACTIVATION_THRESHOLD && !n.blocked)
        .collect();

    let mut candidates = Vec::new();
    for entry in knowledge.entries.values() {
        let matched: Vec<NeedKind> = entry
            .effects
            .iter()
            .filter(|e| activated.iter().any(|n| n.kind == e.satisfies))
            .map(|e| e.satisfies.clone())
            .collect();
        if !matched.is_empty() {
            let prereqs_met = count_prerequisites_met(&entry.functional_prerequisites);
            let total_prereqs = entry.functional_prerequisites.len().max(1);
            candidates.push(CandidateAction {
                knowledge_id: entry.id.clone(),
                matched_needs: matched,
                achievability: if entry.functional_prerequisites.is_empty() {
                    1.0
                } else {
                    prereqs_met as f32 / total_prereqs as f32
                },
                risk: 0.0, // A 方向已知方案，风险 = 0
                source: CandidateSource::KnowledgeGraph,
            });
        }
    }
    candidates
}

// ===== B 方向：感知 → 知识图 → 候选 =====

/// B 方向简单搜索：感知物体 → 查知识图 → "这东西能做什么？"
pub fn search_by_environment(
    _needs: &[NeedState],
    _perceived_properties: &[PropertyRequirement],
    _knowledge: &KnowledgeGraph,
) -> Vec<CandidateAction> {
    todo!("B方向简单搜索：属性匹配")
}

/// B 方向本质查询：感知物体两两组合 → 属性代数合并 → 检查是否覆盖需求
pub fn search_by_essence_combine(
    _needs: &[NeedState],
    _perceived_properties: &[MaterialProperties],
) -> Vec<CandidateAction> {
    todo!("本质查询引擎：O(k²) 组合匹配")
}

// ===== 冲突仲裁 =====

/// 冲突仲裁：选最优候选（紧迫度 × 可达成度）。
///
/// 选择阈值：新候选的 score 必须 > 当前意图 × DECISION_THRESHOLD_DEFAULT 才切换。
pub fn arbitrate(
    candidates: Vec<CandidateAction>,
    needs: &[NeedState],
    current_intention_score: f32,
) -> Option<CandidateAction> {
    let mut scored: Vec<_> = candidates
        .into_iter()
        .map(|c| {
            let urgency_sum: f32 = c
                .matched_needs
                .iter()
                .filter_map(|nk| needs.iter().find(|n| n.kind == *nk))
                .map(|n| n.urgency)
                .sum();
            let score = urgency_sum * c.achievability * (1.0 - c.risk);
            (score, c)
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    if let Some((best_score, best)) = scored.into_iter().next() {
        if best_score > current_intention_score * DECISION_THRESHOLD_DEFAULT {
            return Some(best);
        }
    }
    None
}

// ===== 人格调制过滤 =====

/// 可行性过滤——基于人格标签调节
pub fn filter_by_feasibility(
    _candidates: Vec<CandidateAction>,
    _personality_tags: &TagBits,
) -> Vec<CandidateAction> {
    todo!("人格调制可行性过滤")
}

// ===== 辅助 =====

fn count_prerequisites_met(prereq: &[PropertyRequirement]) -> usize {
    // 当前策略：Decide 阶段乐观假设所有属性前提可满足
    // 实际验证在执行阶段的 execute_acquire_step 中进行
    prereq.len()
}

// ===== 测试 =====

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::data::{
        CompareOp, DecompositionStep, EffectDescriptor, KnowledgeEntry, KnowledgeId,
        KnowledgeSource, NeedState,
    };
    use std::collections::HashMap;

    fn make_need(kind: NeedKind, urgency: f32, blocked: bool) -> NeedState {
        NeedState {
            kind,
            current: 0.5,
            baseline: 0.5,
            urgency,
            blocked,
            decay_rate: crate::meta_values::NUTRITION_DECAY_MEDIUM,
        }
    }

    fn make_knowledge_graph() -> KnowledgeGraph {
        let eat_effect = EffectDescriptor {
            satisfies: NeedKind::Nutrition,
            magnitude: 0.8,
        };
        let rest_effect = EffectDescriptor {
            satisfies: NeedKind::Rest,
            magnitude: 0.5,
        };

        let entry_cook = KnowledgeEntry {
            id: KnowledgeId(1),
            name: "cook".into(),
            functional_prerequisites: vec![PropertyRequirement {
                property: "flammability".into(),
                operator: CompareOp::Present,
                threshold: 0.0,
                quantity_needed: 1.0,
                tag_value: None,
            }],
            decomposition: vec![DecompositionStep::Act {
                action: "cook".into(),
                target: None,
            }],
            effects: vec![eat_effect],
            source: KnowledgeSource::CommonSense,
        };

        let entry_sleep = KnowledgeEntry {
            id: KnowledgeId(2),
            name: "sleep".into(),
            functional_prerequisites: vec![],
            decomposition: vec![DecompositionStep::Act {
                action: "sleep".into(),
                target: None,
            }],
            effects: vec![rest_effect],
            source: KnowledgeSource::CommonSense,
        };

        let mut entries = HashMap::new();
        entries.insert(KnowledgeId(1), entry_cook);
        entries.insert(KnowledgeId(2), entry_sleep);

        KnowledgeGraph {
            entries,
            next_id: 3,
        }
    }

    #[test]
    fn search_by_needs_finds_candidates_for_activated_needs() {
        let needs = vec![
            make_need(NeedKind::Nutrition, 0.8, false), // activated
            make_need(NeedKind::Rest, 0.1, false),       // not activated
        ];
        let kg = make_knowledge_graph();
        let candidates = search_by_needs(&needs, &kg);

        // Nutrition is activated → cook entry should match
        assert!(!candidates.is_empty());
        let cook = candidates
            .iter()
            .find(|c| c.knowledge_id == KnowledgeId(1))
            .expect("cook entry should match Nutrition");
        assert!(cook.matched_needs.contains(&NeedKind::Nutrition));
    }

    #[test]
    fn search_by_needs_excludes_blocked_needs() {
        let needs = vec![
            make_need(NeedKind::Nutrition, 0.8, true), // blocked
        ];
        let kg = make_knowledge_graph();
        let candidates = search_by_needs(&needs, &kg);
        assert!(candidates.is_empty());
    }

    #[test]
    fn search_by_needs_returns_empty_when_no_activated_needs() {
        let needs = vec![make_need(NeedKind::Rest, 0.1, false)]; // below threshold
        let kg = make_knowledge_graph();
        let candidates = search_by_needs(&needs, &kg);
        assert!(candidates.is_empty());
    }

    #[test]
    fn arbitrate_picks_highest_score() {
        let candidates = vec![
            CandidateAction {
                knowledge_id: KnowledgeId(1),
                matched_needs: vec![NeedKind::Nutrition],
                achievability: 0.8,
                risk: 0.0,
                source: CandidateSource::KnowledgeGraph,
            },
            CandidateAction {
                knowledge_id: KnowledgeId(2),
                matched_needs: vec![NeedKind::Rest],
                achievability: 0.3,
                risk: 0.0,
                source: CandidateSource::KnowledgeGraph,
            },
        ];
        let needs = vec![
            make_need(NeedKind::Nutrition, 0.9, false),
            make_need(NeedKind::Rest, 0.5, false),
        ];
        let result = arbitrate(candidates, &needs, 0.0);
        assert!(result.is_some());
        // Nutrition urgency=0.9, achievability=0.8 → score=0.72
        // Rest urgency=0.5, achievability=0.3 → score=0.15
        // Nutrition wins
        assert_eq!(result.unwrap().knowledge_id, KnowledgeId(1));
    }

    #[test]
    fn arbitrate_rejects_below_threshold() {
        let candidates = vec![CandidateAction {
            knowledge_id: KnowledgeId(1),
            matched_needs: vec![NeedKind::Nutrition],
            achievability: 0.5,
            risk: 0.0,
            source: CandidateSource::KnowledgeGraph,
        }];
        let needs = vec![make_need(NeedKind::Nutrition, 0.3, false)];
        // score = 0.3 * 0.5 = 0.15
        // threshold = 0.5 * 1.2 = 0.6
        // 0.15 < 0.6 → rejected
        let result = arbitrate(candidates, &needs, 0.5);
        assert!(result.is_none());
    }

    #[test]
    fn count_prerequisites_met_returns_count() {
        let prereqs = vec![PropertyRequirement {
            property: "flammability".into(),
            operator: CompareOp::Present,
            threshold: 0.0,
            quantity_needed: 1.0,
            tag_value: None,
        }];
        assert_eq!(count_prerequisites_met(&prereqs), 1);
    }
}