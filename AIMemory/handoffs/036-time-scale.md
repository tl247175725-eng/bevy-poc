# Handoff 036 — 时间尺度改造：同构时间 + 同构衰减

## 三柱强制检查

| 柱子 | 用哪个 |
|---|---|
| 标签 | `body_size:*`(质量基础) + `metab:*`(代谢率) |
| 元数值 | `TICKS_PER_DAY`(核心时间常数) + `nutrition_decay_per_tick()`(新增函数) |
| 元动作 | 不涉及 |
| 公理 | 不涉及 |

## 架构计划

**改什么：** `src/meta_values.rs` + `src/initial_spawn.rs` + `src/render/skybox.rs`（3 文件）

**为什么：** 当前一天 3.5 分钟太短，改为 17.5 分钟。同时 decay_rate 从裸数字改为同构推导。

### 改动 1：`src/meta_values.rs`

```rust
// 时间尺度
pub const TICKS_PER_DAY: u64 = 2100;     // 17.5 分钟一天
pub const TICKS_PER_PHASE: u64 = 300;    // 2100/7
pub const TICK_SECONDS: f32 = 0.5;

// 代谢→衰减率推导（同构：从每日能量需求算每tick衰减）
pub fn nutrition_decay_per_tick(metab_rate: f32) -> f32 {
    // 原理：代谢率越高兴奋越快
    // 中代谢动物约1游戏天从饱到饿
    // 公式：decay = metab_rate / (TICKS_PER_DAY * TICK_SECONDS)
    metab_rate / (TICKS_PER_DAY as f32 * TICK_SECONDS)
}

// 代谢率从 metab 标签推导
pub fn metab_rate_from_tags(tags: &TagBits) -> f32 {
    if tags.has(tag::METAB_HIGH) { return 1.5; }
    if tags.has(tag::METAB_LOW)  { return 0.5; }
    1.0  // medium 默认
}
```

删除旧的裸数字常量：
```rust
// ❌ 删除
NUTRITION_DECAY_HIGH/MEDIUM/LOW
SOCIAL_DECAY
CURIOSITY_DECAY
```

改为：
```rust
pub const SOCIAL_DECAY_RATE: f32 = 0.1 / (TICKS_PER_DAY as f32 * TICK_SECONDS);
pub const CURIOSITY_DECAY_RATE: f32 = 0.05 / (TICKS_PER_DAY as f32 * TICK_SECONDS);
```

测试更新：
- `ticks_per_day_is_420` → `ticks_per_day_is_2100`
- `ticks_per_phase_is_60` → `ticks_per_phase_is_300`

### 改动 2：`src/initial_spawn.rs`

`metab_decay_rate()` 替换为调用 `nutrition_decay_per_tick(metab_rate)`，需要先算出 metab_rate：

```rust
fn metab_decay_rate(def: &CardDef) -> f32 {
    let metab = crate::meta_values::metab_rate_from_tags(&def.tag_bits);
    crate::meta_values::nutrition_decay_per_tick(metab)
}
```

social 和 curiosity decay 引用新的 `SOCIAL_DECAY_RATE` 和 `CURIOSITY_DECAY_RATE`。

### 改动 3：`src/render/skybox.rs`

```rust
let ticks_per_day = 420u64;  // ❌
// →
let ticks_per_day = crate::meta_values::TICKS_PER_DAY;  // ✅
```

## 架构反馈

1. **时间密度分离：** 生物需求速度从标签+体重+代谢率推导，和时间步长无关。改时间 = 改衰减分频，不改总能量需求——生物水平不变
2. **裸数字第二次清扫：** NUTRITION_DECAY_* 常量被公式替代，SOCIAL/CURIOSITY 改为 TICKS_PER_DAY 分母下的常量

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- `nutrition_decay_per_tick(1.0)` 返回值 ≈ 1/1050 ≈ 0.00095
- 测试 `ticks_per_day_is_2100` 通过
- skybox.rs 不再硬编码 420
