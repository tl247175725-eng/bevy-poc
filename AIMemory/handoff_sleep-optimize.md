# 实体休眠机制

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-11
**Priority**: P2（最后一道性能优化——减少空闲实体无效 tick）

## 架构计划
Entity 加 `sleep_until_tick`。空闲实体（无活跃 drive、无威胁、不饿）跳过 tick_reactive，睡眠 N tick。被外部事件唤醒时恢复。

## 架构反馈
不改变任何行为逻辑。纯性能优化。

## 智能验收
- idle 实体跳过 tick，活跃实体正常 tick
- smoke test 行为不变（predation > 0, herbivore 移动 > 0）
- 每 tick 处理实体数减少（可 BRP 验证）

---

## 实现

### 1. Entity 新增字段

```rust
pub sleep_until_tick: u64,  // 0 = 不睡眠，>0 = 在此 tick 之前跳过处理
```

### 2. tick_reactive 入口跳过

```rust
pub fn tick_reactive(world, id, delta) {
    let e = &world.entities[&id];
    if e.sleep_until_tick > world.tick_count {
        return; // 还在睡
    }
    // ... 正常逻辑
}
```

### 3. 休眠条件

```
所有 need 值 < 阈值的 30%（不饿、不渴、不恐惧）
当前 ecology_state == Idle || Wandering
无活跃 drive（所有 drive 打分 < 最低阈值）
→ sleep_until_tick = tick_count + 5（睡 5 tick）
```

### 4. 唤醒条件

任何以下事件唤醒：
```
on_move 通知有 predator 进入感知范围
被砸/被叠
cover 被破坏（如果该实体在藏中）
need 值因 decay 达到阈值（由公告栏后台检查）
```

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/world_state.rs` | Entity 加 sleep_until_tick |
| `src/systems/tick_reactive.rs` | 入口跳过 + 休眠/唤醒逻辑 |
| `src/event_registry.rs` / on_move | predator 通知时唤醒周边猎物 |
