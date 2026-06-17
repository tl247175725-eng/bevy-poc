# Handoff 006-e5 — 社会记忆目录

## 架构计划

**改什么：** 新建 `src/need_match/social.rs` + 更新 `mod.rs`（2 文件）
**依据：** `design-philosophy-v5.md` §3、Wegner 1987 交互记忆

### social.rs

```rust
use crate::need_match::data::*;
use std::collections::HashMap;

/// 更新社会目录——从 Signal/Receive/观察中学习"谁擅长什么"
pub fn learn_expertise(
    directory: &mut SocialDirectory,
    from_entity: EntityId,
    knowledge_id: KnowledgeId,
) {
    directory.expertise
        .entry(from_entity)
        .or_default()
        .push(knowledge_id);
}

/// 查询社会目录——谁有满足当前需求的知识？
pub fn find_expert(
    directory: &SocialDirectory,
    needed_knowledge: &[NeedKind],
    knowledge_graphs: &HashMap<EntityId, KnowledgeGraph>,
) -> Vec<(EntityId, KnowledgeId)> {
    let mut results = Vec::new();
    for (&entity_id, expertises) in &directory.expertise {
        if let Some(graph) = knowledge_graphs.get(&entity_id) {
            for &kid in expertises {
                if let Some(entry) = graph.entries.get(&kid) {
                    let matched: Vec<_> = entry.effects.iter()
                        .filter(|e| needed_knowledge.contains(&e.satisfies))
                        .collect();
                    if !matched.is_empty() {
                        results.push((entity_id, kid));
                    }
                }
            }
        }
    }
    results
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
        knowledge_needed: Vec::new(), // 专家会用自己的知识图匹配
    }
}

pub struct SignalRequest {
    pub from: EntityId,
    pub to: EntityId,
    pub content: String,
    pub knowledge_needed: Vec<NeedKind>,
}
```

### mod.rs 更新

```rust
pub mod social;
pub use social::*;
```

## 架构反馈

**Wegner 交互记忆验证：** "Knowing who knows what" → SocialDirectory 存储 EntityId → Vec<KnowledgeId> 映射 ✅
**层次：** 可行性失败 → 查社会目录 → find_expert → request_help → Signal 元动作
**简单化：** 此 handoff 只做 SocialDirectory 的存储和查询 API，不做求助的完整 Signal 流程——后续对接感知层。

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 测试：learn_expertise 后 find_expert 能找到匹配专家
- 测试：无匹配专家时返回空列表
