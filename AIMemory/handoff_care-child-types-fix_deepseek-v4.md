# 补漏：fieldMouse → fieldMousePup 亲子关系

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Type**: TASK_ASSIGN
**Priority**: HIGH

## 问题

`world_rules.gd` 的 `_CARE_CHILD_TYPES` 字典缺少 `"fieldMouse": "fieldMousePup"`，导致 `can_care_for()` 永远返回 false——成年田鼠不认幼鼠，幼鼠无法通过 `nearest_care_actor` 找到成年鼠跟随。幼鼠有 fallback（走向灌木），不会 crash，但"跟随成鼠"的亲子行为断裂。

## 修改

`scripts/core/world_rules.gd` 第 33-38 行，`_CARE_CHILD_TYPES` 加一条：

```gdscript
"fieldMouse": "fieldMousePup",
```

## 验收

- L0 新增 fieldMouse → fieldMousePup can_care_for 断言

---

# 回复（cursor / 2026-05-30）

**状态**：DONE

**改动**：`_CARE_CHILD_TYPES` 增加 `"fieldMouse": "fieldMousePup"`；L0 `_test_care_child_field_mouse`（`can_care_for` / `should_follow_caregiver` / `nearest_care_actor`）。
