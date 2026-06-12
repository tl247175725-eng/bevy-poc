# 提纯 Phase 1：建立元数值和元动作基础

**Priority**: P0 — 整个整改的地基

## 架构计划

当前代码的底层没有"量纲"和"动作基元"。所有数字是硬编码的，所有行为分散在 if-else 链中。本次改动建立两层基础，不删任何现有代码——纯增量。

### 新增文件 1: `src/meta_values.rs`

定义世界的基础量纲。所有后续代码中的数字必须从此派生。

```rust
// 时间
pub const TICK_SECONDS: f32 = 0.5;
pub const TICKS_PER_SECOND: f32 = 2.0;
pub const SECONDS_PER_MINUTE: u32 = 60;
pub const MINUTES_PER_HOUR: u32 = 60;
pub const HOURS_PER_DAY: u32 = 24;
pub const TICKS_PER_DAY: u64 = (TICKS_PER_SECOND * SECONDS_PER_MINUTE as f32 * MINUTES_PER_HOUR as f32 * HOURS_PER_DAY as f32) as u64;
// = 2 * 60 * 60 * 24 = 172800

// 空间
pub const GRID_CELL_SIZE: f32 = 1.0;

// 物质
pub const BASE_HP: i32 = 1;
pub const BASE_ENERGY: u32 = 1;

// 体型 → 重量映射
pub fn size_to_weight(size: u8) -> u8 {
    match size {
        1 => 1,   // tiny
        2 => 3,   // small
        3 => 5,   // medium
        4 => 8,   // large
        _ => size * 2,
    }
}

// 体型 → 速度修正
pub fn size_to_speed_mod(size: u8) -> f32 {
    match size {
        1 => 1.5,  // tiny → 快
        2 => 1.2,  // small
        3 => 1.0,  // medium → 基准
        4 => 0.7,  // large → 慢
        _ => 1.0,
    }
}

// 基础移动速度（从元数值推导）
pub const BASE_MOVE_SPEED: f32 = GRID_CELL_SIZE; // 1 cell / tick
pub fn entity_move_speed(size: u8) -> f32 {
    BASE_MOVE_SPEED / size_to_speed_mod(size)
}

// 基础冲刺速度
pub fn entity_sprint_speed(size: u8, sprint_tier: f32) -> f32 {
    BASE_MOVE_SPEED / (size_to_speed_mod(size) * sprint_tier)
}

// 撞击伤害（从重量+速度推导）
pub fn impact_damage(weight: u8, speed: f32) -> i32 {
    (weight as f32 * speed).ceil() as i32
}
```

### 新增文件 2: `src/meta_actions.rs`

定义不可分解的行为基元枚举。

```rust
use crate::spatial_index::EntityId;

/// 不可分解的原子动作。所有复杂行为 = 元动作的组合。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaAction {
    /// 曼哈顿单步移动
    Move { dx: i16, dy: i16 },
    /// 对目标施加打击
    Strike { target: EntityId },
    /// 消耗目标（转化为自身能量）
    Consume { target: EntityId },
    /// 将自身与目标合并为新物
    Combine { ingredient: EntityId },
    /// 将携带物放置到世界
    Release { x: u8, y: u8 },
    /// 维持原状 N tick
    Wait { ticks: u64 },
    /// 进入容纳态（不占格）
    Hide { cover_id: EntityId },
    /// 从容纳态退出
    Emerge,
}

/// 元动作执行结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionResult {
    Success,
    Blocked { reason: String },
    Invalid,
    Consumed { energy_gained: u32 },
    Killed { corpse_spawned: bool },
}
```

### 注册到 lib.rs

在 `src/lib.rs` 中添加：
```rust
pub mod meta_values;
pub mod meta_actions;
```

### 测试

`src/meta_values.rs` 底部加：
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_per_day_is_derived() {
        // 2 tick/s * 60s * 60min * 24h
        assert_eq!(TICKS_PER_DAY, 172800);
    }

    #[test]
    fn speeds_are_not_magic() {
        let tiny_speed = entity_move_speed(1);   // size 1 = tiny
        let medium_speed = entity_move_speed(3); // size 3 = medium
        assert!(tiny_speed > medium_speed, "tiny things should be faster");
        assert!(tiny_speed > 0.0);
        assert!(medium_speed > 0.0);
    }

    #[test]
    fn impact_damage_is_derived() {
        let dmg = impact_damage(5, 1.0); // weight 5, speed 1
        assert_eq!(dmg, 5);
    }
}
```

## 智能验收

- `cargo test -- --nocapture` 全部 PASS（新增测试 + 原有测试全部通过）
- `cargo build` 成功
- `meta_values.rs` 中所有 pub fn 的数字来自 const 计算，无裸数字
- `meta_actions.rs` 中的 enum 覆盖我们讨论过的 8 个元动作
- `lib.rs` 正确注册了两个新模块

## 涉改文件

| 文件 | 改动 |
|---|---|
| `src/meta_values.rs` | **新建** — 元数值定义 + 派生函数 + 测试 |
| `src/meta_actions.rs` | **新建** — 元动作枚举 + 执行结果枚举 |
| `src/lib.rs` | 注册两个新模块 |

## 设计文档引用

- `design-philosophy-v5.md` §4 — 元动作与元数值
- `design_machine-readable_v4.md` §1 — 一格一卡、曼哈顿移动
