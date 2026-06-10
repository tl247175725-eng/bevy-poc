# 闪退修复

**Priority**: P0

## 崩溃 1：B0001 Query 冲突
sprint_visual::sync_sprint_fx 的 Query 访问 Transform，与其他系统冲突。改成 `Without<CardVisual>` 或把残影逻辑移入 sync_card_visuals。

## 崩溃 2：tick_reactive.rs:187 HashMap 查不到 key
检查第 187 行附近的 `world.entities[&id]` 或类似索引访问，改为 `world.entities.get(&id)` 安全读取。

## 验收
游戏不闪退
