# 相邻格交互 — 修复游戏卡死

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-09
**Priority**: P0（游戏完全不运行——compose 拒绝进入后没实现相邻交互）

---

## 根因

compose 改为默认一格一卡后，`move_toward` 无法进入任何已占格。但捕猎/进食/吃草仍要求 `x == target_x && y == target_y`（同格）→ 所有动物卡死。

## 修复

### 1. tick_reactive — try_consume 改为相邻攻击

当前：`move_toward(id, x, y, tx, ty)` → 走到目标格上 → `try_consume(id, target_id)`

改为：到达目标**相邻格**（chebyshev_distance == 1）即可 `try_consume(id, target_id)`。不需要同格。

```rust
// execute_drive 中 Seek/Flock 到达后的判断
let dist = chebyshev_distance(x, y, tx, ty);
if dist <= 1 {
    // 到达相邻格或同格 → 可以交互
    try_consume(world, id, target_id);
} else {
    move_toward(world, id, x, y, tx, ty);
}
```

### 2. tick_herbivore — try_eat_grass 改为相邻进食

当前：`x == gx && y == gy` → eat

改为：`chebyshev_distance(x, y, gx, gy) <= 1`

### 3. tick_predator 遗留引用检查

`try_hunt` / `try_scavenge` 如果也有 `x == target_x` 判断 → 同样改为 `<= 1`

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/systems/tick_reactive.rs` | try_consume 相邻攻击 |
| `src/systems/tick_herbivore.rs` | try_eat_grass 相邻进食 |
| `src/systems/tick_predator.rs` | 如有遗留的 try_hunt/try_scavenge 同格检查 |

## 验收

启动游戏 → 动物开始移动、进食、捕猎，不卡死
