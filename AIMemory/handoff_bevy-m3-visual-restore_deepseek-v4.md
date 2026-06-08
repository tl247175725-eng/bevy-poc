# M3 视觉还原：逐像素对齐 Godot 原版

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: P0（阻塞级）
**基准**: Godot 原版截图 + Godot UI 源码

---

## 目标

Bevy 版打开后，看起来和 Godot 原版一样。不是"色块能跑就行"——是布局、颜色、文字、堆叠全部对齐。

---

## 第一步：读 Godot 原版 UI（强制）

在动手写 Bevy 代码之前，先读以下 Godot 源文件：

```
E:\桌面\方寸商国：桃花源记\scripts\ui\game_ui.gd          ← 主界面布局
E:\桌面\方寸商国：桃花源记\scripts\ui\selection_info_panel.gd  ← 右侧信息面板
E:\桌面\方寸商国：桃花源记\scripts\ui\terrain_visual_palette.gd ← 地形配色
E:\桌面\方寸商国：桃花源记\scripts\cards\card_base.gd       ← 卡牌视觉样式
E:\桌面\方寸商国：桃花源记\scripts\world\world_manager.gd   ← 世界初始化和 spawn
```

**理解这些文件中定义了什么样的视觉表现，然后在 Bevy 中逐一还原。**

---

## 第二步：阻塞级还原（现在立刻做）

### 2.1 每张卡 = 彩色圆角矩形 + 中文名称

不是纯色方块。不是只有颜色的四边形。

每张卡在 Bevy 中渲染为：
- 带 `CardDef.color` 背景色的矩形（不是 Mesh2d 顶点色——用 Sprite + 自定义材质或 9-slice）
- 1px 深色边框
- 矩形中心叠加中文文字（`CardDef.display_name`，如"羊""狼""草皮"）
- 文字用 `bevy_ui::Text` 或 `Text2dBundle`，加载中文字体

**中文字体**：需要 `.ttf` 文件。用 Windows 系统自带中文字体（`C:\Windows\Fonts\msyh.ttc` 微软雅黑 或 `simhei.ttf` 黑体），或下载思源黑体。在 Bevy 中 `asset_server.load("fonts/xxx.ttf")`。

### 2.2 同格堆叠有纵向偏移

Godot 原版：3 只羊在同一格 → 3 个矩形纵向错开 5px。

Bevy 实现：同一 `(grid_x, grid_y)` 上的卡，按 entity index 排序，第 N 张 Y 偏移 `N * 4` 像素。

### 2.3 右侧 30% 面板

Godot 原版布局：

```
┌──────────────────────┬──────────┐
│   格网 70%           │ 面板 30% │
│                      │          │
│                      │ 本轮目标  │
│                      │ 操作说明  │
│                      │ 重置按钮  │
│                      │ 倍速按钮  │
│                      │ 游戏时间  │
│                      │ 天气      │
│                      │ 河岸压力  │
│                      │ 当前选中  │
│                      │ 微缩地图  │
│                      │ 日志流    │
└──────────────────────┴──────────┘
```

Bevy 实现：`bevy_ui` 的 `NodeBundle` + flex 列布局。
- 根节点：窗口宽高，横向 flex
- 左子节点：格网区域，`flex_grow: 1.0`
- 右子节点：固定宽 250px，纵向 flex，所有子项从上到下排列

### 2.4 面板内容逐项还原

**本轮目标**：静态文本，从 Godot 原版复制文案。

**操作说明**：静态文本。"左键：物理拖拽，碰撞砸击""右键：幽灵叠放/关系变化"。

**一键重置**：`bevy_ui Button`，点击调 `reset_world()`。

**时间倍速**：按钮组 1×/1.5×/2×/2.5×/3×/暂停。点击修改 `SimWorld.tick_speed`。

**游戏时间**：`bevy_ui Text`，读 `SimWorld.tick_count`，换算显示"游戏时间：HH:MM"。现实 1 秒 = 游戏 1 分钟。

**天气**：当前写死"晴天"。"距下雨：X:XX"倒计时。

**河岸压力**：`bevy_ui` 进度条。`NodeBundle` + 背景 + 前景色块百分比。

**当前选中**：和已有 `selection_info.rs` 的右上角小窗合并——选中卡的详细信息展示在这个区域。

**微缩地图**：第二 `Camera2d`，渲染到 `RenderTarget`（小纹理），面板右下角显示。或用简单方案：缩放后的静态格网色块阵列。

**日志流**：`bevy_ui` 滚动文本容器。事件发生时追加一行。最新在底。如"野兔繁殖一窝幼兔""狼叼走干草准备筑窝"。

### 2.5 顶部工具栏

倍速按钮、调试信息、版本号。一行水平排列。`bevy_ui` flex row。

### 2.6 初始卡牌数量 = 123 张

`demo_world` 或 `setup_sim_world` 必须生成和 Godot `_spawn_initial_cards` 同等数量和类型的卡。不是 30 张。是 123 张。

---

## 验收标准

**不是 `cargo test` PASS。是肉眼验收：**

1. 打开 Bevy 窗口 → 左侧格网 70%，右侧面板 30%
2. 格网上每张卡是带文字标签的圆角矩形，不是纯色方块
3. 同格多张卡有纵向堆叠偏移
4. 右侧面板显示全部内容：目标、操作说明、按钮、时间、天气、选中信息、微缩地图、日志
5. 狼在羊附近，雉鸡在灌木丛旁，水牛在湿地边缘——生态关系通过空间布局可读
6. 可点击选中卡，可拖拽

---

## 不动的

- M1/M2 全部逻辑层（systems/、world_rules.rs、spatial_index.rs）
- `card_defs.ron`
- 121 条断言（必须继续 PASS）

## 技术建议

- 中文字体：复制 `C:\Windows\Fonts\msyh.ttc` 到 `assets/fonts/`
- 卡牌渲染：每个实体 spawn 一个 SpriteBundle（背景矩形）+ 子实体 Text2dBundle（文字标签）
- 堆叠偏移：在实体 spawn 时按同格计数设置 `Transform::from_xyz(x, y + offset, z)`
- 面板：一个父 `NodeBundle` 包含所有子项，不需要每帧重建
