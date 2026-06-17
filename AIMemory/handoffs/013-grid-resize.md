# Handoff 013 — 棋盘 32×32

## 架构计划

**改什么：** `src/world_rules.rs` 的 GRID 常量（1 文件）
**做什么：** `GRID_WIDTH: 36 → 32`，`GRID_HEIGHT: 24 → 32`

### 改动

```rust
pub const GRID_WIDTH: u8 = 32;
pub const GRID_HEIGHT: u8 = 32;
```

### 波及

Grep `GRID_WIDTH` 和 `GRID_HEIGHT` 在 `src/` 中的所有引用。如果任何测试或硬编码假设了 36×24 尺寸，更新为 32×32。

### 不做的

- 不修改 spatial_index（已支持 3D 分层，无硬编码格数）
- 不修改卡定义（不是这步的事）

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 冒烟测试受影响则跳过
