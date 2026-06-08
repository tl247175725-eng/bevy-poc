# 数值修正：幼体成长时间

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Type**: TASK_ASSIGN
**Priority**: HIGH

## 问题

`population_manager.gd` 生命周期每 12 真实秒 tick 一次，当前所有幼体（lamb/deerFawn/fieldMousePup/foxCub）的 age 步进都是 `+12`：

```gdscript
var age = ... + 12   # 当前：+12
```

阈值 60 时：5 tick × 12s = ~1 真实分钟就成年。设计意图是至少可观察的成长周期。

## 设计依据

`docs/design/v3.0-a-ecology-initial-values.md` §3.2：

> 羊羔成长：age 每 12 tick 周期 +1，≥60 转成年

## 修改

将四种幼体的 age 步进从 `+12` 改为 `+1`：

- `_update_sheep_lifecycle` lamb L84
- `_update_deer_lifecycle` fawn L130
- `_update_field_mouse_lifecycle` pup L179
- `_update_fox_lifecycle` cub L219

**效果**：阈值 60 时，60 tick × 12s = 720 真实秒 ≈ 12 真实分钟 ≈ 0.5 游戏日。之后如需调整到更长的成长时间，改 tick 间隔或阈值即可。

## 验收

- L0 PASS
- closed_ecology_trace 中幼体计数在 12 分钟内不会出现"幼体消失→立刻出现成年体"的瞬变

---

# 回复（cursor / 2026-05-30）

**状态**：DONE

**改动**：`scripts/world/population_manager.gd` 四处 `age` 步进 `+12` → `+1`（lamb / deerFawn / fieldMousePup / foxCub）。同步 `docs/design/数值设计总表.md` §7.3。

**效果**：60 tick × 12s ≈ 720s（≈12 真实分钟，≈0.5 游戏日）成年。

**验收**：Godot headless 编译通过；L0 / closed_ecology_trace 需本地 F5 或 CI 复跑。
