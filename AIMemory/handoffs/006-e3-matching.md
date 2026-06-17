# Handoff 006-e3 — 匹配搜索引擎

## 架构计划

**改什么：** 新建 `src/need_match/search.rs` + 更新 `mod.rs`（2 文件）
**依据：** `design-philosophy-v5.md` §3.2-3.4、双方向搜索设计、本质查询引擎

### search.rs

```rust
/// A 方向：激活需求 → 查知识图 → 返回匹配候选
pub fn search_by_needs(
    needs: &[NeedState],
    knowledge: &KnowledgeGraph,
) -> Vec<CandidateAction> {
    let activated: Vec<&NeedState> = needs.iter()
        .filter(|n| n.urgency > URGENCY_ACTIVATION_THRESHOLD && !n.blocked)
        .collect();
    
    let mut candidates = Vec::new();
    for entry in knowledge.entries.values() {
        let matched: Vec<NeedKind> = entry.effects.iter()
            .filter(|e| activated.iter().any(|n| n.kind == e.satisfies))
            .map(|e| e.satisfies)
            .collect();
        if !matched.is_empty() {
            // achievability = 功能前提中已满足的占比
            let prereqs_met = count_prerequisites_met(&entry.functional_prerequisites, /* environment */);
            let total_prereqs = entry.functional_prerequisites.len().max(1);
            candidates.push(CandidateAction {
                knowledge_id: entry.id,
                matched_needs: matched,
                achievability: prereqs_met as f32 / total_prereqs as f32,
                risk: 0.0,  // A方向已知方案，风险 = 0
                source: CandidateSource::KnowledgeGraph,
            });
        }
    }
    candidates
}

/// B 方向：感知物体 → 查知识图 → "这东西能做什么？"
pub fn search_by_environment(
    needs: &[NeedState],
    perceived_properties: &[(PropertyRequirement, /* 实体的属性表 */)],
    knowledge: &KnowledgeGraph,
) -> Vec<CandidateAction> {
    // 感知物体查知识图——有没有条目用的上这些属性的物体
    todo!("B方向简单搜索：属性匹配")
}

/// B 方向本质查询：感知物体两两组合 → 属性代数合并 → 检查是否覆盖需求
pub fn search_by_essence_combine(
    needs: &[NeedState],
    perceived_properties: &[MaterialProperties],
) -> Vec<CandidateAction> {
    // 两两组合 → 模拟合并属性 → 检查是否覆盖激活需求
    todo!("本质查询引擎：O(k²) 组合匹配")
}

/// 冲突仲裁：选最优候选（紧迫度 × 可达成度）
/// 选择阈值：新候选的 score 必须 > 当前意图 × DECISION_THRESHOLD_DEFAULT 才切换
pub fn arbitrate(
    candidates: Vec<CandidateAction>,
    needs: &[NeedState],
    current_intention_score: f32,
) -> Option<CandidateAction> {
    let mut scored: Vec<_> = candidates.into_iter()
        .map(|c| {
            let urgency_sum: f32 = c.matched_needs.iter()
                .filter_map(|nk| needs.iter().find(|n| n.kind == *nk))
                .map(|n| n.urgency)
                .sum();
            let score = urgency_sum * c.achievability * (1.0 - c.risk);
            (score, c)
        })
        .collect();
    
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    
    if let Some((best_score, best)) = scored.into_iter().next() {
        if best_score > current_intention_score * DECISION_THRESHOLD_DEFAULT {
            return Some(best);
        }
    }
    None
}

/// 可行性过滤——基于人格标签调节
pub fn filter_by_feasibility(
    candidates: Vec<CandidateAction>,
    personality_tags: &TagBits,  // 位掩码人格标签
) -> Vec<CandidateAction> {
    // reckless → 跳过所有过滤
    // cautious → 过滤 achievability < 0.5
    // clever → 过滤 + 风险排序
    todo!("人格调制可行性过滤")
}

fn count_prerequisites_met(prereq: &[PropertyRequirement]) -> usize {
    // 检查功能前提中哪些已被环境满足
    // 当前 stub：后续对接感知数据
    0
}

/// 材质属性表（供本质查询使用）
#[derive(Debug, Clone)]
pub struct MaterialProperties {
    pub hardness: Option<f32>,
    pub density: Option<f32>,
    pub flammability: Option<f32>,
    pub spark_on_strike: bool,
    pub edge_present: bool,
    pub mass_kg: f32,
}
```

### mod.rs 更新

```rust
pub mod activation;
pub mod search;
pub use search::*;
```

## 架构反馈

**当前 handoff 范围：**
- `search_by_needs` — 完整实现（A 方向最基础路径）
- `search_by_environment` — `todo!()`（需感知数据就位）
- `search_by_essence_combine` — `todo!()`（需 MaterialProperties 映射就位）
- `arbitrate` — 完整实现（冲突仲裁）
- `filter_by_feasibility` — `todo!()`（需人格标签系统就位）

**设计哲学一致性：**
- 无配方 ✅ — 不写死"篝火=树枝+燧石"，查功能前提
- 需求覆盖率 ✅ — 覆盖越多急迫需求的候选越优先
- 选择阈值防振荡 ✅ — DECISION_THRESHOLD_DEFAULT = 1.2

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 测试：伪造需求+知识图条目 → search_by_needs 返回正确候选
- 测试：arbitrate 高分候选胜出，低分被阈值过滤
