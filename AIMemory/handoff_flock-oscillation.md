# 群聚震荡修复

**Priority**: P0（群分分合合抽搐）

## 根因

compose 允许同种群居共享格子，但 `execute_flock` 的分离力（`dist == 0`）立刻推开。动物聚→推到相邻格→聚合拉回→再推到同格→再推开。无限震荡。

## 修复

`src/systems/tick_reactive.rs` `execute_flock` 函数中，分离力只在**超容量**时触发：

```rust
// 当前（错误）
if dist == 0 {
    sep_dx += ...; sep_dy += ...;
}

// 修复
if dist == 0 && cell_current_count >= profile.flock_max {
    sep_dx += ...; sep_dy += ...;
}
```

低于 flock_max 时同格共存不排斥。超过上限才推开多余的。

需要获取当前格容量：`world.cell_composition.slot(x, y).living_count`。

## 验收

启动游戏 → 同种成群聚在一起不抽搐，超过 flock_max 才自然分裂。
