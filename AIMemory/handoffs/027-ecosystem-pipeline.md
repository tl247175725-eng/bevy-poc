# Handoff 027 — 生态管线接入

## 架构计划

**改什么：** `src/initial_spawn.rs` + `src/systems/main_tick.rs`（2 文件）
**做什么：** 动物生成时初始化 NeedState + 知识图 + 把 need/perceive/decide 管线接入 main_tick

### 1. 动物生成时初始化 Needs（initial_spawn.rs）

每种动物根据标签自动创建需求：

```rust
fn init_animal_needs(tags: &[String]) -> Vec<NeedState> {
    let mut needs = Vec::new();
    
    // 所有动物都需要营养
    needs.push(NeedState {
        kind: NeedKind::Nutrition,
        current: 0.0,   // 初始饱食
        baseline: 0.3,
        decay_rate: if has_tag(tags, "metab:high") { 0.7 } 
                   else if has_tag(tags, "metab:low") { 0.2 }
                   else { 0.4 },
        blocked: false,
    });
    
    // 所有动物都需要安全
    needs.push(NeedState {
        kind: NeedKind::Safety,
        current: 0.0,
        baseline: 1.0,
        decay_rate: 0.0,  // 安全不自然衰减——由威胁触发
        blocked: false,
    });
    
    // 有 social:pack/herd 的动物需要社交
    if has_tag(tags, "social:pack") || has_tag(tags, "social:herd") {
        needs.push(NeedState {
            kind: NeedKind::Social,
            current: 0.0,
            baseline: 0.5,
            decay_rate: 0.1,
            blocked: false,
        });
    }
    
    // cognition:basic_learning 以上的需要好奇
    if has_tag(tags, "cognition:basic_learning") {
        needs.push(NeedState {
            kind: NeedKind::Curiosity,
            current: 0.0,
            baseline: 0.2,
            decay_rate: 0.05,
            blocked: false,
        });
    }
    
    needs
}
```

### 2. 动物生成时初始化知识图

每种动物根据标签自动生成常识表：

```rust
fn init_animal_knowledge(tags: &[String]) -> KnowledgeGraph {
    let mut kg = KnowledgeGraph::new();
    
    // 食草动物知道"草+树叶=食物"
    if has_tag(tags, "diet:herbivore") {
        kg.add(KnowledgeEntry {
            name: "吃草",
            functional_prerequisites: vec![PropertyRequirement {
                property: "edible".into(), operator: Present, threshold: 0.0, quantity_needed: 1.0,
            }],
            effects: vec![EffectDescriptor { satisfies: NeedKind::Nutrition, magnitude: 0.5 }],
            source: KnowledgeSource::CommonSense,
        });
    }
    
    // 捕食者知道"猎物=食物"
    if has_tag(tags, "diet:carnivore") {
        kg.add(KnowledgeEntry {
            name: "捕猎",
            functional_prerequisites: vec![PropertyRequirement {
                property: "prey".into(), operator: Present, threshold: 0.0, quantity_needed: 1.0,
            }],
            effects: vec![EffectDescriptor { satisfies: NeedKind::Nutrition, magnitude: 0.8 }],
            source: KnowledgeSource::CommonSense,
        });
    }
    
    kg
}
```

### 3. main_tick 接入生态管线

在 Phase 2 (Need tick) 和 Phase 4 (Decide) 之间，对每个动物实体运行：

```rust
// Phase 2.5: 动物实体运行 need_match 引擎
for entity in world.entities.values_mut() {
    if !entity.tags.has_any(&[ANIMAL_TAG]) { continue; }
    
    // 需求衰减
    for need in &mut entity.needs {
        tick_need(need, delta);
    }
    
    // 安全阻断
    apply_safety_block(&mut entity.needs);
    
    // 决策（仅在无当前计划或计划失败时）
    if entity.execution.intention.is_none() || is_plan_failed(&entity.execution) {
        let env = build_env_from_perception(entity, world); // 感知周围物体
        if let Some(action) = tick_need_engine(&mut entity.needs, &mut entity.execution, &entity.knowledge, &env, delta) {
            entity.pending_action = Some(action);
        }
    }
}
```

### 关键原则

- 所有参数从标签推导——不写"鹿吃草"的逻辑
- herbivore 标签→知识图自动含"吃草"条目
- carnivore 标签→知识图自动含"捕猎"条目
- metab:high → 饿得更快

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
