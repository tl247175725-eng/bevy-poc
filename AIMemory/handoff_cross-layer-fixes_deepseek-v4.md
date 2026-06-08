# 跨层修复：食草 tick + 堆叠 + 边界 + 选中 + 字体 + 集成测试

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-07
**Priority**: P0（游戏不可玩）
**诊断依据**: 逐文件代码审查。167 测试 PASS 但游戏不可玩——交叉层断裂。

---

## P0-1：main_tick 加食草基线 tick

**文件**: `src/systems/main_tick.rs`

当前只 tick 捕食者（`mark_baseline_predator_patrol` + `flush_predator_patrol`）。食草动物只有在 spawn 或 move 时才被 tick——但它们需要被 tick 才能 move。死锁。

**修法**: 在同文件加：

```rust
fn mark_baseline_herbivore_tick(world: &mut WorldState) {
    for entity in world.entities.values() {
        if entity.is_corpse || entity.in_den { continue; }
        let Some(def) = world.card_defs.get(&entity.type_name) else { continue; };
        if card_has_tag(def, "herbivore") || card_has_tag(def, "omnivore.small")
            || card_has_capability(def, "capability.forage")
        {
            entity.needs_grazing_tick = true;
        }
    }
}

fn flush_herbivore_tick(world: &mut WorldState, delta: f32) {
    let ids: Vec<EntityId> = world.entities.iter()
        .filter(|(_, e)| e.needs_grazing_tick)
        .map(|(id, _)| *id)
        .collect();
    for id in ids {
        if let Some(e) = world.entities.get_mut(&id) {
            e.needs_grazing_tick = false;
        }
        let Some(def) = world.entities.get(&id)
            .and_then(|e| world.card_defs.get(&e.type_name).cloned())
        else { continue; };
        EventRegistry::tick_non_predator_ecology(world, id, &def, delta);
    }
}
```

`main_tick` 中加一行 `flush_herbivore_tick(world, delta)` ——放在 `flush_predator_patrol` 之后即可。

**约束**: `handle_move` 中 `tick_non_predator_ecology` 调用时把 `needs_grazing_tick` 设 false，避免同一帧重复 tick。

---

## P0-2：堆叠 Y 偏移

**文件**: `src/render/card_view.rs` + `src/card_visual.rs`

`STACK_OFFSET_Y = 4.0` 定义了但没用于位置——只改了 z 序。同格 3 张卡叠在一起不可见。

**修法**: `sync_card_visuals`（card_visual.rs 第 58 行 `card_world_pos` 调用）：
`card_world_pos` 的 Y 坐标加 `stack_index * STACK_OFFSET_Y`。`stack_indices` 已返回 `HashMap<(x,y), Vec<id>>`，直接用。

---

## P0-3：卡出地图边界 clamp

**文件**: `src/world_state.rs` 的 `move_entity` 函数

当前无边界检查。

**修法**: `move_entity` 入口加：
```rust
let x = x.clamp(0, GRID_WIDTH as u8 - 1);
let y = y.clamp(0, GRID_HEIGHT as u8 - 1);
```

---

## P1-1：选中过滤容纳卡

**文件**: `src/ui_interaction.rs` 的 `handle_selection_click`

点击格子→返回该格第一张卡。但 `in_tree`/`in_pool`/`in_ground` 的卡虽然不可见，仍可能被选中。

**修法**: `resolve_selection_card` 里过滤掉 `entity.in_tree || entity.in_pool || entity.in_ground || entity.in_den || entity.in_burrow`。

---

## P1-2：字体像素感

**文件**: `src/card_visual.rs` 第 146、163 行

卡牌文字挂 WorldRoot 下，WorldRoot scale = `(zoom, -zoom, 1)`。放大时文字光栅化尺寸不变但被世界缩放拉伸——像素感。

**修法**: 文字不跟 WorldRoot 缩放。法一：文字挂在独立 Camera（UI 层）。法二（简单）：`sync_card_visuals` 更新时把 `font_size` 设为 `12.0 / view.zoom`，抵消世界缩放。法二更轻量，推荐。

---

## P1-3：跨层集成测试

**文件**: 新建 `tests/integration_cross_layer.rs`（5 条）

| # | 断言 |
|----|------|
| 1 | `cell_center(5,3)` → `world_to_grid` → 返回 `(5,3)`（坐标往返一致） |
| 2 | 食草实体在 `flush_herbivore_tick` 后被调用过（`needs_grazing_tick` 初始 true，flush 后 false） |
| 3 | `card_world_pos(0,0,1,...).x` ≥ 0 且 ≤ `world_width()` |
| 4 | `stack_indices` 对同格 3 实体返回 `vec![id1,id2,id3]` |
| 5 | `in_tree` 实体不被 `stack_indices` 计入 |

---

## 涉及文件

| 文件 | 改动 |
|------|------|
| `src/systems/main_tick.rs` | P0-1: 食草 baseline tick |
| `src/world_state.rs` | P0-1: needs_grazing_tick 字段 + P0-3: 边界 clamp |
| `src/card_visual.rs` | P0-2: 堆叠 Y 偏移 |
| `src/ui_interaction.rs` | P1-1: 容纳过滤 |
| `tests/integration_cross_layer.rs` | P1-3: 新建 5 条 |

## 验收

- `cargo test`：167 + 5 = 172 PASS
- `cargo run --release`：羊动、兔动、所有食草动物正常觅食移动；同格多卡可见堆叠偏移；卡不跑出边界；选中不误选容纳卡；字体放大清晰

## 约束

- 不碰 Observer/Rete/Player AI
- 不碰 `card_defs.ron`
- 记 FIX_LOG
