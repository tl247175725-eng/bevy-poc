# Handoff 011 — TagRegistry 初始化 + TagBits 占位符替换

## 架构计划

**改什么：** `src/tags.rs` + `src/card_def.rs` + `src/world_rules.rs`（3 文件）
**做什么：** 加载 tags.ron → 初始化 TagRegistry → 替换所有 `TagBits::new()` 占位符

### tags.rs — 实现 from_tags_ron

```rust
impl TagRegistry {
    pub fn from_tags_ron() -> Self {
        // 读取 assets/tags.ron
        // 遍历树结构:
        //   1. 每个节点分配 bit
        //   2. 递归收集子节点 → 计算 descendants 位掩码
        //   3. 填入 name_to_bit / bit_to_name / descendants
        // 返回注册表
    }
}
```

### world_rules.rs — 初始化

```rust
// 在 App 启动时调用一次
pub fn init_tag_registry() {
    let registry = TagRegistry::from_tags_ron();
    TAG_REGISTRY.set(registry).expect("TagRegistry 只能初始化一次");
}
```

### card_def.rs — CardDef 加载时填 tag_bits

```rust
pub fn load_card_defs_with_tags(path: impl AsRef<Path>) -> Vec<CardDef> {
    let mut defs = load_card_defs(path);
    if let Some(registry) = TAG_REGISTRY.get() {
        for def in &mut defs {
            def.tag_bits = TagBits::from_tag_names(&def.tags, registry);
        }
    }
    defs
}
```

### TagBits 新增

```rust
pub fn from_tag_names(names: &[String], registry: &TagRegistry) -> Self {
    let mut bits = TagBits::new();
    for name in names {
        if let Some(&bit) = registry.name_to_bit.get(name.as_str()) {
            bits.set(bit);
        }
    }
    bits
}
```

## 架构反馈

- tags.ron → TagRegistry → TagBits 完整链路打通 ✅
- card_has_tag 优先走位掩码（已有），fallback 字符串 ✅
- 感知系统 Predator/Hidden 等标签改用真实 bit 查询 ✅

## 注意事项

- `from_tags_ron` 需要解析 tags.ron 的树结构。当前 tags.ron 是 RON 格式，深度嵌套。优先实现 positional 树的解析（身体部位是嵌套 dict），systemic 树同样处理。
- 位分配规则：深度优先遍历，每个节点递增分配 bit。bit 范围由 tags.ron 的 bit_ranges 决定。

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 测试：加载 tags.ron → TagRegistry.name_to_bit 非空
- 测试：BODY descendants 包含 HEAD
- 测试：CardDef tag_bits 正确映射
