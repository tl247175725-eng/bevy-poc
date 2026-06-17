# Handoff 024 — 性状标签接入 TagRegistry

## 架构计划

**改什么：** `src/tags.rs`（1 文件）
**做什么：** 在 `tag` 模块新增 130 个性状标签常量，在 `default_registry()` 注册。

### 标签常量（tag 模块新增）

按 tags.ron 的 15 个维度，每个标签一行：

```rust
// ── 性状标签 (bit 100–230) ──
// 维度 1: habitat
pub const HAB_AQUATIC: TagInfo      = TagInfo { name: "habitat:aquatic",      bit: 100, parent_bit: None };
pub const HAB_WETLAND: TagInfo      = TagInfo { name: "habitat:wetland",      bit: 101, parent_bit: None };
pub const HAB_GRASSLAND: TagInfo    = TagInfo { name: "habitat:grassland",    bit: 102, parent_bit: None };
pub const HAB_FOREST: TagInfo       = TagInfo { name: "habitat:forest",       bit: 103, parent_bit: None };
pub const HAB_MOUNTAIN: TagInfo     = TagInfo { name: "habitat:mountain",     bit: 104, parent_bit: None };
pub const HAB_SUBTERRANEAN: TagInfo = TagInfo { name: "habitat:subterranean", bit: 105, parent_bit: None };
pub const HAB_AERIAL: TagInfo       = TagInfo { name: "habitat:aerial",       bit: 106, parent_bit: None };

// 维度 2: diet
pub const DIET_CARNIVORE: TagInfo   = TagInfo { name: "diet:carnivore",   bit: 107, parent_bit: None };
pub const DIET_HERBIVORE: TagInfo   = TagInfo { name: "diet:herbivore",   bit: 108, parent_bit: None };
pub const DIET_OMNIVORE: TagInfo    = TagInfo { name: "diet:omnivore",    bit: 109, parent_bit: None };
pub const DIET_PISCIVORE: TagInfo   = TagInfo { name: "diet:piscivore",   bit: 110, parent_bit: None };
pub const DIET_INSECTIVORE: TagInfo = TagInfo { name: "diet:insectivore", bit: 111, parent_bit: None };
pub const DIET_FRUGIVORE: TagInfo   = TagInfo { name: "diet:frugivore",   bit: 112, parent_bit: None };
pub const DIET_GRANIVORE: TagInfo   = TagInfo { name: "diet:granivore",   bit: 113, parent_bit: None };
pub const DIET_DETRITIVORE: TagInfo = TagInfo { name: "diet:detritivore", bit: 114, parent_bit: None };
pub const DIET_SCAVENGER: TagInfo   = TagInfo { name: "diet:scavenger",   bit: 115, parent_bit: None };
pub const DIET_NECTAR: TagInfo      = TagInfo { name: "diet:nectar_feeder", bit: 116, parent_bit: None };
pub const DIET_FILTER: TagInfo      = TagInfo { name: "diet:filter_feeder", bit: 117, parent_bit: None };
pub const DIET_WOOD: TagInfo        = TagInfo { name: "diet:wood_eater",  bit: 118, parent_bit: None };
pub const DIET_SANGUIVORE: TagInfo  = TagInfo { name: "diet:sanguivore",  bit: 119, parent_bit: None };
```

（以此类推，全部 15 个维度共 ~130 个标签常量）

### default_registry 注册

```rust
pub fn default_registry() -> Self {
    // ... 现有身体标签注册 ...
    
    // 性状标签注册（从 TAG_CONSTANTS 自动注册）
    for info in TAG_CONSTANTS.iter() {
        reg.register_tag(info);
    }
    // 自动计算 descendants
    reg.compute_descendants();
    reg
}
```

### 关键原则

- 不修改 TagBits / TagRegistry / TagQuery 类型 ✅
- 不加新模块、不引入新依赖 ✅
- 所有标签 name 使用 `category:value` 格式（如 `diet:carnivore`）✅
- bit 范围 100-230，远离身体标签（0-99）✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- TagRegistry 包含新标签（如 `diet:carnivore`）
