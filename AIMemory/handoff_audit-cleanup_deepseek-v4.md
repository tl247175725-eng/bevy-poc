# 架构专项整治：消除硬编码

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（8 处致命缺陷 + 29 处高级硬编码，架构正在被侵蚀）

---

## 架构计划

本次修改全部通过现有公理/标签/引擎完成：
- **compose/traverse/perceive** 公理在 move_entity/query_near_filtered 中已集成，只需调用方检查返回值
- **EntityProfile** 已有 drives/channels/bridges/social_structure，只需消除 species fallback
- **DriveDef** 标签系统已就位，只需将硬编码物种行为翻译为标签参数
- **query_near_filtered** 已封装 perceive，只需替换旧 query_near 调用

## 架构反馈

审计暴露三个缺陷：
1. `move_entity` 调用方不检查返回值 → 公理层被动失效。以后新增 move_entity 调用点需在代码审查中强制检查
2. Entity 上大量状态布尔值（in_tree/in_pool/hidden_in_grass）在 spawn 和外层手动设置 → 应逐步迁移为从标签+介质自动推导
3. 繁殖系统（tick_reproduction）完全绕开公理和标签 → 后续专项重构

---

## P0：致命级——move_entity 返回值未检查

以下 6 处调用 `world.move_entity` 但丢弃了 `MoveResult`：

| 文件 | 行 | 函数 | 修法 |
|---|---|---|---|
| `movement.rs` | 53 | `flee_from` | 检查返回值，Blocked 时尝试其他方向 |
| `movement.rs` | 77 | `wander` | 检查返回值，Blocked 时不做移动（不尝试其他方向） |
| `tick_environment.rs` | 52 | `tick_river_bounce` | 检查返回值，Blocked 时跳过此实体 |
| `ui_interaction.rs` | 81 | `try_place_entity` | 移动前调用 `cell_composition.can_occupy` 检查 |
| `ui_interaction.rs` | 95 | `revert_drag` | 检查返回值，Blocked 时重新查找空位 |
| `interaction/mod.rs` | 145,149,154 | `try_ghost_drop` | 三处均检查返回值 |

修法示例（以 `flee_from` 为例）：
```rust
// 修改前
world.move_entity(id, nx, ny);

// 修改后
if world.move_entity(id, nx, ny) != MoveResult::Moved {
    // 尝试垂直方向
    let alt_nx = (x as i16 - tx as i16).signum().clamp(-1, 1);
    let alt_ny = 0;
    let fallback_x = (x as i16 + alt_nx).clamp(0, GRID_WIDTH as i16 - 1) as u8;
    let fallback_y = y; // 不移动 y
    if (fallback_x != x || fallback_y != y) && (fallback_x != nx || fallback_y != ny) {
        world.move_entity(id, fallback_x, fallback_y);
    }
}
```

---

## P0：致命级——手动设状态绕过公理

### tick_reactive.rs — DriveBehavior::Hide（行 461-468）
当前：`e.hidden_in_grass = true` 或 `e.in_burrow = true` 直接在代码里设。
修法：改为设置 Entity 上的临时标志，由 CellComposition 在 occupy 时通过 `entity_occupies_active_cell` 自动处理。暂不重构（涉及 CellComposition 架构变更），但标记 `// FIX: should derive from conceal axiom`。

### ui_interaction.rs — in_pool 硬编码（行 82-88）
当前：UI 拖放后手动根据 type_name 设置 in_pool。
修法：删除这段硬编码。`spawn_with_sex` 已根据 type_name 设置 in_pool，移动不应改变该状态。如果实体移入水池需要变为 in_pool，应由 traverse 公理自动推导（后续专项）。

---

## P1：高级——消除物种名称硬编码

### tick_reactive.rs（最严重）

| 行 | 问题 | 修法 |
|---|---|---|
| 129-138 | `type_name == "wolf"` `"fox"` 分发 | 通过 `DriveDef` 中的 `behavior: ReturnDen` 标签统一驱动，wolf/fox 各加 `drive:return_den` 标签 |
| 119-123 | `"fish" \|\| "waterBug"` 水生物种检查 | 替换为检查 `profile.native_medium == "water"` |
| 330 | `hunter_def.type_name == "wolf"` 群猎规模 | 替换为标签 `pack_hunter` 检查（已有 `is_pack_hunter` 函数） |
| 673 | `type_name == "wolf"` 携带尸体 | 替换为 `card_has_capability(def, "capability.carry")` |
| 542-571 | `"fieldMouse"/"bambooRat"` 取食 | 替换为标签驱动的 `drive:seek` 目标 |

### tick_reproduction.rs（全文件）

当前 9 个 `try_reproduce_*` 函数按物种硬编码。
替为统一函数 `try_reproduce(world, id, def)` 通过标签参数化：
```
repro_cycle: 30      → 繁殖周期（tick）
repro_pop_cap: 6     → 种群上限
repro_litter: 2      → 每胎数量
repro_require_tag: grass  → 需要有某标签的卡在附近
```
每个物种加对应标签，不需要 `try_reproduce_sheep`/`try_reproduce_rabbit` 等。

### world_rules.rs

| 行 | 问题 | 修法 |
|---|---|---|
| 253-260 | `corpse_type_for` 物种映射 | 改为读取卡牌标签 `corpse_of: sheep` |
| 82,470,476 | 多处 `query_near` 绕过感知 | 替换为 `query_near_filtered` |
| 451-458 | `is_grass` type_name fallback | 移除，纯靠标签 |
| 503 | `ecosystem_behavior_key_legacy` 物种匹配 | 确认 RuleIndex 稳定后删除 legacy 路径 |
| 473-486 | `wolves_near` 物种硬编码 | 改为 `predators_near` 通用函数 |

### event_registry.rs

| 行 | 问题 | 修法 |
|---|---|---|
| 175-179 | fear range 按物种 | 改为读取 `EntityProfile.channels[].range` |
| 70,97,142 | `"fish" \|\| "waterBug"` 特殊处理 | 标签 `is_sessile` 或检查 `native_medium == "water"` |

### selection_info.rs

| 行 | 问题 | 修法 |
|---|---|---|
| 238-248 | wolf/fox UI 统计 | 改为标签 `meat_diet`/`scavenger` 驱动 UI |
| 345-363 | 巢穴计数的 wolf/wolfCub | 改为标签 `den_resident`/`juvenile` 计数 |
| 437 | den 覆盖检测 | 改为标签 `overlay` |

### world_state.rs

- `sheep_count()`/`wolf_count()`/`grass_count()` → 替换为 `count_by_tag(tag)`
- `is_autonomous` 中的 `type_name == "player"` → 替换为标签 `player`

### axioms/profile.rs

移除所有 type_name fallback：
- `parse_size`: `"waterBuffalo"` → 在 card_defs 加 `size:3`
- `parse_native_medium`: `"algae"` → 在 card_defs 加 `medium:water`
- `parse_bridges`: `"waterBuffalo"` → 在 card_defs 加 `bridge:land->water`
- `parse_channels`: type_name fallback → 在 card_defs 明确加 perception 标签

### 其他文件

| 文件 | 问题 | 修法 |
|---|---|---|
| `tick_aquatic.rs:45` | `"algae"` 特殊豁免 | 标签 `primary_producer` |
| `tick_containment.rs:13-54` | 树种产品硬编码 | 标签 `produces:acorn` |
| `tick_harvest.rs:26-46` | 收获物品种类匹配 | 标签 `harvest_product:caltropFruit` |
| `tick_environment.rs:154` | 尸体腐烂时间 | 标签 `decay_time:300` |
| `player/*` | 多处物种匹配 | 改为标签检查 |

---

## P1：中级——perception 绕过

以下文件使用 `spatial_index.query_near()` 替代 `world.query_near_filtered()`：

| 文件 | 行 | 修法 |
|---|---|---|
| `event_registry.rs` | 160 | `query_near_filtered` |
| `rule_index.rs` | 338,380,382,402,403 | `query_near_filtered` |
| `world_rules.rs` | 82,470,476 | `query_near_filtered` |
| `player/brain_tags.rs` | 145 | `query_near_filtered` |
| `player/brain_world.rs` | 47 | `query_near_filtered` |
| `systems/tick_reactive.rs` | 77,82 | `query_near_filtered` |

---

## 涉及文件（完整清单）

| 文件 | 改动类型 |
|---|---|
| `src/systems/movement.rs` | 致命：返回值检查 |
| `src/systems/tick_environment.rs` | 致命：返回值检查 |
| `src/ui_interaction.rs` | 致命：返回值检查 + 移除 in_pool 硬编码 |
| `src/interaction/mod.rs` | 致命：返回值检查 |
| `src/systems/tick_reactive.rs` | 致命：Hide 标记 + 高级：消除 5 处物种匹配 + 中级：query_near_filtered |
| `src/systems/tick_reproduction.rs` | 高级：全文件重构为标签驱动 |
| `src/world_rules.rs` | 高级：corpse_type_for + 5 处 |
| `src/event_registry.rs` | 高级：3 处 + 中级：perception |
| `src/selection_info.rs` | 高级：4 处 |
| `src/world_state.rs` | 高级：count 方法 + player 检测 |
| `src/axioms/profile.rs` | 高级：移除 5 个 type_name fallback |
| `src/rule_index.rs` | 中级：5 处 query_near_filtered |
| `src/player/brain_tags.rs` | 中级：perception |
| `src/player/brain_world.rs` | 中级：perception |
| `src/systems/tick_aquatic.rs` | 高级：algae 豁免 |
| `src/systems/tick_containment.rs` | 高级：生产者物种匹配 |
| `src/systems/tick_harvest.rs` | 高级：收获物种匹配 |
| `assets/card_defs.ron` | 高级：加标签替代 type_name fallback |

## 验收

1. `cargo check` 0 错误
2. `cargo test --release` 全 PASS
3. `cargo run --release -- --smoke-test` SMOKE: PASS
4. 游戏中所有搜索到的 `type_name == "wolf"` 等物种硬编码均已消除或被标记 `// FIX`
5. 6 处 move_entity 返回值均已检查

## 约束

- 不碰公理层四条定律本身
- 不碰地形/生成代码
- 不改变游戏行为（只重构内部实现）
