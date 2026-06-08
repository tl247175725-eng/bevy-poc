# 狐狸行为修复：捕猎迁移后剩余分支接回

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Priority**: HIGH

## 问题

步骤 A（capability.hunt 迁移）后：
- 狐狸显示"捕猎"状态但 0 杀（498 tick），无法靠近/击杀猎物
- 筑窝逻辑未执行（0 狐窝）
- §7 诊断仍为空

## 修复

### 1. FoxBehavior.tick 恢复全部原有分支

确认 `fox_behavior.gd` 的 tick 函数中，捕猎之外的逻辑全部正常运行：

```
进 tick → from_card → 眩晕检查 → 窝内休整 →
躲人 → 惧火 → 躲狼 → 叼肉回窝 →
筑窝判定（den == null → update_fox_build_den）→
清道夫（try_fox_scavenge）→ 冷却 →
move_tick → 捕猎（走 WorldRules.tick_capability_hunt 共享片段）
```

每个分支的 `return` 都必须到位。关键是**捕猎是最后一个分支，之前的筑窝/清道夫/躲狼不能被跳过**。

### 2. capability.hunt 片段内确认 `consume_move_tick`

确保 `tick_capability_hunt` 内部正确调用了 `WorldHelpers.consume_move_tick(actor, real_delta)`。狐狸"捕猎"不动 496 tick 很大概率是这个检查一直返回 false。

### 3. 修复 §7 诊断

狐 tick 诊断的 `record_fox_tick` 调用可能在上次迁移中丢失。确认在 `FoxBehavior.tick` 的每个关键分支前都有诊断记录。

## 验收

- 狐狸 §5 捕食者详情中猎杀 > 0
- 母狐在若干 tick 后进入筑窝流程（状态变为"占灌木"或"寻灌丛"）
- §7 有诊断记录

---

# 回复

**From**: cursor | **Date**: 2026-05-30 | **Status**: DONE

- 删除 `FoxBehavior` 顶部过早 `consume_move_tick` return，筑窝/清腐/躲狼/冷却恢复。
- `tick_capability_hunt`：先选猎物再 `consume_move_tick`，避免假「捕猎」僵住；§7 经 `_fox_diag` + 捕猎片段 `_record_fox_hunt_diag`。
