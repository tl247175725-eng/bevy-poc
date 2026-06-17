# Handoff 030 — C2：用派生规则替代 per-diet 知识分支

## 架构计划

**改什么：** `src/initial_spawn.rs` 的 `init_animal_knowledge()`（1 文件内改动）
**为什么：** 当前每个 diet 写一个 `if has_tag` 分支——这是被铁律禁止的行为链。Opus 确认这是叠加域未实现的核心证据。

### 旧代码（违反铁律）

```rust
// ❌ 每个 diet 一个分支——行为链
if has_tag("diet:herbivore") { add("吃草") }
if has_tag("diet:carnivore") { add("捕猎") }
if has_tag("diet:omnivore") { add("吃草"); add("捕猎"); }
// diet:piscivore, insectivore, frugivore... 全漏
```

### 新代码（派生规则）

```rust
/// 从 diet 标签派生可食标签集（一条通用规则，不分物种）
fn diet_to_edible_tags(tags: &TagBits, registry: &TagRegistry) -> Vec<String> {
    let mut edible = Vec::new();
    // carnivore → 可吃 animal
    if tags.has_descendant_of(registry.diet_carnivore_bit) { edible.push("animal".into()); }
    // herbivore → 可吃 plant
    if tags.has_descendant_of(registry.diet_herbivore_bit) { edible.push("plant".into()); }
    // piscivore → 可吃 fish
    if tags.has_descendant_of(registry.diet_piscivore_bit) { edible.push("fish".into()); }
    // insectivore → 可吃 insectoid
    if tags.has_descendant_of(registry.diet_insectivore_bit) { edible.push("insectoid".into()); }
    // frugivore → 可吃 plant（结果实的）
    if tags.has_descendant_of(registry.diet_frugivore_bit) { edible.push("fruit".into()); }
    // omnivore → 可吃 animal + plant
    if tags.has_descendant_of(registry.diet_omnivore_bit) { edible.push("animal".into()); edible.push("plant".into()); }
    // detritivore/scavenger → 可吃 corpse
    if tags.has_descendant_of(registry.diet_scavenger_bit) 
    || tags.has_descendant_of(registry.diet_detritivore_bit) { edible.push("corpse".into()); }
    edible
}

/// 从可食标签集构建**一条**通用觅食知识条目（不外分支）
fn build_foraging_knowledge(edible_tags: &[String]) -> KnowledgeEntry {
    KnowledgeEntry {
        name: "觅食",
        functional_prerequisites: edible_tags.iter().map(|t| PropertyRequirement {
            property: "has_tag".into(),
            operator: Present,
            threshold: 0.0,
            quantity_needed: 1.0,
            tag_value: t.clone(),  // ← 这里存的是派生的可食标签，不是写死的"草"
        }).collect(),
        decomposition: vec![
            DecompositionStep::Acquire { requirements: vec![] },
            DecompositionStep::Act { action: "Consume".into(), target: Some("acquired_0".into()) },
        ],
        effects: vec![EffectDescriptor { satisfies: NeedKind::Nutrition, magnitude: 0.5 }],
        source: KnowledgeSource::CommonSense,
    }
}

/// 重构——一条规则，覆盖全部食性
fn init_animal_knowledge(tags: &TagBits, registry: &TagRegistry) -> KnowledgeGraph {
    let edible = diet_to_edible_tags(tags, registry);
    if edible.is_empty() { return KnowledgeGraph::new(); }
    let kg = KnowledgeGraph::new();
    kg.add(build_foraging_knowledge(&edible));
    kg
}
```

### PropertyRequirement 扩展

在 `src/need_match/data.rs` 的 PropertyRequirement 加 `tag_value: Option<String>` 字段——用于匹配实体标签。

在 `MaterialProperties.satisfies()` 中加对 `property == "has_tag"` 的处理——查目标实体的 tags 是否包含 `tag_value`。

## 架构反馈

**叠加域实现：**
- 不再有 `if diet:xxx` 的知识分支 ✅
- 加新食性 = 在 `diet_to_edible_tags` 加一行映射 + 在 tags.ron 加标签。不改引擎 ✅
- 知识条目从标签**派生**，不硬编码 ✅
- 食性标签组合（omnivore = animal+plant）是自然涌现 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 代码中 `init_animal_knowledge` 内无 `if has_tag("diet: 格式的分支
