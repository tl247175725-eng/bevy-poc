# 族群封装 + 统一移速

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（架构修正——群必须是一张卡，不是 N 个个体叠在一起）

---

## 核心设计

### 群 = 一张 Entity

同种 ≥2 只个体在相邻/同格范围内 → **合并为一张群卡。** 群卡是一张独立的 Entity，`herd_count: N`。

群卡不是"N 个 Entity 加个视觉遮罩"。群卡就是一张卡，参与一切系统。

### 剥离

| 触发 | 效果 |
|---|---|
| 捕食者成功攻击群 | 随机击杀 1 只，herd_count -= 1。降到 1 → 变回单卡 |
| 惊散（predator 触发） | 群裂为 N 张单卡，各自 flee |
| 单卡加入 | 同种单卡走到同格 → herd_count += 1 |

### 公理兼容

| 定律 | 群卡行为 |
|---|---|
| compose | 占 1 格（和单卡一样） |
| perceive | 视距 ×1.5（一群比一只更容易被看到） |
| transform | 狼 kill 群 → 随机杀 1 只，能量按 1 只计算 |

---

## 移速系统

### 所有卡基速统一

`move_speed: normal` → 0.25s/格

### 冲刺分档

只在 **捕猎追击** 或 **惊逃跑路** 时触发冲刺速度。平时归零。

| 档位 | 秒/格 | 标签 |
|---|---|---|
| 慢冲 | 0.18 | `sprint:slow` |
| 常冲 | 0.12 | `sprint:normal` |
| 快冲 | 0.08 | `sprint:fast` |
| 爆发 | 0.05 | `sprint:burst` |

| 物种 | 冲刺档 |
|---|---|
| 羊、鹿、水牛 | `sprint:slow` |
| 狼 | `sprint:normal` |
| 狐、兔 | `sprint:fast` |
| 猎豹（后续） | `sprint:burst` |

---

## 实现

### 合并逻辑

在 `tick_reactive` 的 `drive:flock` 分支：同格有 ≥2 只同种 → 合并。

```rust
// 合并: 如果同格有同种非群个体
let same_cell: Vec<EntityId> = world.entities_at(x, y).iter()
    .filter(|eid| *eid != id)
    .filter(|eid| world.entities[eid].type_name == my_type)
    .filter(|eid| world.entities[eid].herd_count == 0) // 非群
    .copied()
    .collect();

if same_cell.len() >= 1 {
    let count = 1 + same_cell.len();
    // 移除所有单卡
    for eid in &same_cell { world.remove_entity(*eid); }
    world.remove_entity(id);
    // 创建群卡
    let herd_id = world.spawn_herd(type_name, x, y, count as u8);
    return;
}
```

### 剥离逻辑（惊散）

```rust
// 惊散: predator 在 flock_alert_range 内
let herd = world.entities[&id];
let count = herd.herd_count;
world.remove_entity(id);
for i in 0..count {
    let new_id = world.spawn(type_name, x, y);
    // 给每个新个体设 scatter_timer
    if let Some(e) = world.entities.get_mut(&new_id) {
        e.scatter_timer = 5; // 5 tick 散开
    }
}
```

### Entity 新增字段

```rust
pub herd_count: u8,   // 0 = 不是群卡；>0 = 群卡，包含 N 只
```

### 移速引擎

`EntityProfile.move_speed` 始终 0.25。
新增 `EntityProfile.sprint_speed: f32`，只在 seek 目标时触发（追击）或 flee 时触发（逃命）。

`execute_drive` 中：
- Seek/Flee 驱动 → 使用 `profile.sprint_speed`
- 其他驱动 → 使用 `profile.move_speed`

---

## 简化：移除旧 boids 系统

删除以下代码/标签：
- `execute_flock`（separation/cohesion/alignment 三力模型）→ 替换为合并/剥离
- `flock_cohesion`, `flock_separation` 标签 → 不再需要
- `flock_alert: startle|scatter|stampede|school` → 统一为惊散=裂成单卡
- `flock_max` → 替换为 `herd_max`（群卡最大数量）

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/world_state.rs` | Entity 加 `herd_count`；新增 `spawn_herd` |
| `src/systems/tick_reactive.rs` | 合并/剥离逻辑替换 execute_flock；冲刺速度 |
| `src/axioms/profile.rs` | 新增 `sprint_speed`；简化标签解析 |
| `src/axioms/laws.rs` | perceive 群卡视距 ×1.5 |
| `src/card_visual.rs` | 群卡视觉已支持（GroupCardMarker） |
| `assets/card_defs.ron` | 移除旧 flock 标签，加 sprint 标签 |
| `AIMemory/design_flock-system_deepseek-v4.md` | 更新设计文档 |

## 验收

1. `cargo test --release` 全 PASS
2. `cargo run --release -- --smoke-test` SMOKE: PASS
3. 启动游戏：
   - 同种动物靠近 → 合并为群卡（数量标注）
   - 狼攻击群 → 数量 -1，不爆 N 张尸体
   - 惊散 → 群裂成单卡各自跑
   - 所有动物平时移速一致
   - 追击/逃命时速度不同

## 约束

- 不碰公理层 compose/traverse/perceive（除 perceive 群卡加成）
- 不碰 Player AI
- 不碰地形/初始生成
