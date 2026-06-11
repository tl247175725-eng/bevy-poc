# 环境系统批量落地

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-11
**Priority**: P1

**参考设计**: `AIMemory/design_human-readable_v4.md` 第九部分 (9.1-9.3)

## 架构计划
标签驱动，不新建系统。温度用 SimClock 现有时间系统扩展。地块标签在 GridCell 上。衰老在 Entity.age 上。

## 架构反馈
SimClock 已有 season 字段基础。

## 智能验收
- 冬天水潭结冰卡
- 地块有不同肥力标签
- 动物有年龄，超限自然死亡

---

## P0-1：温度与季节

### SimClock 扩展

```rust
pub struct SeasonInfo {
    pub current: Season,
    pub day_of_year: u16,
    pub temperature: f32, // 当前温度，受季节+海拔影响
}

pub enum Season { Spring, Summer, Autumn, Winter }
```

### 温度影响

- 冬天：全局温度 -20。水潭表面生成冰卡
- 夏天：全局温度 +30。湿地水蒸发加速
- 海拔每 -100 → 温度 +1（水潭底部更冷？由设计决定）

### 冰卡

```
条件：格是 pool 且温度 ≤ 0
生成：冰卡覆盖水格（in_tree 逻辑，不占格？不——冰卡是一张可互动的实体卡）
破坏：被砸 2 次 → 碎 → 露出水格
采集：冰可被搬运到篝火旁融化 → 变水
```

## P0-2：地块多样化

### 地块标签初始化

在 `ensure_map_ecology` 中根据地形类型分配土壤标签：

```
pool 周边 2 格 → soil:rich, fertility:high（湿地+沃土）
pool 周边 3-6 格 → soil:wet, fertility:high（湿地）
大树 1 格内 → soil:loose, shaded（林地）
山周边 2 格 → soil:rocky, fertility:none（石地）
远离水域（5 格+）→ soil:dry, fertility:low（旱地）
随机少数格 → soil:deep, fertility:high（深土）
其余 → 不设特殊标签（默认 land）
```

## P0-3：衰老

### Entity 扩展

```rust
pub max_age: f32,  // 从标签 max_age:N 读取（天数）
pub age: f32,      // 已有
```

### 标签

```
max_age:3650   → 约 10 游戏年
max_age:730    → 约 2 年（小型动物）
max_age:18250  → 约 50 年（人类/大型动物）
```

### 衰老逻辑

```
每 tick：age += 1/TICKS_PER_DAY

当 age > max_age × 0.7：
  添加 trait:frail（移动慢 30%）
  添加 trait:wise（经验修正，仅人类）
当 age > max_age：
  finalize_prey_kill（自然死亡）
```

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/sim_clock.rs` | SeasonInfo + 温度计算 |
| `src/world_state.rs` | Entity 加 max_age；地块标签初始化 |
| `src/terrain_ecology.rs` | ensure_map_ecology 加土壤标签分配 |
| `src/systems/tick_environment.rs` | 冬天结冰 + 衰老检查 |
| `assets/card_defs.ron` | 动物加 max_age 标签 |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS + smoke PASS
- 冬天水潭上有冰卡
- 地块有不同肥力
- 动物超最大年龄后死亡
