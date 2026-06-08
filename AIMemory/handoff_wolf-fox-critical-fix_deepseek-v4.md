# 紧急修复：狼被移除 + 狐狸筑窝卡死

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Type**: TASK_ASSIGN
**Priority**: HIGH

## 问题 1：狼在封闭模式下仍被移除

**根因**：`wolf_behavior.gd` L59 `should_wolf_leave` 调用没有 `ECOSYSTEM_CLOSED_MODE` 守卫。整包迁出封了，但近窝溢出踢出没封。

**触发条件**：狼建了窝、在窝边（3×3）、窝边成年狼超过 2 只 → 立即 remove_card。

**修复**：`wolf_behavior.gd` L59 加 `ECOSYSTEM_CLOSED_MODE` 守卫：
```
if not GameState.ECOSYSTEM_CLOSED_MODE and host.should_wolf_leave(wolf):
```

封闭模式下不靠容量溢出踢狼，靠饥饿个体死亡（已有逻辑）。

## 问题 2：狐狸筑窝全死锁

三个缺陷形成死锁：

### A. `_ensure_bush_for_fox_builder` 逻辑错误（`ecosystem_manager.gd` L558-563）
`_nearest_bush_for_fox_den` 检查"灌木是否存在"——只要 map 上有灌木就返回，不管是否可达。然后 `_ensure_bush_for_fox_builder` 被短路，永远不补生。

**修复**：`_nearest_bush_for_fox_den` 改为只返回**可达**的灌木（需要 `find_path` 成功或至少路径存在）。如果所有灌木都不可达 → 返回 null → 触发 `_ensure_bush_for_fox_builder` 在狐狸身边补生新灌丛。

或者更简单的修复：在 `update_fox_build_den` 的"占灌木"分支里，如果 `move_one_step_near` 失败，随机向一个方向走一步 + 等下一 tick 再找灌木。

### B. `update_fox_build_den` 不检查移动结果（`ecosystem_manager.gd` L554）
`move_one_step_near` 寻路失败后静默 return，不换目标、不报错。

**修复**：`move_one_step_near` 失败时，清掉当前选择的灌木引用（或随机移动一步），让下一 tick 重新选目标。

### C. 无"占灌木"状态的超时退出
狐狸可以永久卡在"占灌木"分支。需要增加超时：连续 N tick 未能到达目标灌木 → 换一个灌木或进 fallback。

## 验收

- 新开一局，跑 500+ tick，检查 session_report：
  - 成狼不应被"移除"（状态"筑巢"/"搬肉"）
  - 狐狸应成功筑窝或至少开始移动

---

# 回复

**From**: cursor | **Date**: 2026-05-30 | **Status**: DONE

1. `wolf_behavior`：`should_wolf_leave` 加 `ECOSYSTEM_CLOSED_MODE` 守卫（封闭模式不迁出）。
2. `ecosystem_manager`：`_nearest_bush_for_fox_den` 仅可达灌丛；`move_one_step_near` 失败随机步 + 45 tick 换目标；无可达时补生灌丛。
3. L0 741 通过。请 F5 跑 500+ tick 后关游戏，核对 `session_report` 狼无异常「移除」、狐有移动/筑窝。
