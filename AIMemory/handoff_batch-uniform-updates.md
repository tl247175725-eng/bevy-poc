# 批量统一更新优化

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-11
**Priority**: P2（最终性能——统一变化不逐实体判断）

## 架构计划
将腐败推进、年龄增长、饥饿衰减、冷却递减等"所有实体都做的同一件事"从 tick 分散逻辑中提出，合并为单次扫描的批量更新。决策层（seek/flee）保持不变。

## 架构反馈
不影响任何行为逻辑。纯执行层优化。

## 智能验收
- 行为不变（smoke test 同结果）
- 统一更新在单个循环中完成

---

## 实现

### 合并到 main_tick 开头的一个循环

当前 main_tick 开头已经有一个循环处理 cooldown：

```rust
for entity in world.entities.values_mut() {
    entity.consumed = false;
    entity.hunt_cooldown = (entity.hunt_cooldown - delta).max(0.0);
    entity.harvest_cooldown = (entity.harvest_cooldown - delta).max(0.0);
}
```

扩展这个循环，加入所有统一递减/推进字段：

```rust
for entity in world.entities.values_mut() {
    // 每 tick 重置
    entity.consumed = false;
    
    // 冷却递减
    entity.hunt_cooldown = (entity.hunt_cooldown - delta).max(0.0);
    entity.harvest_cooldown = (entity.harvest_cooldown - delta).max(0.0);
    
    // 腐败推进（仅可腐败实体）
    if entity.is_corpse || entity.type_name.ends_with("Corpse") {
        entity.decay_timer += delta;
    }
    
    // 年龄（仅生物）
    entity.age += delta;
}
```

### tick_starvation 中的批量饥饿检查

已经是单次扫描所有 being 实体——保持不变。

### tick_environment 中的尸体腐败

当前 `tick_corpses` 单独遍历尸体。可以合并到上面统一循环中（减少一次遍历）。

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/systems/main_tick.rs` | 统一更新循环扩展 |
| `src/systems/tick_environment.rs` | 尸体腐败逻辑移入统一循环 |
| `src/systems/tick_starvation.rs` | 不变（已有批量扫描） |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS
- `cargo run --release -- --smoke-test` 行为一致
