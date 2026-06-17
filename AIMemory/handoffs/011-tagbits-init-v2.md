# Handoff 011 v2 — TagRegistry + TagBits 初始化（纯 Rust，无 RON 解析）

> ⚠️ v1 用 ron crate 解析 tags.ron 失败——ron 不支持 HashMap<String,T> 的标识符键。
> v2：标签常量直接用 Rust const 定义。和 meta_values.rs 一样，纯常量，零解析开销。

## 架构计划

**改什么：** `src/tags.rs` + `src/world_rules.rs` + `src/card_def.rs`（3 文件）

**清理 v1 残骸：** v1 可能在以下位置留下了部分代码——TagRegistry 中的 RON 解析逻辑、tags.ron 中 Python 脚本修改的引号格式。全部删除，从干净状态开始。

### tags.rs — Tag 常量 + TagRegistry::default

```rust
// 标签常量——编译期定义，不解析任何文件
pub mod tag {
    use super::TagInfo;
    
    pub const BODY: TagInfo     = TagInfo { bit: 0,  parent: None };
    pub const HEAD: TagInfo     = TagInfo { bit: 1,  parent: Some(0) };
    pub const SKULL: TagInfo    = TagInfo { bit: 2,  parent: Some(1) };
    pub const BRAIN: TagInfo    = TagInfo { bit: 3,  parent: Some(1) };
    // ... 到 ~70 个标签（对应 tags.ron 的结构）
}

impl TagRegistry {
    /// 从编译期常量构建 TagRegistry
    pub fn default_registry() -> Self {
        let mut reg = Self { name_to_bit: HashMap::new(), bit_to_name: HashMap::new(), descendants: HashMap::new(), next_bit: 512 };
        // 注册所有 tag::* 常量
        // 递归计算 descendants（从 parent 关系自动构建树）
    }
}
```

### world_rules.rs — 启动时初始化

```rust
pub fn init_tag_registry() {
    let reg = TagRegistry::default_registry();
    TAG_REGISTRY.set(reg).ok();
}
```

### card_def.rs — CardDef 加载时填 tag_bits

```rust
pub fn init_tag_bits(defs: &mut [CardDef], registry: &TagRegistry) {
    for def in defs {
        def.tag_bits = TagBits::from_tag_names(&def.tags, registry);
    }
}
```

### 关键简化

❌ 不解析 tags.ron
❌ 不用 serde/ron
✅ Rust const 常量定义 ~70 个标签
✅ parent 关系直接写在 TagInfo.parent 中
✅ TagRegistry.default_registry() 从常量构建

## 架构反馈

- tags.ron 保留作为设计文档（人类阅读用）
- 运行时不读 tags.ron——所有标签定义编译进二进制
- 加新标签 = 改 tags.rs 加一行常量 + 改 tags.ron 加设计文档。build.rs 自动化后续做

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 测试：TagRegistry 包含 BODY/HEAD/PREDATOR 等标签
- 测试：has_descendant_of(BODY, HEAD) = true
