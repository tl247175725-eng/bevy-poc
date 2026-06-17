# Handoff 008 — 工程管线接入

## 架构计划

**改什么：** `src/world_state.rs` + `src/systems/main_tick.rs`（2 文件）
**做什么：** Entity 加 MemoryStore、tick 顺序重排为六阶段、需求引擎管线接入

### world_state.rs — Entity 加字段

```rust
pub struct Entity {
    // ... 现有字段 ...
    pub memory: MemoryStore,       // 流动记忆
    pub needs: Vec<NeedState>,     // 需求状态（代替旧 NeedState）
    pub knowledge: KnowledgeGraph,  // 个人知识图
    pub execution: ExecutionState,  // 当前执行计划
}
```

### main_tick.rs — tick 顺序重排

```rust
pub fn main_tick(world: &mut WorldState, delta: f32) {
    world.tick_delta = delta;
    world.tick_count += 1;
    world.elapsed += delta;

    // === Phase 1: Perceive（所有实体统一感知，冻结快照） ===
    // TODO: 感知系统 → 填充每个实体的 perceived_entities 列表
    // 当前占位：标记为待实现，不阻塞编译
    
    // === Phase 2: Need tick（需求衰减） ===
    for entity in world.entities.values_mut() {
        for need in &mut entity.needs {
            tick_need(need, delta);
        }
    }
    
    // === Phase 3: Safety block（安全阻断非安全需求） ===
    for entity in world.entities.values_mut() {
        apply_safety_block(&mut entity.needs);
    }
    
    // === Phase 4: Decide（匹配引擎） ===
    for entity in world.entities.values_mut() {
        if entity.execution.intention.is_none() || is_plan_failed(&entity.execution) {
            let env: Vec<(u32, MaterialProperties)> = Vec::new(); // TODO: 感知数据就位后填充
            if let Some(action) = tick_need_engine(
                &mut entity.needs,
                &mut entity.execution,
                &entity.knowledge,
                &env,
                delta,
            ) {
                // TODO: 将 action 加入待执行队列
            }
        }
    }
    
    // === Phase 5: Execute（执行元动作） ===
    for entity in world.entities.values_mut() {
        let env: Vec<(u32, MaterialProperties)> = Vec::new(); // TODO
        if let Some(_action) = tick_execution(&mut entity.execution, &env) {
            // TODO: 将 action 加入 Apply 队列
        }
    }
    
    // === Phase 6: Apply（统一生效） ===
    // TODO: 执行队列中的元动作，更新世界状态
    
    // 保留现有基础设施
    crate::bulletin::maybe_update(world);
    batch_uniform_entity_updates(world, delta);
    flush_corpse_decay(world);
    // ... 等现有逻辑保留但标记为待迁移
}
```

### 不做的

- 不实现感知系统（Perceive 阶段标记 TODO）
- 不实现 Apply 机制（标记 TODO）
- 保留所有现有批量更新逻辑不删（标记注释：待迁移到新六阶段）

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 六阶段管线编译通过
- 现有功能不被破坏
