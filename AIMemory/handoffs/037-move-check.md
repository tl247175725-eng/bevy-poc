# Handoff 037 — Move 公理：地形阻力同构模型

## 三柱强制检查

| 柱子 | 用哪个 |
|---|---|
| 标签 | 地形：卡牌 type_name 查地形类型。动物：`body_plan:*` + `body_size:*` + `habitat:aquatic` + `capability:swim` |
| 元数值 | 地形阻力表（新增常量表）+ 致命地形每 tick 伤害 |
| 元动作 | `MetaAction::Move { dx, dy }` — 已有，增加阻力验证 |
| 公理 | 新建 `move_check()` 在 `src/axioms/move_check.rs` — 纯函数，从地形+标签算阻力 |

## 架构计划

**改什么：** 新建 `src/axioms/move_check.rs` + 改 `src/axioms/mod.rs` + 改 `src/systems/main_tick.rs`（3 文件）

**为什么：** Move 当前直接改坐标，鹿能走进深水、鱼能走上岸。同构模型要求地形阻力从标签推导。

### 设计决策（策划已确认）

- **地形阻力模型**：每种地形对每种 body_plan 有不同阻力（生态学"景观阻力面"）
- **阻力等级**：free(1)、slow(2)、hard(4)、lethal — 数字=需要几个 tick 才能移动一格
- **致命地形**：能进但每 tick 掉血（鱼上岸窒息、鹿掉深水溺水）
- **不主动进入**：高阻力地形在需求匹配搜索时自然被避开（低阻力路线优先）
- **浅水过膝**：大型动物阻力比小型低（水到脚踝 vs 到腰）
- **熊捞鱼**：需求匹配引擎自动选择"虽然有阻力但食物在那"的路线——无需专门逻辑

### 改动 1：新建 `src/axioms/move_check.rs`

```rust
use crate::tags::{TagBits, tag};

/// 地形阻力等级
pub enum TerrainCost {
    Free,          // 1 tick 走一格
    Slow,          // 2 tick
    Hard,          // 4 tick
    Lethal,        // 能进，但每 tick 掉血
}

/// 从目标格的地形类型 + 动物标签 → 阻力
pub fn terrain_resistance(
    terrain_type: &str,
    entity_tags: &TagBits,
) -> TerrainCost {
    let is_aquatic = entity_tags.has(tag::HAB_AQUATIC.bit)
        || entity_tags.has(tag::CAP_SWIM.bit);
    let is_large = entity_tags.has(tag::SIZE_LARGE.bit)
        || entity_tags.has(tag::SIZE_HUGE.bit);
    let is_fish = entity_tags.has(tag::PLAN_FISH.bit);
    let is_serpentine = entity_tags.has(tag::PLAN_SERPENTINE.bit);

    match terrain_type {
        "abyss_pool" => {
            if is_fish { TerrainCost::Free }
            else if is_aquatic { TerrainCost::Slow }
            else { TerrainCost::Lethal }
        }
        "shallow_water" => {
            if is_fish || is_aquatic { TerrainCost::Free }
            else if is_large { TerrainCost::Slow }      // 大型动物涉水
            else { TerrainCost::Hard }                    // 小型动物困难
        }
        "wetland" => {
            if is_aquatic || is_serpentine { TerrainCost::Free }
            else if is_large { TerrainCost::Free }        // 大型动物不受湿地影响
            else { TerrainCost::Slow }                    // 小型动物泥泞减速
        }
        "grassland" => TerrainCost::Free,
        "broadleaf_forest" => {
            if is_large { TerrainCost::Slow }             // 大型动物被树木减速
            else { TerrainCost::Free }
        }
        "foothills" => {
            if is_fish { TerrainCost::Lethal }
            else { TerrainCost::Slow }                    // 山地普遍减速
        }
        "cliff" => {
            if is_fish { TerrainCost::Lethal }
            else { TerrainCost::Hard }                    // 崖壁极难通行
        }
        _ => TerrainCost::Free,
    }
}

/// 阻力→移动 tick 成本
pub fn move_cost_ticks(cost: &TerrainCost) -> u32 {
    match cost {
        TerrainCost::Free => 1,
        TerrainCost::Slow => 2,
        TerrainCost::Hard => 4,
        TerrainCost::Lethal => 1, // 能进，但掉血
    }
}

/// 致命地形每 tick 伤害
pub fn lethal_terrain_damage() -> i32 {
    // 后续可从 meta_values 配置
    2
}
```

### 改动 2：`src/axioms/mod.rs`

```rust
pub mod move_check;
pub use move_check::terrain_resistance;
```

### 改动 3：`src/systems/main_tick.rs` Move 分支

```rust
MetaAction::Move { dx, dy } => {
    let Some(entity) = world.entities.get(&entity_id) else { return };
    let new_x = ((entity.x as i16) + dx).clamp(0, 255) as u8;
    let new_y = ((entity.y as i16) + dy).clamp(0, 255) as u8;

    // 查目标格地形
    let terrain = crate::terrain::terrain_at(world, new_x, new_y);

    // 查动物标签
    let entity_tags = world.card_defs.get(&entity.type_name)
        .map(|d| &d.tag_bits);

    if let Some(tags) = entity_tags {
        let cost = crate::axioms::move_check::terrain_resistance(terrain, tags);

        match cost {
            TerrainCost::Lethal => {
                // 能进但掉血
                if let Some(e) = world.entities.get_mut(&entity_id) {
                    e.x = new_x;
                    e.y = new_y;
                    e.hp = e.hp.saturating_sub(move_check::lethal_terrain_damage());
                    world.spatial_index.move_entity(entity_id, new_x, new_y);
                }
            }
            _ => {
                // 正常移动（阻力暂不影响速度——后续 handoff 通过 move cooldown 实现）
                if let Some(e) = world.entities.get_mut(&entity_id) {
                    e.x = new_x;
                    e.y = new_y;
                    world.spatial_index.move_entity(entity_id, new_x, new_y);
                }
            }
        }
    }
}
```

注意：阻力导致的移动减速（2 tick / 4 tick）通过后续 handoff 的 move_cooldown 实现。本次只做"致命地形掉血"和"公理验证框架"。

## 本体变更

- [ ] ontology.md traverse 公理状态 ❌→✅
- [ ] cross_references 补充 habitat/body_plan/capability → Move

## 架构反馈

1. **terrain_resistance 用地形字符串而非 type_name**：`terrain_at()` 已有函数返回地形类型字符串，不涉及 type_name 匹配
2. **阻力减速延后**：本次只做地形验证框架+致命伤害。speed 模拟需要 Entity 上加 move_cooldown 字段，是后续改动
3. **旧 traverse 公理成废代码**：不删——后续统一清理旧 axiom 系统

## 智能验收

- [ ] `cargo check` 零错误
- [ ] `cargo test` 全 PASS
- [ ] 新增测试：鱼在深水 Free、鱼在草地 Lethal
- [ ] 新增测试：鹿在草地 Free、鹿在深水 Lethal
- [ ] 新增测试：大型动物在浅水 Slow、小型动物在浅水 Hard
- [ ] `apply_meta_action` 的 Move 分支有地形检查
