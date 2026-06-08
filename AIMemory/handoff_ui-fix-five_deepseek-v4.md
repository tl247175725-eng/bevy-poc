# UI 信息栏修复：五项

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-02

---

## 1. sex_bracket：空性别不显示【无】

`selection_info_panel.gd` L220-221：
```gdscript
if s == "":
    return "【无】"
```

改为：sex 为空时返回空字符串（不显示任何性别标记）。只有 sex 实际为 【公】【母】才显示。物品、无性别卡标题行不应出现【无】。

## 2. 选中绿框留在原始卡上

`world_manager.gd` `select_card()`：
当前流程：原始卡 → ui_resolve_selection_card → 窝卡 → set_selected(窝) → 绿框在不可见的窝上。

改为：绿框标在原始卡（草皮/灌木）上，信息栏内容照常走 resolve（显示窝面板）。两者解耦。

## 3. cell.overlay 卡地面标记

den/humus 是 cell.overlay，`configure_cell_overlay` 设了 visible=false。地图上看不到。

需要：该格有小标记（小图标/色块/文字），表示"此格有巢穴"或"此格有沃土"。点上去信息栏已正确显示（你之前的代码没问题），只是缺视觉提示。

## 4. 格子信息加海拔

`build_cell_lines()` 不显示海拔。格子的 elevation > 0 时，加一行 `海拔：N`。elevation = 0 不显示。

## 5. 灌木变狐窝后外观变化

狐狸占灌木筑窝后，灌木还是普通灌木外观。该灌木应该有视觉区分——加个窝的标记（同问题 3 的 overlay 标记逻辑）。

---

## 涉及文件

| 文件 | 改什么 |
|------|--------|
| `scripts/ui/selection_info_panel.gd` | sex_bracket 空性别不显示；build_cell_lines 加海拔行 |
| `scripts/world/world_manager.gd` | select_card 绿框与信息栏解耦 |
| `scripts/core/world_rules.gd` | overlay 地面标记机制 |
| `scripts/core/world_rules/world_rules_ui.gd` | cell 海拔读取 |
| 相关视觉层 | overlay 标记渲染 |

## 约束

- L0 不降
- 不碰管线/签名/封闭模式
- 记 fix-log
