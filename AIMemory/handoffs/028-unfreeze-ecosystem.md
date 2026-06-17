# Handoff 028 — 解冻生态系统（一次修 8 断点）

> Opus 发现 10 个断点使生态系统完全冻结。本次修通其中 8 个——一次让所有 16 种动物解冻。

## 架构计划

**改什么：** 4 个文件（`main_tick.rs` / `perception/senses.rs` / `execution.rs` / `need_match/search.rs`）

### 断点 1+3：env 为空 + 感知占位符 → 动物看不见世界

**文件：** `src/systems/main_tick.rs` Phase 4 前 + `src/perception/senses.rs`

```rust
// main_tick.rs Phase 1.5: 为每个动物组装感知环境
// 替代: let env: Vec<...> = Vec::new(); // TODO
fn build_env_for_entity(entity: &Entity, world: &WorldState) -> Vec<(u32, MaterialProperties)> {
    let neighbors = world.spatial_index.query_near(entity.x, entity.y, entity.z, MAX_SENSE_RANGE);
    neighbors.iter()
        .filter_map(|&nid| {
            let neighbor = world.entities.get(&nid)?;
            let props = card_to_material_properties(neighbor, world.card_defs);
            Some((nid.0 as u32, props))
        })
        .collect()
}
```

```rust
// perception/senses.rs: has(0) → 真实标签查询
// 旧: if target_tags.has(0) { ... }  // 占位符
// 新: 从 TagRegistry 获取 predator/herbivore/prey 的 bit
fn is_predator(tags: &TagBits, registry: &TagRegistry) -> bool {
    registry.get_bit("diet:carnivore").map(|b| tags.has(b)).unwrap_or(false)
}
fn is_herbivore_prey(tags: &TagBits, registry: &TagRegistry) -> bool {
    tags.has_any(&["animal"]) && registry.get_bit("diet:herbivore").map(|b| tags.has(b)).unwrap_or(false)
}
```

### 断点 4：Act 步骤 target 硬编码 EntityId(0)

**文件：** `src/need_match/execution.rs`

```rust
// Act 步骤的 target 从 EnvironmentPerceive 来源绑定真实 EntityId
// 旧: MetaAction::Strike { target: EntityId(0) }
// 新: MetaAction::Strike { target: EntityId(state.acquired_objects[0].entity_id as u64) }
// 旧: MetaAction::Consume { target: EntityId(0) }
// 新: MetaAction::Consume { target: EntityId(acquired_target_id as u64) }
```

### 断点 6：Move{0,0} → 朝目标走一步

**文件：** `src/need_match/execution.rs` + 新增辅助

```rust
/// 从当前位置向目标实体走一步（曼哈顿单步）
fn move_toward(from: (u8, u8, i16), target: (u8, u8, i16)) -> MetaAction {
    let dx = (target.0 as i16 - from.0 as i16).signum();
    let dy = (target.1 as i16 - from.1 as i16).signum();
    // 曼哈顿：优先长轴
    if dx.abs() >= dy.abs() && dx != 0 {
        MetaAction::Move { dx, dy: 0 }
    } else if dy != 0 {
        MetaAction::Move { dx: 0, dy }
    } else {
        MetaAction::Pause { ticks: 1 }  // 已在目标位置
    }
}
```

### 断点 2：decomposition 为空 → 知识条目带上真实步骤

**文件：** `src/initial_spawn.rs` 中 `init_animal_knowledge()`

```rust
// 旧: decomposition: vec![]
// 新: 每种知识一个通用模板
KnowledgeEntry {
    name: "吃草",
    functional_prerequisites: vec![
        PropertyRequirement { property: "is_plant".into(), operator: Present, threshold: 0.0, quantity_needed: 1.0 }
    ],
    decomposition: vec![
        DecompositionStep::Acquire { requirements: vec![...] },
        DecompositionStep::Act { action: "Consume".into(), target: Some("acquired_0".into()) },
    ],
    effects: vec![EffectDescriptor { satisfies: NeedKind::Nutrition, magnitude: 0.5 }],
    source: KnowledgeSource::CommonSense,
}
```

### 断点 4 桥接词表：edible/prey → is_plant/is_animal

**文件：** `src/need_match/search.rs` 的 `MaterialProperties.satisfies()`

```rust
// 旧: 查 "edible" / "prey" 字符串
// 新: 从实体的 tag_bits 判断
//   "is_plant" → entity.tags.has_any_plant()（nutrition:autotroph 或 body_plan:plant）
//   "is_animal" → entity.tags.has("animal")
```

### 断点 7+9：饥饿死亡 + 繁殖新词表

**文件：** 各 tick 系统（后续 handoff 单独处理——这俩涉及旧系统词表重写，不在本次范围）

---

## 改了什么（一句话）

感知不再返回虚假数据 → env 不再为空 → 知识条目有真实步骤 → Act 步骤绑定真实目标 → 动物朝目标移动 → Strike/Consume 生效 → 鹿吃掉草。

## 不改的

- 饥饿死亡（tick_starvation）——后续
- 繁殖新词表（tick_reproduction）——后续
- smoke_test——后续
- C2（叠加域）——后续

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 新增集成测试（如果可行）：生成世界 → tick 100 次 → 断言至少 1 只鹿移动了位置
