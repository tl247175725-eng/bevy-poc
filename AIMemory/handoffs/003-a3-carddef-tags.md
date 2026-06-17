# Handoff 003-a3-1 — CardDef + world_rules 迁移到位掩码

## 架构计划

**改什么：** `src/card_def.rs` + `src/world_rules.rs`（2 文件）
**为什么：** 标签查询从字符串分配+遍历换成位掩码 O(1)

### card_def.rs 改动

1. 在 `CardDef` struct 中新增 `tag_bits: TagBits` 字段
2. 新增 `CardDef::has_tag_from_registry(&self, registry: &TagRegistry, name: &str) -> bool`
3. `load_card_defs` 保持加载 `Vec<String>` 原始标签，暂不解析位掩码（解析需要 TagRegistry 就位——后续 handoff）
4. 新增 `fn init_tag_bits(defs: &mut [CardDef], registry: &TagRegistry)` 函数——把 tags 字符串向量转为 TagBits

### world_rules.rs 改动

1. `card_has_tag(def, tag)` — 增加一条路径：如果 `def.tag_bits` 非空（`bits != [0;8]`），走位掩码查询；否则 fallback 旧字符串路径
2. 不删除旧字符串实现——保持向后兼容（tag_zh.rs 等仍依赖字符串标签）
3. 新标签查询优先走位掩码，旧路径作为未迁移标签的兜底

### 位掩码查询实现

```rust
// world_rules.rs
pub fn card_has_tag(def: &CardDef, tag: &str) -> bool {
    // 新路径：位掩码查询（TagRegistry 就位后）
    if let Some(ref registry) = TAG_REGISTRY.get() {
        if let Some(&bit) = registry.name_to_bit.get(tag) {
            return def.tag_bits.has(bit);
        }
    }
    // 旧路径：字符串 fallback
    def.tags.iter().any(|t| t == tag || t.starts_with(&format!("{tag}.")))
}
```

使用全局 `OnceLock<TagRegistry>` 避免在所有函数签名中传递 registry。TagRegistry 在 App 启动时初始化一次。

### 不做的

- 不删除 `tags: Vec<String>`（保留给 tag_zh.rs 和未迁移代码）
- 不移除旧查询逻辑（作为 fallback）
- 不改 tag_zh.rs、card_audit.rs（后续 handoff）

## 架构反馈

**兼容策略：**
- 渐进迁移：旧路径保留 → 不会破坏现有代码 ✅
- 位掩码优先：TagRegistry 就位后走快速路径 ✅
- 旧标签 Vec 保留：中文化/审计仍可用 ✅

**后续 debt：**
- `TAG_REGISTRY` 全局 OnceLock 是临时方案——后续移到 WorldState
- `format!("{tag}.")` 堆分配仍存在于 fallback 路径中——全部迁移完后删除

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 新增测试：`carddef_tagbits_empty_by_default` — 新 CardDef 的 tag_bits 全零
- 现有所有依赖 card_has_tag 的测试不受影响
