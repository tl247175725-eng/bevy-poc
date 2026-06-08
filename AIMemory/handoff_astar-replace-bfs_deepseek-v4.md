# 寻路根治：纯 GDScript BFS → Godot AStarGrid2D

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Priority**: HIGH

## 原因

`pathfinding.gd` 的 BFS 是纯 GDScript 实现，每 tick 每张自动卡都在调。加搜索上限、加盲向步降级都是治标——一碰边界条件反而更差（"更卡了"）。

根源是**寻路底层错了**。Godot 内置的 AStarGrid2D 是 C++ 实现的网格寻路，就是为这个场景设计的。

## 改法

用 AStarGrid2D 替换 find_path 的纯 BFS 实现。

### 核心思路

1. 在 `pathfinding.gd` 或一个新 autoload 中维护一个 `AStarGrid2D` 实例
2. 每 tick 开始时（或寻路调用前），把当前有阻挡卡的所有格子标记为 `solid`
3. `find_path` 调用 `astar.get_id_path()` 获得路径
4. 返回格式保持现有的 `[{x, y}, ...]` 格式，不动调用方

### 关键点

- AStarGrid2D 的 `cell_size` 与现有 `CELL_SIZE=56` 对齐
- `region` 设为 `Rect2i(0, 0, 36, 24)`
- `diagonal_mode` 设为 `DIAGONAL_MODE_NEVER`（保持现有四方向）
- 阻挡物更新：调用 `astar.set_point_solid(x, y, true)` 给有阻挡卡的格子
- 玩家/动物自格不算阻挡（通过 `actor_id` 排除，复用现有 `is_blocked_for` 逻辑）

### 删除

- 旧 `find_path` 的 BFS while 循环
- 上轮加的 `MAX_SEARCH` 和 `move_one_blind_step`
- 如果有 `move_one_step_toward` 到 `move_one_step_near` 的递归调用

## 验收

- 帧率持续 ≥2.7 tick/s
- 动物移动行为不变（路径计算方式换了，但结果同构）
- 不再需要任何搜索深度限制或降级策略

---

# 回复

**From**: cursor | **Date**: 2026-05-30 | **Status**: DONE

- `pathfinding.gd`：`AStarGrid2D`（36×24、四向、CELL_SIZE）；每游戏 tick `begin_tick` 刷新占位；`find_path` 仍返回 `[{x,y},…]`。
- 已删 BFS、`MAX_SEARCH`、`move_one_blind_step_toward_xy`。
