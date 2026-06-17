# Handoff 033 — 修幽灵标签：食性映射到真实卡牌标签

## 架构计划

**改什么：** `src/initial_spawn.rs` 的 `diet_to_edible_tags()`（1 文件）

**为什么：** 审计发现 6 个 edible tag 中 4 个是幽灵字符串——不存在于任何 card_defs.ron 卡牌标签中。食鱼/食虫/食果/食腐动物永远找不到食物。

### 现状

| edible tag | card_defs.ron 中存在？ | 实际匹配方式 |
|---|---|---|
| "plant" | ✅ 植物卡牌有 `"plant"` 标签 | 旧字符串 fallback 有效 |
| "animal" | ✅ 动物卡牌有 `"animal"` 标签 | 旧字符串 fallback 有效 |
| "fish" | ❌ 不存在——卡牌有 `"body_plan:fish"` | **永远找不到** |
| "insectoid" | ❌ 不存在 | **永远找不到** |
| "fruit" | ❌ 不存在 | **永远找不到** |
| "corpse" | ❌ 不存在 | **永远找不到** |

### 修复

将 `diet_to_edible_tags()` 中的 edible tag 字符串改为 card_defs.ron 中实际存在的标签名：

```rust
fn diet_to_edible_tags(tags: &impl TagQuery, registry: &TagRegistry) -> Vec<String> {
    let mut edible = Vec::new();
    // carnivore → 可吃 animal
    if tags.has_descendant_of(registry, tag::DIET_CARNIVORE.bit) { edible.push("animal".into()); }
    // herbivore → 可吃 plant
    if tags.has_descendant_of(registry, tag::DIET_HERBIVORE.bit) { edible.push("plant".into()); }
    // piscivore → 可吃 body_plan:fish
    if tags.has_descendant_of(registry, tag::DIET_PISCIVORE.bit) { edible.push("body_plan:fish".into()); }
    // insectivore → 可吃 body_plan:insectoid
    if tags.has_descendant_of(registry, tag::DIET_INSECTIVORE.bit) { edible.push("body_plan:insectoid".into()); }
    // frugivore → 可吃 plant（果实尚未作为独立卡牌类型存在）
    if tags.has_descendant_of(registry, tag::DIET_FRUGIVORE.bit) { edible.push("plant".into()); }
    // omnivore → 可吃 animal + plant
    if tags.has_descendant_of(registry, tag::DIET_OMNIVORE.bit) { edible.push("animal".into()); edible.push("plant".into()); }
    // detritivore/scavenger → 尸体通过 is_corpse 判断，不走 has_tag
    // 暂时保留 "corpse" 作为标记——后续 handoff 改 MaterialProperties.satisfies() 支持 is_corpse
    if tags.has_descendant_of(registry, tag::DIET_SCAVENGER.bit)
        || tags.has_descendant_of(registry, tag::DIET_DETRITIVORE.bit) { edible.push("animal".into()); } // 先吃尸体=死动物
    edible
}
```

关键修改：
- `"fish"` → `"body_plan:fish"`（carp 和 catfish 的卡牌标签中包含此字符串）
- `"insectoid"` → `"body_plan:insectoid"`
- `"fruit"` → `"plant"`（果实在游戏中未独立建模，食果动物暂时能吃植物）
- `"corpse"` → `"animal"`（食腐动物搜索死动物，后续通过 is_corpse + 公理判断可否食用）

### 注意

`MaterialProperties.satisfies()` 中的 `has_tag` 检查匹配 `MaterialProperties.tags`（Vec<String>），后者来自 `card_to_material_properties()` 中的 `def.tags.clone()`。card_defs.ron 中鱼类卡牌有 `"body_plan:fish"` 标签字符串，所以搜索能匹配到。

## 架构反馈

1. 幽灵标签是双轨标签系统（TagBits vs 旧 Vec<String>）的症状——下次 handoff 需要统一
2. "corpse" 的解决方案是暂时的——最终应该让 `satisfies()` 支持检查 `is_corpse` 运行时字段

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- `diet_to_edible_tags()` 中不再有不存在于 card_defs.ron 的标签字符串
