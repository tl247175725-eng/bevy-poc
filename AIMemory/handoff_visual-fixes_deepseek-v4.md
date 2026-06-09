# 视觉/交互修复

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（游戏不可玩）

---

## P0-1：卡牌与格子对齐

卡牌统一向上偏移，没有居中对齐各自所在的格子。

**查**：`src/coords.rs` 的 `card_world_pos` 和 `src/card_visual.rs` 的 `sync_card_visuals` 中位置计算逻辑。确保每张卡的中心与格子的中心重合。

---

## P0-2：边界地格 → 荒地 + 外围山卡

当前地图最外圈仍是"边界"（edge）类型地格。

**需要**：
1. 地图最外面一圈（x=0, x=35, y=0, y=23）——spawn "荒芜"（wasteland/barren land，不是 edge）
2. 在该"荒地"上 spawn 一圈"山"卡。山卡 = mountain card（card_defs.ron 中有定义，类型名为 "mountain"）

**实现**：
```rust
// 在 spawn_initial_world 开头
for x in 0..GRID_WIDTH {
    w.spawn("荒芜", x, 0);
    w.spawn("荒芜", x, GRID_HEIGHT - 1);
    w.spawn("mountain", x, 0);
    w.spawn("mountain", x, GRID_HEIGHT - 1);
}
for y in 1..GRID_HEIGHT - 1 {
    w.spawn("荒芜", 0, y);
    w.spawn("荒芜", GRID_WIDTH - 1, y);
    w.spawn("mountain", 0, y);
    w.spawn("mountain", GRID_WIDTH - 1, y);
}
```

如果 card_defs.ron 中没有"荒芜"这张卡，创建一个：
- 类型名: `荒芜`
- 标签: `terrain`, `barren`
- 颜色: 灰色/暗棕色
- HP: 1

---

## P0-3：鼠标选中偏移

点击格子下方半格才能选到目标卡。松开鼠标后卡才出现在目标位置（不跟随拖动）。

**查**：`src/ui_interaction.rs` 的 `handle_selection_click` 和拖拽逻辑。屏幕坐标→世界坐标→格子坐标的转换链断裂了。

可能原因：摄像头 zoom/pan 变化后，`cursor_to_world` 或 `grid_from_cursor` 没有正确补偿 viewport 偏移。

**预期行为**：
- 点击卡牌所在格 → 选中该卡
- 拖拽卡牌 → 卡牌跟随鼠标实时移动（ghost 预览）
- 松开 → 卡牌落在目标格

---

## P0-4：群卡信息面板

选中群卡时，右侧信息栏显示的是单只动物的信息（HP=1、标签等），而不是群卡专有信息。

**需要**：选中群卡（`herd_count > 0`）时，面板显示：
- 卡名 + "群" 标识（如"羊群"）
- 数量：×N 只
- 群体标签（从基础动物继承，去掉不适用于群的个体标签）

**实现**：`src/selection_info.rs` 的 `build_panel` / `build_card_panel` 中，检测 `entity.herd_count > 0`，走群卡面板分支。

---

## 验收

1. `cargo check` 通过
2. 启动游戏：
   - 卡牌居中在格子上
   - 地图边界是荒地+山卡
   - 鼠标点击准确选中目标
   - 拖拽卡牌跟随鼠标
   - 点击群卡显示群面板
