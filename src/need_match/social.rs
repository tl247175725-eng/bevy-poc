//! 社会记忆目录 — "谁懂什么" 基础存储和查询
//!
//! 依据 design-philosophy-v5.md §3、Wegner 1987 交互记忆

use std::collections::HashMap;

use crate::spatial_index::EntityId;

use super::data::{KnowledgeGraph, KnowledgeId, NeedKind, SocialDirectory};

// ===== 学习 =====

/// 更新社会目录——从 Signal/Receive/观察中学习"谁擅长什么"
pub fn learn_expertise(
    directory: &mut SocialDirectory,
    from_entity: EntityId,
    knowledge_id: KnowledgeId,
) {
    directory
        .expertise
        .entry(from_entity)
        .or_default()
        .push(knowledge_id);
}

// ===== 查询 =====

/// 查询社会目录——谁有满足当前需求的知识？
pub fn find_expert(
    directory: &SocialDirectory,
    needed_knowledge: &[NeedKind],
    knowledge_graphs: &HashMap<EntityId, KnowledgeGraph>,
) -> Vec<(EntityId, KnowledgeId)> {
    let mut results = Vec::new();
    for (&entity_id, expertises) in &directory.expertise {
        if let Some(graph) = knowledge_graphs.get(&entity_id) {
            for kid in expertises {
                if let Some(entry) = graph.entries.get(&kid) {
                    let matched: Vec<_> = entry
                        .effects
                        .iter()
                        .filter(|e| needed_knowledge.contains(&e.satisfies))
                        .collect();
                    if !matched.is_empty() {
                        results.push((entity_id, kid.clone()));
                    }
                }
            }
        }
    }
    results
}

// ===== 求助 =====

/// 求助信号
pub struct SignalRequest {
    pub from: EntityId,
    pub to: EntityId,
    pub content: String,
    pub knowledge_needed: Vec<NeedKind>,
}

/// 询问专家——向 expert_id 发送 Signal 请求帮助
pub fn request_help(
    self_id: EntityId,
    expert_id: EntityId,
    problem_description: &str,
) -> SignalRequest {
    SignalRequest {
        from: self_id,
        to: expert_id,
        content: format!("需要帮助: {}", problem_description),
        knowledge_needed: Vec::new(),
    }
}

// ===== 测试 =====

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::data::{
        EffectDescriptor, KnowledgeEntry, KnowledgeSource,
    };

    fn make_graph_with(id: u64, need: NeedKind) -> KnowledgeGraph {
        let mut entries = HashMap::new();
        entries.insert(
            KnowledgeId(id),
            KnowledgeEntry {
                id: KnowledgeId(id),
                name: format!("skill_{id}"),
                functional_prerequisites: vec![],
                decomposition: vec![],
                effects: vec![EffectDescriptor {
                    satisfies: need,
                    magnitude: 0.8,
                }],
                source: KnowledgeSource::CommonSense,
            },
        );
        KnowledgeGraph {
            entries,
            next_id: id + 1,
        }
    }

    #[test]
    fn learn_and_find_expert() {
        let mut dir = SocialDirectory {
            expertise: HashMap::new(),
        };
        let alice = EntityId(1);
        let bob = EntityId(2);

        // Alice knows how to cook (Nutrition)
        learn_expertise(&mut dir, alice, KnowledgeId(10));
        // Bob knows how to rest
        learn_expertise(&mut dir, bob, KnowledgeId(20));

        let mut graphs = HashMap::new();
        graphs.insert(alice, make_graph_with(10, NeedKind::Nutrition));
        graphs.insert(bob, make_graph_with(20, NeedKind::Rest));

        // Find cooking expert
        let experts = find_expert(&dir, &[NeedKind::Nutrition], &graphs);
        assert_eq!(experts.len(), 1);
        assert_eq!(experts[0].0, alice);
        assert_eq!(experts[0].1, KnowledgeId(10));
    }

    #[test]
    fn find_expert_returns_empty_when_no_match() {
        let mut dir = SocialDirectory {
            expertise: HashMap::new(),
        };
        let eve = EntityId(3);
        learn_expertise(&mut dir, eve, KnowledgeId(30));

        let mut graphs = HashMap::new();
        graphs.insert(eve, make_graph_with(30, NeedKind::Rest));

        // Looking for Nutrition — no match
        let experts = find_expert(&dir, &[NeedKind::Nutrition], &graphs);
        assert!(experts.is_empty());
    }

    #[test]
    fn find_expert_skips_unknown_entities() {
        let mut dir = SocialDirectory {
            expertise: HashMap::new(),
        };
        let ghost = EntityId(99);
        learn_expertise(&mut dir, ghost, KnowledgeId(1));

        // No knowledge graphs for ghost
        let experts = find_expert(&dir, &[NeedKind::Nutrition], &HashMap::new());
        assert!(experts.is_empty());
    }

    #[test]
    fn request_help_creates_signal() {
        let req = request_help(EntityId(1), EntityId(2), "需要生火");
        assert_eq!(req.from, EntityId(1));
        assert_eq!(req.to, EntityId(2));
        assert!(req.content.contains("生火"));
        assert!(req.knowledge_needed.is_empty());
    }
}
