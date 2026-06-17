# Handoff 023 — 卡牌棋子底座

## 架构计划

**改什么：** 新建 `src/render/card_base.rs`，修改 `src/render/mod.rs`（2 文件）
**做什么：** 统一黑圆柱底座 + 数字标签。所有卡牌棋子共用。

### 底座 mesh

```rust
/// 底座：黑色扁平圆柱。半径 0.3，高 0.05。
/// 放在每个悬浮卡的位置下方。
pub fn generate_base_mesh() -> Mesh {
    // Cylinder { radius: 0.3, height: 0.05, segments: 8 }
    // 纯黑色顶点色
}
```

### 数字标签

```rust
use bevy::prelude::*;

/// 底座上的数量标签（白字）
#[derive(Component)]
pub struct BaseQuantityLabel;

pub fn spawn_base_with_label(
    commands: &mut Commands,
    pos: Vec3,
    quantity: u32,
    base_mesh: Handle<Mesh>,
    base_material: Handle<StandardMaterial>,
    font: Handle<Font>,
) {
    // 底座圆柱
    commands.spawn((
        Mesh3d(base_mesh),
        MeshMaterial3d(base_material),
        Transform::from_translation(pos),
    ));
    // 数字标签（Text2d 或 billboard）
    // 只在 quantity > 1 时显示
}
```

### 渲染着色

```
底座: 纯黑 #000000（材质颜色 = Color::BLACK）
数字: 白色，font_size = 24，贴在底座上方
```

## 架构反馈

- 所有卡共用同一个底座 mesh——GPU 自动实例化 ✅
- 数字只在 quantity > 1 时显示 ✅
- 独立模块——不依赖任何其他渲染系统 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
