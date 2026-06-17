# Handoff 018 — 地形块渲染

## 架构计划

**改什么：** 新建 `src/render/terrain.rs`，修改 `src/render/mod.rs`（2 文件）
**做什么：** 32×32 格 → 合并为 ~50-80 个平面 mesh chunk，顶点色区分地形

### 核心逻辑

**1. 格→chunk 分组合并：**

```
扫描 32×32 格:
  同行同 terrain 类型 → 合并为一个矩形 chunk
  → 一个 chunk = 连续几格 × 几格的平面
  → 每个 chunk 生成一个 Mesh3d entity
```

**2. 每 chunk mesh 生成：**

```rust
struct TerrainChunk {
    x_start: u8, x_end: u8,  // 列范围
    y_start: u8, y_end: u8,  // 行范围
    terrain_type: String,    // "grassland" / "broadleaf_forest" / ...
}

// 生成平面 mesh：
// - 每个格 = 一个 quad（2 个三角）
// - 顶点色 = 地形颜色（从 card_def 的 color 字段读）
// - 高度 Z 由环决定（深潭最低=-0.5, 山壁最高=+0.5）
fn generate_chunk_mesh(chunk: &TerrainChunk, card_defs: &[CardDef]) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    // 遍历 chunk 内每格，生成 quad + 顶点色
    // 每格宽 1.0 单位（棋盘上 1 格 = 1.0 单位）
}
```

**3. 动态更新：**

当卡片堆变化（地形变了、树被砍了），需要更新对应 chunk：
- 简单：提供 `update_chunk_mesh(chunk_entity, world_state)` 函数
- 当前 handoff 只做初次生成，更新逻辑后续

### 关键参数

```rust
const TILE_SIZE: f32 = 1.0;       // 一格在渲染空间 = 1.0 单位
const CENTER_X: f32 = 16.0;       // 地图中心 x
const CENTER_Y: f32 = 16.0;       // 地图中心 y
const RING_HEIGHTS: [(f32, f32); 7] = [  // (距离, 高度偏移)
    (1.5, -0.5),   // 深潭最低
    (3.5, -0.3),
    (7.0, -0.1),
    (12.0, 0.0),   // 草原基准
    (19.0, 0.1),
    (25.0, 0.3),
    (99.0, 0.5),   // 山壁最高
];
```

### 系统注册

在 `RenderPlugin` 的 Startup 阶段生成初始地形 chunk。

## 架构反馈

- 纯顶点色——零贴图 ✅
- chunk 合并减少 draw calls（~50-80 vs 1024）✅
- 高度偏移体现碗形地形 ✅
- card_def 的 color 字段直接驱动顶点色 ✅
- Bevy 0.15 Mesh API 完全兼容（已验证）✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- chunk 数 > 0 且 ≤ 80
