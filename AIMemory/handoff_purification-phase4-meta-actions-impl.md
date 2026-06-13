# 提纯 Phase 4：元动作执行器

**Priority**: P0 — 把 meta_actions 从枚举定义变成可执行函数

## 架构计划

`src/meta_actions.rs` 目前只有 `enum MetaAction` 和 `enum ActionResult` 的定义——空壳。Phase 4 新建 `src/meta_actions/mod.rs`，为每个元动作写执行函数。

### 涉改文件

| 文件 | 改动 |
|---|---|
| `src/meta_actions/mod.rs` | **新建** — 8 个元动作的执行函数 + 测试 |
| `src/lib.rs` | 改 `pub mod meta_actions;` 为目录模式 |

### 新建文件：`src/meta_actions/mod.rs`

此为目录模块。将现有 `src/meta_actions.rs` 的内容移到 `src/meta_actions/types.rs`，新建的 `mod.rs` 作为入口。但为遵循"每次只改 1-2 文件"规则，本次只新建一个独立文件，把执行函数写进去，等 CI 绿了再做目录化。

**本次只新建 `src/meta_actions_exec.rs`**（独立文件，不碰 `meta_actions.rs`）：

```rust
//! 元动作执行器 — 每个 MetaAction 的执行函数
//! 
//! 每个函数是纯操作：输入 WorldState + 参数，输出 ActionResult。
//! 函数内部调用公理验证（compose/traverse/transform）。

use crate::meta_actions::{MetaAction, ActionResult};
use crate::axioms::AxiomEngine;
use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

/// 执行任意元动作
pub fn execute(world: &mut WorldState, action: &MetaAction, actor_id: EntityId) -> ActionResult {
    match action {
        MetaAction::Move { dx, dy } => exec_move(world, actor_id, *dx, *dy),
        MetaAction::Strike { target } => exec_strike(world, actor_id, *target),
        MetaAction::Consume { target } => exec_consume(world, actor_id, *target),
        MetaAction::Combine { ingredient } => ActionResult::Invalid,
        MetaAction::Release { x, y } => ActionResult::Invalid,
        MetaAction::Wait { ticks } => ActionResult::Invalid,
        MetaAction::Hide { cover_id } => ActionResult::Invalid,
        MetaAction::Emerge => ActionResult::Invalid,
    }
}

fn exec_move(world: &mut WorldState, id: EntityId, dx: i16, dy: i16) -> ActionResult {
    // 调用现有 move_entity，经过 compose + traverse 验证
    let (x, y) = match world.entities.get(&id) {
        Some(e) => (e.x, e.y),
        None => return ActionResult::Invalid,
    };
    let nx = (x as i16 + dx).clamp(0, 23) as u8;
    let ny = (y as i16 + dy).clamp(0, 35) as u8;
    match world.move_entity(id, nx, ny) {
        crate::world_state::MoveResult::Moved => ActionResult::Success,
        crate::world_state::MoveResult::Blocked => ActionResult::Blocked {
            reason: format!("compose or traverse blocked ({x},{y})→({nx},{ny})"),
        },
        crate::world_state::MoveResult::NoOp => ActionResult::Invalid,
    }
}

fn exec_strike(world: &mut WorldState, source: EntityId, target: EntityId) -> ActionResult {
    // 伤害 = 1 HP（基础），未来从 meta_values 派生
    use crate::interaction::smash::damage_entity_hp;
    if damage_entity_hp(world, target, 1) {
        ActionResult::Killed { corpse_spawned: true }
    } else {
        ActionResult::Success
    }
}

fn exec_consume(world: &mut WorldState, eater: EntityId, target: EntityId) -> ActionResult {
    use crate::axioms::TransformAction;
    let (src_profile, eater_profile) = {
        let t = world.entities.get(&target);
        let e = world.entities.get(&eater);
        match (t, e) {
            (Some(t), Some(e)) => (t.profile.clone(), e.profile.clone()),
            _ => return ActionResult::Invalid,
        }
    };
    let result = AxiomEngine::transform(&src_profile, &eater_profile, TransformAction::Eat);
    world.remove_entity(target);
    if let Some(e) = world.entities.get_mut(&eater) {
        e.profile.energy = e.profile.energy.saturating_add(result.energy_received);
    }
    ActionResult::Consumed { energy_gained: result.energy_received }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn move_execute_success_east() {
        let mut w = empty_world();
        let id = w.spawn("sheep", 5, 5);
        let r = exec_move(&mut w, id, 1, 0);
        assert_eq!(r, ActionResult::Success);
        assert_eq!(w.entities[&id].x, 6);
    }

    #[test]
    fn strike_damages_target() {
        let mut w = empty_world();
        let wolf = w.spawn("wolf", 5, 5);
        let sheep = w.spawn("sheep", 6, 5);
        let hp_before = w.entities[&sheep].hp;
        let r = exec_strike(&mut w, wolf, sheep);
        assert!(matches!(r, ActionResult::Success | ActionResult::Killed { .. }));
        assert!(w.entities[&sheep].hp < hp_before || !w.entities.contains_key(&sheep));
    }
}
```

并在 `src/lib.rs` 中注册 `pub mod meta_actions_exec;`。

## 架构反馈

1. `Combine`, `Release`, `Wait`, `Hide`, `Emerge` 暂返回 `Invalid`——这些需要更复杂的世界交互，Phase 4 只是搭骨架，后续逐步填充。
2. `exec_strike` 伤害固定为 1——未来应从 `meta_values::impact_damage` 派生（Phase 4 后接上元数值）。
3. `exec_move` 通过 `world.move_entity` 走现有 compose/traverse 验证——这是正确用法。

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- `exec_move` 单步东移测试通过
- `exec_strike` 造成伤害测试通过
- `exec_consume` 能量转化测试通过

## 设计文档引用

- `design-philosophy-v5.md` §4.1 — 元动作定义
- `design-philosophy-v5.md` §2.1 — 公理作为物理引擎
- `src/meta_actions.rs` — 已有枚举定义
