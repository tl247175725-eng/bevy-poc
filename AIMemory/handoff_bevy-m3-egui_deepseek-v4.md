# M3 切换：Bevy 世界 + egui 面板

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: P0（替代当前 bevy_ui 方案）
**依据**: Bevy 生态调查——Endless（殖民地模拟）、BioDynasties3（RTS）、ClaudeTest2（生物模拟）全部使用 bevy_egui。没有人用 bevy_ui 做复杂游戏面板。

---

## 架构

```
Bevy 负责                        egui 负责
┌────────────────────┐      ┌─────────────────────┐
│ bevy_ecs_tilemap   │      │ 右侧 360px 面板      │
│ 格网 tile 批次渲染   │      │ ─ 标题              │
│ 卡牌 SpriteBundle   │      │ ─ 规则说明           │
│ + Text2d 中文标签   │      │ ─ 一键重置按钮        │
│ 选中高亮（绿色环）    │      │ ─ 倍速按钮组          │
│ 拖拽（左键物理）      │      │ ─ 时间与天气          │
│ 堆叠偏移             │      │ ─ 当前选中详情        │
│ 相机缩放/平移        │      │ ─ 日志流（滚动）      │
└────────────────────┘      └─────────────────────┘
         │                        │
         └── bevy_egui 插件 ──────┘
              同一个窗口
```

**格网用 `bevy_ecs_tilemap`**，不用 Mesh2d 手搓三角形。Bevy 生态做格网游戏的标准方案——Hivemind（5 万实体）、BioDynasties3（RTS）、Endless（殖民地模拟）全用它。一张纹理当 tile，GPU 批次渲染，几千格瞬间完成。
│ 中文字体渲染         │      │ ─ 时间与天气          │
│ 堆叠偏移             │      │ ─ 当前选中详情        │
│                      │      │ ─ 日志流（滚动）      │
└────────────────────┘      └─────────────────────┘
         │                        │
         └── bevy_egui 插件 ──────┘
              同一个窗口
```

---

## 第一步：修格网——bevy_ecs_tilemap

`Cargo.toml`：

```toml
[dependencies]
bevy_ecs_tilemap = "0.6"
bevy_egui = "0.31"
```

删掉 `grid_render.rs` 中全部 Mesh2d + 顶点色代码。用 `bevy_ecs_tilemap` 重建：

```rust
// 创建 TilemapBundle
commands.spawn(TilemapBundle {
    grid_size: TilemapGridSize { x: 36, y: 24 },
    tile_size: TilemapTileSize { x: 56.0, y: 56.0 },
    storage: TileStorage::empty(36 * 24),
    texture: TilemapTexture::Single(texture_handle), // 纯色纹理
    transform: Transform::from_xyz(...),
    ..default()
});

// 每格的 tile 颜色从 terrain_colors.rs 读取
// 每格放一个 Tile，对应一个地形色纹理
```

**关键**：卡牌 sprite 不是 tile——是独立 `SpriteBundle`。叠在 tilemap 上面渲染。每个卡 = 1 个 sprite（背景矩形）+ 1 个 `Text2dBundle` 子实体（中文标签）。

## 第二步：加 bevy_egui

`Cargo.toml`：

```toml
[dependencies]
bevy_egui = "0.31"
```

`main.rs`：

```rust
use bevy_egui::{egui, EguiContexts, EguiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        // Bevy 渲染系统...
        .add_systems(Update, game_ui_panel_system)
        .run();
}
```

---

## 第二步：右侧面板用 egui::SidePanel

egui 内置 `SidePanel`——固定宽度 + 相机自动调整剩余空间。Bevy 的 `side_panel` 示例验证过。

```rust
fn game_ui_panel_system(
    mut contexts: EguiContexts,
    sim: Res<SimWorld>,
    selection: Res<SelectionState>,
) {
    egui::SidePanel::right("game_panel")
        .resizable(false)
        .exact_width(360.0)
        .show(contexts.ctx_mut(), |ui| {
            // 面板内容
        });
}
```

---

## 第三步：面板内容逐项对照视觉规范

参照 `docs/spec/visual-spec-from-godot.md` 的 §四：

### 3.1 标题
```
ui.heading("方寸商国 · MVP v2.24");
```

### 3.2 规则说明
```
ui.label("本轮目标：夜归机制、营地堆叠上限倒塌...");
ui.separator();
ui.label("左键：物理拖拽，碰撞砸击...");
ui.label("右键：幽灵叠放 / 关系变化。");
```

### 3.3 一键重置
```
if ui.button("一键重置").clicked() {
    // 触发重置
}
```

### 3.4 倍速按钮组
```
ui.horizontal(|ui| {
    for speed in [1.0, 1.5, 2.0, 2.5, 3.0] {
        if ui.selectable_label(current_speed == speed, format!("{}×", speed)).clicked() {
            // 变速
        }
    }
    if ui.selectable_label(paused, "暂停").clicked() {
        // 暂停
    }
});
```

### 3.5 时间与天气
```
ui.collapsing("时间与天气", |ui| {
    ui.label(format!("游戏时间：{}", game_time));
    ui.label("时间比例：现实 1 秒 = 游戏 1 分钟");
    ui.label(format!("天气：{}", weather));
    ui.label(format!("距下雨：{}", rain_countdown));
    ui.add(egui::ProgressBar::new(river_pressure / 100.0).text(format!("河岸压力：{}/100", river_pressure)));
});
```

### 3.6 当前选中
```
ui.collapsing("当前选中", |ui| {
    if let Some(selected) = selection.entity {
        ui.label(format!("【{}】", selected.display_name));
        ui.label(format!("身份：{}", selected.tags_zh));
        ui.label(format!("能力：{}", selected.caps_zh));
        ui.label(format!("状态：{}", selected.state));
        ui.label(format!("HP：{}", selected.hp));
        if let Some(sex) = selected.sex {
            ui.label(format!("性别：{}", sex));
        }
        // 容纳列表
        if !selected.containments.is_empty() {
            for c in selected.containments {
                ui.label(format!("容纳：【{}】", c.name));
            }
        }
    }
});
```

### 3.7 日志流
```
ui.collapsing("日志", |ui| {
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in log_lines.iter().rev().take(80) {
                ui.label(line);
            }
        });
});
```

---

## 第四步：Bevy 侧保留的

- `grid_render.rs` —— **重写为 bevy_ecs_tilemap**。删 Mesh2d 顶点色，换 tile 批次渲染
- 卡牌渲染 —— 每卡一个 `SpriteBundle`（50×50 背景 + 1px 边框）+ `Text2dBundle` 子实体（中文字号 12，颜色 #2c2117）
- `ui_interaction.rs` —— 拖拽 + 点击选中（不动，但选中信息写到 `Res<SelectionState>` 供 egui 读）
- `sim_clock.rs` —— 游戏时间 + 天气（不动，egui 面板读它的数据）
- M1/M2 全部逻辑层和 121 断言（不动）
- `card_defs.ron` 颜色表 —— 卡牌颜色从这里读取

---

## 第五步：删掉的

- `panel_ui.rs` —— bevy_ui 面板代码（全部替换为 egui）
- `selection_info.rs` —— bevy_ui 选中信息面板（egui 直接读 SelectionState）
- `game_ui_panel.rs` —— bevy_ui 右栏面板（egui 替代）
- 双相机补丁相关代码（egui 用单相机 + SidePanel 自动裁切）

---

## 验收

- `cargo test`：121 PASS
- `cargo run --release`：左侧 Bevy 格网 + 右侧 egui 面板，和视觉规范对照
- 面板可交互（按钮、倍速、日志滚动）
- 选中卡 → 面板显示详情

## 不要求

- 像素对齐 Godot 原版——这是 Bevy+egui 原生体验
- 圆角、精确配色——功能等价即可，视觉细节后续迭代
