# Handoff 010 — Perceive 接入 + Apply 引擎

## 架构计划

**改什么：** `src/systems/main_tick.rs`（1 文件）
**做什么：** 填 Perceive 和 Apply 两个 TODO

### Perceive 阶段

每实体扫描邻居 → 四通道感知 → 直推需求急迫度

```rust
// Phase 1: Perceive
for entity in world.entities.values() {
    let neighbors = spatial_index.query_radius(entity.x, entity.y, entity.z, MAX_SENSE_RANGE);
    for neighbor_id in neighbors {
        let neighbor = &world.entities[&neighbor_id];
        let distance = manhattan_distance(entity.pos(), neighbor.pos());
        
        // 视觉
        if let Some(result) = perceive_vision(entity, neighbor, distance, world.light_level, /* occlusion */) {
            apply_perception_to_needs(&mut entity.needs, &result);
        }
        // 听觉
        // 嗅觉
    }
}

// Phase 2-5: 已有
// Phase 6: Apply
for action in world.pending_actions.drain(..) {
    apply_meta_action(world, action);
}
```

### Apply 阶段

收集 tick 中产生的所有元动作，统一在末尾应用：

```rust
/// 待执行的元动作队列
world.pending_actions: Vec<(EntityId, MetaAction)>

/// 统一应用
fn apply_meta_action(world: &mut WorldState, entity_id: EntityId, action: MetaAction) {
    match action {
        MetaAction::Move { dx, dy } => { /* 更新位置 + spatial_index */ }
        MetaAction::Strike { target } => { /* 计算伤害 + 更新标签 */ }
        MetaAction::Pause { .. } => { /* 无副作用 */ }
        _ => {} // 其他元动作暂时 stub
    }
}
```

## 架构反馈

- Perceive 每实体只扫描邻居（spatial_index 已过滤）✅
- Apply 统一生效保证确定性 ✅
- 元动作执行结果在同一帧内对后续实体不可见 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- Perceive/Apply 阶段从 TODO 变为实际调用
