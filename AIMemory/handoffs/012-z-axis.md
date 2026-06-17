# Handoff 012 — Z 轴实现

## 架构计划

**改什么：** `src/world_state.rs` + `src/spatial_index.rs` + `src/coords.rs`（3 文件）
**依据：** design-philosophy-v5.md §11（Z 轴铁律）、FACT.md Z 轴规则

### 1. Entity 加 z 字段（world_state.rs）

```rust
pub struct Entity {
    pub x: u8,
    pub y: u8,
    pub z: i16,  // 新增：负=地下, 0=地表, 正=空中。默认 0
    // ... 其余字段不变
}
```

### 2. spatial_index 扩 3D（spatial_index.rs）

当前 2D 桶 → 3D 桶。接口兼容改动：
- `insert(entity_id, x, y)` → `insert(entity_id, x, y, z)`
- `query_radius(x, y, r)` → `query_radius(x, y, z, r)`
- 桶结构：`HashMap<(u8, u8, i16), Vec<EntityId>>`
- 或分层 2D（每 Z 层独立 spatial_index）——更轻

**推荐分层 2D 方案：** 因为只有活跃棋盘层才有实体，不是连续的。`HashMap<i16, SpatialIndex2D>`，键是 Z 层。

### 3. 坐标工具更新（coords.rs）

新增 Z 相关辅助函数。调用方机械适配。

### 4. 调用方适配

Grep 所有使用 `entity.x` / `entity.y` 的地方，如有需要加 `z`。manhattan_distance 扩到 3D。

## 架构反馈

- 分层 2D 方案：每 Z 层独立 spatial_index
- 只有活跃层消耗内存
- 棋盘切换通过更改 "当前渲染层" 实现（后续）
- compose 公理只检查同 Z 层

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- Entity 默认 z=0
- spatial_index 查询正确区分 Z 层
