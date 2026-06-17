# Handoff 032 — 合并常量文件，消除重复定义

## 架构计划

**改什么：** `meta_values.rs` + `game_constants.rs` + 更新约 10 个文件的 import 路径

**为什么：** `game_constants.rs` 和 `meta_values.rs` 定义了同名但不同值的常量（TICK_SECONDS: 1.0 vs 0.5, TICKS_PER_DAY: 1440 vs 420）。两个"真理来源"直接矛盾。铁律：meta_values.rs 是唯一数值来源。

### 改动 1：`game_constants.rs` → 删除冲突常量，保留唯一常量

删除以下与 meta_values.rs 冲突的行：
```rust
pub const TICK_SECONDS: f32 = 1.0;           // meta_values 已有 0.5
pub const TICKS_PER_DAY: u64 = 86_400 / 60;  // meta_values 已有 420
```

保留其他唯一常量（约 60 个），但这些常量需要后续 handoff 逐步迁移到 meta_values 或 tags 系统。

### 改动 2：`src/sim_clock.rs` → 改引用路径

```rust
// 旧：crate::game_constants::TICK_SECONDS（3 处）
// 新：crate::meta_values::TICK_SECONDS
```

### 改动 3：`src/world_state.rs` → 改引用路径

```rust
// 旧：use crate::game_constants::TICK_SECONDS;
// 新：use crate::meta_values::TICK_SECONDS;
// 旧：crate::game_constants::TICK_SECONDS (line 181)
// 新：crate::meta_values::TICK_SECONDS
// 旧：crate::game_constants::BUSH_INITIAL_MICROFAUNA
// 新：crate::game_constants::BUSH_INITIAL_MICROFAUNA  // 不变，这个常量只在 game_constants
```

### 改动 4：`src/interaction/smash.rs` → 改引用

```rust
// 旧：crate::game_constants::TICK_SECONDS
// 新：crate::meta_values::TICK_SECONDS
```

### 其余引用文件（只引用 game_constants 唯一常量，无需改）

- `event_registry.rs` — WILDPREY_FEAR_RANGE（只在 game_constants）
- `selection_info.rs` — CORPSE_DECAY_SECONDS 等（只在 game_constants）
- `world_rules.rs` — PERISHABLE_TICKS（只在 game_constants）
- `tick_containment.rs` — CONE_PRODUCE_INTERVAL 等（只在 game_constants）
- `tick_harvest.rs` — POOL_HARVEST_REGEN_SECONDS（只在 game_constants）
- `tick_reproduction.rs` — 繁殖相关（只在 game_constants）

## 架构反馈

1. **消除了致命冲突**：TICK_SECONDS 和 TICKS_PER_DAY 不再有两个不同值
2. **game_constants.rs 降级**：从"第二个真理来源"降为"遗留生态常量暂存文件"——后续 handoff 逐步迁移
3. **meta_values.rs 成为唯一 A/B 层常量来源**

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- TICK_SECONDS 在整个 src/ 中只有 meta_values.rs 定义
- TICKS_PER_DAY 在整个 src/ 中只有 meta_values.rs 定义
- `sim_clock.rs` 的 `tick_accum` 逻辑不受影响（之前用 1.0 是因为 game_constants，现在统一到 meta_values 的 0.5）
