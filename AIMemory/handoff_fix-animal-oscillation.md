# 动物振荡修复 — 喂食后行为断裂

**Priority**: P0（所有动物无生产性行为，2格来回移动）

## 根因诊断

### 链条 A：fed_today 锁死 Seek → 草食动物无事可做

1. `active_drives` 生成 Seek 驱动时 `condition_fed=true`，当 `entity.fed_today=true` 时整条 Seek 驱动被 `continue` 跳过
2. `fed_today` 只在 `tick_starvation()` 中每天一次重置（480 ticks/天）
3. 草食动物吃一次草后，后续 ~470 ticks 内 Seek 驱动完全不生成
4. 结果：草食动物吃完就无行为驱动

### 链条 B：草食动物无后备行为 → 空转睡眠

- `select_utility_drive` 返回 `None` 时：
  - **捕食者** 有 `wander` 兜底（tick_reactive.rs:325-329）
  - **草食动物** 直接 `Idle` → `try_enter_sleep`（tick_reactive.rs:332-336）
- 草食动物在 fed_today 期间：Idle → 睡眠5tick → 被邻格动物唤醒 → Idle → 再睡 → 循环
- 外观：动物"抽搐"——醒了站一下又睡

### 链条 C：Flock 驱动导致中心聚集振荡

- 三只以上同种动物在 Flock range 内时，Flock 驱动激活
- 所有成员向重心移动 → 在中心碰撞 → compose 阻止同格 → 位移到邻格
- 下一 tick：再向重心移动 → 再碰撞 → 再位移 → 2格来回移动
- Flock 的 `ecology_state` 被设为 `SeekingFood`（误导标签）

### 链条 D：Wander 方向严重偏斜

- `wander()` 函数非边缘时只尝试右(1,0)或下(0,1)，从不向左/上
- `rng_coin_for(salt)` 每 tick 交替奇偶 → 右/下交替 → 阶梯式漂移

### 链条 E：flatten_ground_cover_at 破坏草皮

- Medium 高度动物走上草格即摧毁草（`world_state.rs:553-578`）
- 白羊吃草前可能被途经的其他动物踩毁
- 减少可用食物 → 加剧竞争 → 更多重定向

---

## 架构计划

### 修复 1：所有自主实体统一 wander 后备（改 tick_reactive.rs）

**位置**: `select_utility_drive` 返回 `None` 后的分支（tick_reactive.rs:319-336）

**当前**:
```rust
} else if card_has_tag(&def, "predator") || card_has_tag(&def, "mesopredator") {
    wander(world, id, x, y, world.tick_count);
    // ...
} else {
    ecology_state = Idle;
    false  // → try_enter_sleep
}
```

**改为**: 所有实体（不只是捕食者）无驱动时走 wander：
```rust
} else {
    wander(world, id, x, y, world.tick_count);
    if let Some(e) = world.entities.get_mut(&id) {
        e.ecology_state = EcologyState::Wandering;
    }
    true  // executed_drive = true → 不睡眠
}
```

**注意**: 仍然保留 `try_enter_sleep` 的调用，但只在 ecology_state 为 Idle 时才进入睡眠（现有检查 `tick_reactive.rs:361-364` 已要求 Idle|Wandering 状态）。改为 Wandering 后实体不会立即睡——它们会漫游直到饥饿，然后 Seek 重新激活。

### 修复 2：Wander 四方向随机（改 movement.rs:wander）

**位置**: `src/systems/movement.rs` `wander()` 函数

**当前**: 只尝试右(1,0)和下(0,1)，fallback 左(-1,0)和上(0,-1)

**改为**: 四个曼哈顿方向中等概率随机选择

```rust
pub fn wander(world: &mut WorldState, id: EntityId, x: u8, y: u8, tick: u64) {
    let salt = tick.wrapping_add(id.0 as u64);
    // 用 salt 从四个方向中选一
    let dirs = [(1i16, 0i16), (-1, 0), (0, 1), (0, -1)];
    let idx = (salt % 4) as usize;
    let (step_dx, step_dy) = dirs[idx];
    if step_dx < 0 && x == 0 { return; }
    if step_dy < 0 && y == 0 { return; }
    if step_dx > 0 && x >= GRID_WIDTH - 1 { return; }
    if step_dy > 0 && y >= GRID_HEIGHT - 1 { return; }
    let _ = attempt_move_with_resolution(world, id, x, y, step_dx, step_dy);
}
```

### 修复 3：Flock 重心重合时不移动（改 tick_reactive.rs:active_drives Flock 段）

**位置**: `active_drives` 中 `DriveBehavior::Flock` 段

**当前**: 有 ≥2 mates 即生成 Flock 驱动，不检查自身是否已在重心

**改为**: 计算重心后，若 `chebyshev_distance(x, y, avg_x, avg_y) == 0`（已在重心），不生成该驱动

### 修复 4：Flock ecology_state 改为 Idle（改 tick_reactive.rs:execute_drive Flock 段）

**位置**: `execute_drive` `DriveBehavior::Flock` 分支

**当前**: `e.ecology_state = EcologyState::SeekingFood`

**改为**: 删除此行（Flock 移动后 state 由 move_toward 内部的 on_move 设定），或设为 `Wandering`

### 修复 5（可选但建议）：flatten_ground_cover_at 条件放宽

`move_entity` 中每次移动都调用 `flatten_ground_cover_at`，这导致动物走到草格上时草被摧毁。改为**仅在进食成功时**摧毁草（已在 `try_consume` 中处理 hp-- 逻辑），`flatten_ground_cover_at` 改为仅对 High 高度动物踩踏时触发（如大象级别）。

当前实现参考: `world_state.rs:553-578` + `world_state.rs:481`（调用点）

改为 `incoming_height >= Height::High` 才触发踩踏。

---

## 架构反馈

1. **fed_today 作为锁的粒度太粗**: 当前设计是"一天吃一次"，但 480 tick 无行为太长。应该让动物在"不饿"状态下仍有探索/社交/巡游行为。修复 1 补上了 wander 兜底，但长远应考虑减少 fed_today 的锁定时长或改为"饱食度"渐进系统。

2. **Flock 驱动缺少死区**: Flock 设计来自 boids 模型，但 boids 有分离力防止碰撞。当前实现只有聚合（向重心移动），没有分离。碰撞由 compose 硬阻止，造成振荡。修复 3 加了重心死区，但理想方案是引入分离力或 flock 内位置协商。

3. **wander 的随机性**: 原来的 wander 使用 `rng_coin_for` (tick*magic & 1) 生成方向，这在实体密度低时产生可见的右/下偏斜模式。修复 2 用 salt%4 从四方向均匀随机选，消除偏斜。但长期应使用更丰富的随机源（如实体专属 RNG）。

4. **flatten_ground_cover_at 概念混淆**: 这个函数名字暗示"踩平"，但实际上直接移除实体。草被踩应该降 HP 而非直接消失。但考虑到一格一卡规则，Low 以下实体被踩符合设计意图（草不高，动物踩上去就没了）。修复 5 仅在 High 高度触发是合理折中。

---

## 智能验收

### 验收 A: 草食动物吃完草后不睡眠/空转
```
启动游戏 → 观察羊/兔吃草后行为
断言: 吃完后动物仍移动（Wandering），不是站着不动（Idle）或瞬间睡眠
断言: 5分钟内动物状态分布中 Idle 占比 < 30%
```

### 验收 B: 动物不在 2 格间来回振荡
```
启动游戏 → 运行 2 分钟 → 观察任意一群动物
断言: 同种动物群不出现 (x,y) ↔ (x±1,y) 或 (x,y) ↔ (x,y±1) 的持续 2 格循环
断言: 捕食者不沿右/下方向阶梯式漂移
```

### 验收 C: Flock 重心重合时不移动
```
构造测试场景: 3 只羊在 (5,5), (6,5), (5,6)（重心 = (5,5) 取整）
断言: (5,5) 处的羊不生成 Flock 驱动（已在重心）
断言: (6,5) 和 (5,6) 处的羊生成 Flock 并向 (5,5) 移动
```

### 验收 D: 草地不被动物踩毁
```
启动游戏 → 让动物在草地上自由行动 1 分钟
断言: 动物经过的草地格，草仍然存在（不是吃掉的，是经过的）
断言: grass 实体数量不因动物移动而减少（排除被吃掉的）
```

### 验收 E: 编译 + 测试
```
cargo test -- --nocapture 全部 PASS
cargo build 成功
游戏可启动运行 无 panic
```

---

## 涉改文件

| 文件 | 改动 |
|---|---|
| `src/systems/tick_reactive.rs` | 修复 1（wander 兜底）、修复 3（Flock 死区）、修复 4（Flock state） |
| `src/systems/movement.rs` | 修复 2（wander 四方向随机） |
| `src/world_state.rs` | 修复 5（flatten_ground_cover_at 条件） |

## 设计文档引用

- `design_machine-readable_v4.md` §1.1 一格一卡（compose 规则不因本次修复改变）
- `design_flock-system_deepseek-v4.md`（Flock 是 utility drive 行为，不是独立 boids 引擎）
- `AIMemory/design_human-readable_v4.md` §5 驱动系统（utility scoring）
