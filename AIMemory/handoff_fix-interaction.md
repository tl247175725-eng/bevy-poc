# 修复鼠标交互：选中偏移 + 拖拽不跟随

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（游戏不可玩——点击和拖拽全错乱）

## 架构计划
交互层（ui_interaction.rs / coords.rs）与视觉层（world_view.rs）的坐标转换链断裂。需要追踪 screen → world → grid 的完整转换，不引入新公理。WorldView 的 camera zoom/pan 和 viewport layout 可能未正确传导到点击坐标。

## 架构反馈
坐标系统的耦合点太多——coords.rs 的多个函数（card_world_pos、grid_to_world、cursor_to_world）各自独立计算，没有统一的坐标转换入口。后续需要重构为单一 CoordinateSystem。

---

## 问题描述

### 选中偏移
点击格子下半部分才选中目标格。点击卡牌下方才能点到卡。

### 拖拽不跟随
按住卡牌拖拽时，卡牌不随鼠标移动，松开后才在目标位置出现。

## 诊断任务

### 1. 追踪坐标转换链
`cursor_to_world` → `grid_from_cursor` → `handle_selection_click` 的完整数据流。确认每一步的 Y 轴方向、偏移量、缩放因子是否正确。已知 WorldView 有 zoom 和 pan 状态，需要在坐标转换中补偿。

### 2. 拖拽预览
Godot 项目中拖拽有一个 ghost（半透明预览）跟随鼠标。检查 `DragState` 和 `try_place_entity` 是否有 ghost 渲染逻辑，以及 `GhostPlaceMode` 是否被正确激活。

### 3. 独立测试
创建一个测试用例：在已知格坐标放置卡牌 → 用模拟的鼠标坐标尝试点击 → 断言选中的是正确实体。

## 修改范围

| 文件 | 改动 |
|---|---|
| `src/coords.rs` | `cursor_to_world` / `grid_from_cursor` 修正 |
| `src/world_view.rs` | 确认 `area_to_world` 和 zoom/pan 传导 |
| `src/ui_interaction.rs` | `handle_selection_click` + `try_place_entity` + drag ghost |

## 验收
- 点击卡牌所在格 → 精确选中
- 拖拽卡牌 → 卡牌跟随鼠标
- `cargo test --release` 不因改动而减少 PASS 数量
