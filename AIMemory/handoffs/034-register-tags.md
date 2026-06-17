# Handoff 034 — 注册缺失标签 + 修标签名不一致

## 架构计划

**改什么：** `src/tags.rs`（1 文件）

**为什么：** 审计发现：
1. `animal`、`plant`、`terrain`、`tree`、`fish` 在 card_defs.ron 中使用但在 TagRegistry 中未注册——所有依赖这些标签的位掩码查询走慢速 fallback
2. `personality:bold`（tags.ron）vs `reckless`（tags.rs）名称不一致
3. `personality:social`（tags.ron）vs `peaceful`（tags.rs）名称不一致
4. `state:dead` 在 tags.ron 第 318 行定义了但在 tags.rs 中缺失
5. `next_bit` 应该从 100 改为 264（因为已有标签占到了 bit 263，从 100 开始会冲突）

## 具体改动

### 改动 1：添加缺失的标签常量

在 tags.rs 的 `TAG_CONSTANTS` 数组末尾，已有标签之后，添加以下标签常量：

**基础分类标签（接 bit 264 开始）：**
```rust
// ── 基础分类标签（自动注册） ──
pub const BASE_TERRAIN: TagInfo  = TagInfo { name: "terrain", bit: 264, parent_bit: None };
pub const BASE_TREE: TagInfo     = TagInfo { name: "tree",    bit: 265, parent_bit: None };
pub const BASE_PLANT: TagInfo    = TagInfo { name: "plant",   bit: 266, parent_bit: None };
pub const BASE_ANIMAL: TagInfo   = TagInfo { name: "animal",  bit: 267, parent_bit: None };
pub const BASE_FISH: TagInfo     = TagInfo { name: "fish",    bit: 268, parent_bit: None };
```

**state:dead — 接 bit 269：**
```rust
pub const STATE_DEAD: TagInfo    = TagInfo { name: "state:dead", bit: 269, parent_bit: None };
```

### 改动 2：修标签名不一致（personality）

找到并修改以下两行（约 bit 55-56 附近）：

```rust
// 旧：
pub const PERSONALITY_RECKLESS: TagInfo = TagInfo { name: "reckless", bit: 55, parent_bit: None };
pub const PERSONALITY_PEACEFUL: TagInfo = TagInfo { name: "peaceful", bit: 59, parent_bit: None };

// 新（对齐 tags.ron）：
pub const PERSONALITY_BOLD: TagInfo     = TagInfo { name: "personality:bold",     bit: 55, parent_bit: None };
pub const PERSONALITY_SOCIAL: TagInfo   = TagInfo { name: "personality:social",   bit: 59, parent_bit: None };
```

### 改动 3：修 next_bit

找到 TagRegistry::default_registry() 函数中的 `next_bit: 100`，改为 `next_bit: 270`。

### 改动 4：将新标签常量加入 TAG_CONSTANTS 数组

在 TAG_CONSTANTS（约 620 行附近）的最后添加新常量引用：
```rust
&tag::BASE_TERRAIN,
&tag::BASE_TREE,
&tag::BASE_PLANT,
&tag::BASE_ANIMAL,
&tag::BASE_FISH,
&tag::STATE_DEAD,
```

## 架构反馈

1. 消除双轨标签系统的性能问题：`animal`/`plant` 等高频标签从字符串 fallback 升为位掩码 O(1) 查询
2. 统一 tags.ron 和 tags.rs 的命名——不再有两个矛盾的真值来源
3. next_bit 修复防止未来动态标签冲突

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- `state:dead` 标签常量在 tags.rs 中存在
- `BASE_ANIMAL`/`BASE_PLANT` 等常量可通过 `tag::BASE_ANIMAL.bit` 访问
- `personality:bold` 和 `personality:social` 不再有 `reckless`/`peaceful` 旧名
