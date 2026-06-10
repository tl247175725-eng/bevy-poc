# 曼哈顿移动 + 三层碰撞解决

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（基础移动——目前仍是八方向斜走）

## 架构计划
修改 `move_toward` 和 `flee_from` 为曼哈顿（随机选单轴）。在 `move_entity` 前后加入三层碰撞解决。全部走现有 compose/traverse 公理。

## 架构反馈
移动系统需新增 `entity_priority` 概念——可能需要在 `EntityProfile` 或 `ActiveDrive` 中暴露当前行为优先级。公理层 compose 不变，碰撞解决在移动层处理。

## 智能验收
- 断言：任何 `move_toward` 调用的 `dx` 和 `dy` 不同时为非零（无斜走）
- 断言：smoke test 中 herbivore 移动数 > 100（曼哈顿不会导致动物卡死）
- 断言：两个面对面实体能通过 yield/shove 交换位置不卡死

---

## P0：曼哈顿移动

`src/systems/movement.rs` `move_toward`：

```rust
pub fn move_toward(world, id, x, y, tx, ty) {
    if x == tx && y == ty { return; }
    
    let dx = (tx as i16 - x as i16).signum();
    let dy = (ty as i16 - y as i16).signum();
    
    // Manhattan: never both non-zero. Random pick when diagonal.
    let (step_dx, step_dy) = if dx != 0 && dy != 0 {
        if world.rng_coin() { (dx, 0) } else { (0, dy) }
    } else {
        (dx, dy)
    };
    // ... rest of move logic with step_dx/step_dy
}
```

同样适用于 `flee_from` 和 `move_toward_greedy`。

---

## P0：三层碰撞解决

在 `move_toward` 中，当 `world.move_entity(id, gx, gy) == Blocked` 时触发：

### 第一层：Dodge（行动者绕行）

```rust
// 主方向被占 → 尝试垂直方向
let alts = [(step_dy, step_dx), (-step_dy, -step_dx)]; // 两个垂直方向
for (ax, ay) in alts {
    let alt_x = (x as i16 + ax).clamp(...) as u8;
    let alt_y = (y as i16 + ay).clamp(...) as u8;
    if world.move_entity(id, alt_x, alt_y) == Moved { return; }
}
```

### 第二层：Yield（阻挡者主动让路）

```rust
// 挡路者在 (gx, gy)，查询其当前行为优先级
let blocker_priority = get_entity_priority(world, blocker_id);
let mover_priority = get_entity_priority(world, id);

if blocker_priority < mover_priority {
    // 阻挡者优先级更低 → 尝试挤到旁边
    for (bx, by) in adjacent_cells(gx, gy) {
        if world.cell_composition.slot(bx, by).living_count == 0 {
            world.move_entity(blocker_id, bx, by); // 阻挡者让路
            world.move_entity(id, gx, gy);          // 行动者进占
            return;
        }
    }
}
```

### 第三层：Shove（强制推开）

```rust
// Yield 失败 + mover 优先级显著高于 blocker（差 ≥ 2）
if mover_priority - blocker_priority >= 2 {
    // 强制推 blocker 到最远空位
    let push_dir = (step_dx, step_dy); // 沿移动方向推
    let mut push_x = gx as i16 + push_dir.0;
    let mut push_y = gy as i16 + push_dir.1;
    // 找连续空位链
    while cell_is_empty(push_x, push_y) {
        world.move_entity(blocker_id, push_x as u8, push_y as u8);
        world.move_entity(id, gx, gy);
        return;
    }
}
```

### 优先级查表

```rust
fn get_entity_priority(world, id) -> u8 {
    let e = &world.entities[&id];
    match e.ecology_state {
        EcologyState::Fleeing => 5,
        EcologyState::Hunting => 4,
        EcologyState::SeekingFood => 3,
        EcologyState::Wandering => 2,
        EcologyState::Idle => 1,
        _ => 0,
    }
}
```

---

## Ranger 更新：`wander` 也要走曼哈顿

`wander` 当前只在 X 轴移动。保持曼哈顿兼容即可。

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/systems/movement.rs` | 曼哈顿 + 三层碰撞解决 |
| `src/world_state.rs` | `rng_coin()` 添加（如未存在） |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS
- `cargo run --release -- --smoke-test` SMOKE: PASS + herbivore 移动数 > 100
- 游戏内：动物不斜走、不被卡死、优先高的能挤开优先低的
