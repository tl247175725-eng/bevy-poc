# BFS 寻路优化：搜索上限 + 简单步降级

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Priority**: HIGH

## 问题

`pathfinding.gd` 的 `find_path` 没有搜索深度上限。BFS 会穷举整个 36×24=864 格。当路径被河流阻挡时，全网格展开零回报，白耗 CPU。

## 修复

### A. BFS 加搜索上限

`find_path` L14 while 循环加越界保护：

```gdscript
const MAX_SEARCH = 40  # 最多探索 40 格
var searched := 0
while q.size() > 0 and searched < MAX_SEARCH:
    ...
    searched += 1
```

如果超出 MAX_SEARCH 还没找到目标 → 返回 []（等价于"路径太长/不可达"）。

40 格相当于约 10-15 步直线距离，足够覆盖绝大多数寻路场景（狼找猎物、羊找草、鼠找灌木）。

### B. move_one_step_near 降级

当 `find_path` 返回空时，调用方改用 `move_one_step_toward`——不做 BFS，只向目标方向走一步。这是 O(1) 的简单格计算，不消耗 CPU。走一步之后下一 tick BFS 可能因为距离缩短而成功。

修改 `world_helpers.gd` 的 `move_one_step_near`：
```
path 空 → move_one_step_toward(card, target)
```

## 验收

- 帧率至少恢复到持续 ≥2.5 tick/s
- 动物行为不变（只是路径不可达时改走盲向步）

---

# 回复

**From**: cursor | **Date**: 2026-05-30 | **Status**: DONE

- `pathfinding.gd`：`MAX_SEARCH=40`，超出返回 `[]`。
- `world_helpers`：`find_path` 空时 `move_one_blind_step_toward_xy`（`move_one_step_toward` / `move_one_step_toward_xy`）；去掉 toward→near 递归。
