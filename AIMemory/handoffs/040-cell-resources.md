# Handoff 040 — 格子资源模型：植物从独立实体变为数量卡

## 三柱强制检查

| 柱子 | 用哪个 |
|---|---|
| 标签 | 不涉及新标签——植物的 CardDef 和标签不变 |
| 元数值 | 不涉及新数值 |
| 元动作 | `MetaAction::Consume` 的植物消耗路径改为减少格子资源数量 |
| 公理 | `can_digest()` 不变——仍从 TagBits 判断 |

## 架构计划

**改什么：** `src/world_state.rs` + `src/initial_spawn.rs` + `src/systems/main_tick.rs`（3 文件）

**为什么：** 植物应该是格子上的数量卡（芒草 ×40），不是 1500 个独立实体。当前模型违反"一格一叠卡，卡有量"的设计哲学，且性能差。

### 设计决策（策划已确认）

- **植物 = 格子上的数量**：`cell_resources: HashMap<(u8,u8), HashMap<String, u32>>`
- **动物 = 独立实体**：不变
- **Consume 植物**：在动物位置查格子资源 → 找到可消化的 → 数量 -1
- **Consume 动物**：保持不变（EntityId 目标）
- **植物 Entity 暂时保留**：渲染仍需要，后续 handoff 迁移渲染到读 cell_resources
- **植物 Entity 不再参与 Consume**：消耗逻辑走 cell_resources，不走 entity.consumed

### 改动 1：`src/world_state.rs`

新增字段和方法：

```rust
pub struct WorldState {
    // ... 现有字段
    /// 格子资源——每格的植物/材料数量。植物不再是独立 Entity。
    pub cell_resources: HashMap<(u8, u8), HashMap<String, u32>>,
}

impl WorldState {
    /// 向格子添加资源
    pub fn add_resource(&mut self, x: u8, y: u8, resource_type: &str, qty: u32) {
        *self.cell_resources
            .entry((x, y))
            .or_default()
            .entry(resource_type.to_string())
            .or_insert(0) += qty;
    }

    /// 从格子消耗 1 单位资源。返回是否成功。
    pub fn consume_resource(&mut self, x: u8, y: u8, resource_type: &str) -> bool {
        if let Some(cell) = self.cell_resources.get_mut(&(x, y)) {
            if let Some(qty) = cell.get_mut(resource_type) {
                if *qty > 0 {
                    *qty -= 1;
                    return true;
                }
            }
        }
        false
    }

    /// 查询格子上某种资源的数量
    pub fn resource_qty(&self, x: u8, y: u8, resource_type: &str) -> u32 {
        self.cell_resources
            .get(&(x, y))
            .and_then(|cell| cell.get(resource_type))
            .copied()
            .unwrap_or(0)
    }

    /// 获取格子上所有资源（类型+数量）
    pub fn cell_resource_list(&self, x: u8, y: u8) -> Vec<(String, u32)> {
        self.cell_resources
            .get(&(x, y))
            .map(|cell| cell.iter().map(|(k, &v)| (k.clone(), v)).filter(|(_, v)| *v > 0).collect())
            .unwrap_or_default()
    }
}
```

`WorldState::new()` 中初始化 `cell_resources: HashMap::new()`。

### 改动 2：`src/initial_spawn.rs`

`spawn_concentric_world` 中的植物生成改为 `add_resource`：

```rust
// 旧：world.spawn("miscanthus", x, y);
// 新：world.add_resource(x, y, "miscanthus", 1);
// 但按设计，一格应有多丛——所以数量不是 1

// 草原格子：芒草密度
if r < 50 { world.add_resource(x, y, "miscanthus", 30); }

// 浅水格子：莲花 + 水草
if r < 17 { world.add_resource(x, y, "lotus", 5); }
if r < 50 { world.add_resource(x, y, "waterweed", 20); }

// 湿地格子：芦苇 + 菖蒲
if r < 50 { world.add_resource(x, y, "reed", 25); }
if r < 33 { world.add_resource(x, y, "cattail", 15); }

// 森林格子：楠木 + 樟树 + 竹子（数量少）
if r < 33 { world.add_resource(x, y, "nanmu_tree", 3); }
if r < 25 { world.add_resource(x, y, "camphor_tree", 2); }
if r < 20 { world.add_resource(x, y, "bamboo", 5); }

// 山麓格子：松 + 杜鹃
if r < 50 { world.add_resource(x, y, "pine_forest", 4); }
if r < 20 { world.add_resource(x, y, "azalea", 8); }
```

**同时保留 `world.spawn()` 调用**——暂时还需要植物 Entity 做渲染。后续 handoff 移除。

### 改动 3：`src/systems/main_tick.rs`

**Consume 分支增加植物消耗路径**：

在 `apply_meta_action` 的 `MetaAction::Consume { target }` 分支中：

```rust
MetaAction::Consume { target } => {
    // 先尝试消耗格子资源（食草）
    let actor_pos = world.entities.get(&entity_id).map(|e| (e.x, e.y));
    let actor_type = world.entities.get(&entity_id).map(|e| e.type_name.clone());

    if let (Some((ax, ay)), Some(ref atype)) = (actor_pos, actor_type) {
        let actor_def = world.card_defs.get(atype);
        if let Some(adef) = actor_def {
            let actor_tags = &adef.tag_bits;

            // 检查是否能消化植物
            let can_eat_plant = actor_tags.has(tag::DIET_HERBIVORE.bit)
                || actor_tags.has(tag::DIET_OMNIVORE.bit)
                || actor_tags.has(tag::DIET_FRUGIVORE.bit);

            if can_eat_plant {
                // 查格子上有什么植物资源
                let resources = world.cell_resource_list(ax, ay);
                let mut ate = false;
                for (res_type, qty) in &resources {
                    if *qty > 0 {
                        // 检查这个资源是否可消化（有 plant 标签）
                        if let Some(res_def) = world.card_defs.get(res_type) {
                            if crate::axioms::consume::can_digest(actor_tags, &res_def.tag_bits, false,
                                crate::world_rules::TAG_REGISTRY.get().unwrap_or(&crate::tags::TagRegistry::default_registry()))
                            {
                                if world.consume_resource(ax, ay, res_type) {
                                    // 计算营养
                                    let mass = crate::axioms::consume::estimate_mass_from_tags(&res_def.tag_bits);
                                    let metab = crate::meta_values::metab_rate_from_tags(actor_tags);
                                    let energy = crate::meta_values::baseline_energy(mass, metab);
                                    // 更新需求
                                    if let Some(actor) = world.entities.get_mut(&entity_id) {
                                        for need in &mut actor.needs {
                                            if matches!(need.kind, NeedKind::Nutrition) {
                                                need.current = (need.current - energy * crate::meta_values::DIGESTION_EFFICIENCY).max(0.0);
                                            }
                                        }
                                        actor.fed = true;
                                    }
                                    ate = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if ate { return; }
            }
        }
    }

    // 如果不是吃植物（或植物没吃到），走原来的 Entity 消耗路径（吃动物）
    // ... 保持原有的 can_digest + baseline_energy + entity.consumed 逻辑
}
```

**build_env_for_entity 也需要更新**：在环境物体列表中包含格子资源。

```rust
// 在 build_env_for_entity 末尾追加：
// 把邻近格子的植物资源也加入 env（作为虚拟 entity，让知识搜索能找到）
for nx in ax.saturating_sub(MAX_SENSE_RANGE)..=ax.saturating_add(MAX_SENSE_RANGE) {
    for ny in ay.saturating_sub(MAX_SENSE_RANGE)..=ay.saturating_add(MAX_SENSE_RANGE) {
        for (res_type, qty) in world.cell_resource_list(nx, ny) {
            if qty > 0 {
                if let Some(def) = world.card_defs.get(&res_type) {
                    let props = card_to_material_properties_from_def(def);
                    // 用虚拟 ID 区分资源（高位标记）
                    let virtual_id = (nx as u32) << 16 | (ny as u32) << 8 | resource_index;
                    env.push((virtual_id, (nx, ny), props));
                }
            }
        }
    }
}
```

注意：这里有性能隐患（遍历感知范围内所有格子的所有资源）。但在 32×32 棋盘上、MAX_SENSE_RANGE=20 的情况下，这基本等于遍历全图——和现有的全实体感知差不多。后续优化。

## 本体变更

- [ ] ontology.md：加 cell_resources 数据结构说明
- [ ] cross_references：Consume → cell_resources（植物路径）

## 架构反馈

1. **双轨过渡**：植物 Entity 暂时保留（渲染用），cell_resources 是新的消耗数据源。后续 handoff 移除植物 Entity
2. **build_env 的虚拟 ID**：用高位编码坐标区分资源和真实 Entity。这是临时方案——长期应统一资源和实体的搜索接口
3. **性能**：遍历格子资源比遍历 1500 实体快（HashMap 查询 vs 空间索引查询），但 build_env 的双重循环需要优化

## 智能验收

- [ ] `cargo check` 零错误
- [ ] `cargo test` 全 PASS
- [ ] 新增测试：`add_resource` + `consume_resource` + `resource_qty` 基本操作
- [ ] 新增测试：格子资源为 0 时 consume_resource 返回 false
- [ ] 集成测试仍通过（鹿吃草管线——现在走 cell_resources 而非 entity.consumed）
