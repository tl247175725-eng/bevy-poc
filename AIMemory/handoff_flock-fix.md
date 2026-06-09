# 群聚 Standoff 修复

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-09
**Priority**: P0（群系统无法聚集——分离力范围太大导致动物互推，始终聚不到一起）

---

## 根因

`src/systems/tick_reactive.rs` 第 563 行：

```rust
if dist <= 2 && dist > 0 {
```

分离力在距离 ≤2 时就触发 → 相邻格被推开 → 动物永远到达不了同一格 → 群卡从不出现。

## 修复

### 1. 分离力只在同格触发

```rust
// 第 563 行改为
if dist == 0 {
    sep_dx += (x as i16 - nx as i16);
    sep_dy += (y as i16 - ny as i16);
}
```

距离=0 表示同格有太多同类 → 推开一个。相邻格不推。

### 2. 聚合力增强——相邻格拉近

聚合力（cohesion）本身就能把动物拉到一起。修复后不需要分离力阻挠，聚合自然工作。

### 3. 群卡阈值先降到 2 测试

`src/card_visual.rs` 第 241 行 `members.len() < 3` → 临时改为 `< 2`。2 只同格也显示群卡。验证群卡渲染正常工作后再改回 3。

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/systems/tick_reactive.rs` | 第 563 行 `dist <= 2` → `dist == 0` |
| `src/card_visual.rs` | 第 241 行 `3` → `2`（临时测试） |

## 验收

1. 启动游戏 → 两只羊走到同格 → 看到群卡出现
2. 确认后把阈值改回 3
