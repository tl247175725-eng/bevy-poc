# 格子滑动动画 — 商业标准版

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-09
**Priority**: P1（视觉核心——替换当前闪现式动画，达到战术游戏标准）

---

## 背景

当前 `sync_card_visuals` 每帧设 `transform.translation = pos`，配合 `move_to()` tween，但：
1. **每帧重建 tween** → 动画不断重启 → 卡顿
2. **斜线移动** → `move_to` 直线插值，格子游戏必须只走上下左右
3. **动画不阻塞 tick** → 模拟层已换格但动画还在跑 → 猎物死了狼才滑到

## 目标

商业级格子滑动：0.25s/格，只走正方向，动画播完才进下一 tick。

## 设计文档

见 `AIMemory/design_movement-animation-standards_deepseek-v4.md`。

---

## 方案

### 架构

```
main_tick 执行
  │
  ├→ move_entity(A, 5,3 → 5,4)  推入 MoveEvent { entity:A, from:5,3, to:5,4 }
  ├→ move_entity(B, 6,2 → 7,3)  推入 MoveEvent { entity:B, from:6,2, to:7,3, diagonal=true }
  │
  ▼
main_tick 返回 → Bevy 渲染帧
  │
  ▼
process_move_queue system (Update 中)
  │  消费所有 MoveEvent
  │
  ├→ A: dx=0 dy=1 → 单步 tween: 0.25s CubicOut  竖向一步
  ├→ B: dx=1 dy=1 → 拆为两步 Sequence:
  │     Step1: dx=1 dy=0 横向 0.25s
  │     Step2: dx=0 dy=1 纵向 0.25s
  │
  ▼
全部 tween 完成
  │
  ▼
发送 MoveAnimationsComplete 事件
```

### 速度查表

```rust
fn move_duration(entity: &Entity) -> f32 {
    // 从标签 move_speed:xxx 查表
    // slow→0.35, normal→0.25, fast→0.18, very_fast→0.12, sprint→0.08
    // 无标签 → 0.25
}
```

---

## P0：新建 Bevy 事件

```rust
// src/sim_events.rs 新增

/// 移动动画事件 — move_entity 确认后发出
#[derive(Event)]
pub struct MoveAnimEvent {
    pub entity_id: EntityId,      // 模拟层 entity id（通过 CardVisual 查找 Bevy entity）
    pub from_x: u8,
    pub from_y: u8,
    pub to_x: u8,
    pub to_y: u8,
    pub duration_per_step: f32,   // 每格秒数（来自 move_speed 标签）
}

/// 所有移动动画播完后发出
#[derive(Event)]
pub struct MoveAnimationsComplete;
```

---

## P0：修改 `world_state.rs` move_entity 发出事件

```rust
// move_entity 末尾，移动成功后

// 计算时长
let duration = world.entities.get(&id)
    .map(|e| move_duration(e))
    .unwrap_or(0.25);

// 发出动画事件
MoveAnimEvent {
    entity_id: id,
    from_x: old_x, from_y: old_y,
    to_x: new_x, to_y: new_y,
    duration_per_step: duration,
}
// 写入 events 队列（或直接发 Bevy event 如果 world_state 可访问）
```

**注意**：如果 `world_state` 不能直接发 Bevy Event（分层问题），则将事件存入 `world.pending_move_anims: Vec<MoveAnimEvent>`，在 `main_tick` 外部由 Bevy system 消费。

---

## P0：新建渲染层动画系统

```rust
// src/render/move_animation.rs (新建)

use bevy::prelude::*;
use bevy_tweening::{Animator, EaseFunction, Sequence, Tween, TransformPositionLens};
use std::time::Duration;
use crate::card_visual::CardVisual;
use crate::coords::card_world_pos;

/// 消费 MoveAnimEvent → 播放 tween → 等待完成 → 发 MoveAnimationsComplete
pub fn process_move_queue(
    mut commands: Commands,
    mut move_events: EventReader<MoveAnimEvent>,
    mut anim_complete: EventWriter<MoveAnimationsComplete>,
    card_visuals: Query<(Entity, &CardVisual)>,
    world_state: Res<SimWorld>,  // 需要访问 WorldState 读取 entity 位置
) {
    let mut active_count = 0;

    for event in move_events.read() {
        // 找到卡牌的 Bevy entity
        let bevy_entity = find_bevy_entity(&card_visuals, event.entity_id);
        let Some(bevy_entity) = bevy_entity else { continue; };

        let dur = Duration::from_secs_f32(event.duration_per_step);

        // 当前视觉位置（卡牌当前的 transform.translation）
        let start_pos = /* from the entity's current Transform */;

        // 目标位置
        let end_pos = card_world_pos(event.to_x, event.to_y, event.entity_id.0, Some(&world_state.0));

        let dx = (event.to_x as i16 - event.from_x as i16).abs() as u8;
        let dy = (event.to_y as i16 - event.from_y as i16).abs() as u8;

        if dx > 0 && dy > 0 {
            // 斜线 → 拆成两步 Sequence
            let step1_target = card_world_pos(
                event.to_x, event.from_y,  // 先横走：X到目标，Y不变
                event.entity_id.0, Some(&world_state.0)
            );
            let step2_target = end_pos;

            let tween1 = Tween::new(
                EaseFunction::CubicOut, dur,
                TransformPositionLens { start: start_pos, end: step1_target },
            );
            let tween2 = Tween::new(
                EaseFunction::CubicOut, dur,
                TransformPositionLens { start: step1_target, end: step2_target },
            );
            let seq = tween1.then(tween2);
            commands.entity(bevy_entity).insert(Animator::new(seq));
        } else {
            // 单步直走
            let tween = Tween::new(
                EaseFunction::CubicOut, dur,
                TransformPositionLens { start: start_pos, end: end_pos },
            );
            commands.entity(bevy_entity).insert(Animator::new(tween));
        }

        active_count += 1;
    }

    // TODO: 检测所有 Animator 是否完成 → 发 MoveAnimationsComplete
    // 这一步可以用 TweenCompleted event 监听
}
```

---

## P0：动画阻塞 tick

`main_tick` 在 tick 开始前检查：如果上一帧的 `MoveAnimationsComplete` 没收到 → 等待，不推进 tick。

```rust
// 在 main_tick 入口（或调用 main_tick 的前置条件）
fn await_animations(
    mut anim_complete: EventReader<MoveAnimationsComplete>,
) -> bool {
    // 返回 true 表示所有动画完成，可以推进 tick
    !anim_complete.is_empty()
}
```

**或者更简单**：用一个 `AtomicBool` 标记 `animations_in_progress`。`main_tick` 开始时检查该标记，true → skip tick。所有 tween 完成时重置为 false。

---

## P0：移除 sync_card_visuals 中旧动画逻辑

1. 移除 `sync_card_visuals` 中所有 `move_to()` / `commands.entity(entity_id).move_to(...)` 调用
2. `sync_card_visuals` 恢复为纯粹的"位置同步"：`transform.translation = pos`（snap，不做动画）
3. 因为动画现在由 `MoveAnimEvent` → `process_move_queue` 驱动，在 tick 间播放

---

## P1：标签解析

`EntityProfile` 新增：

```rust
pub move_speed: f32,  // 每格秒数，默认 0.25
```

`build_profile` 中解析 `move_speed:fast` → 0.18, `move_speed:slow` → 0.35 等。

所有动物卡加 `move_speed` 标签：
- sheep/deer/waterBuffalo → `move_speed:slow`
- wolf → `move_speed:fast`
- rabbit → `move_speed:very_fast`
- fox → `move_speed:fast`
- 其他动物 → `move_speed:normal`

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/render/move_animation.rs` | **新建**：MoveAnimEvent 消费 → tween → 完成检测 |
| `src/sim_events.rs` | 新增 `MoveAnimEvent` + `MoveAnimationsComplete` |
| `src/world_state.rs` | `move_entity` 末尾发出 `MoveAnimEvent` |
| `src/card_visual.rs` | **移除** `sync_card_visuals` 中旧 tween 逻辑，恢复 snap |
| `src/systems/main_tick.rs` | tick 前检查 `animations_in_progress` 标记 |
| `src/axioms/profile.rs` | `EntityProfile` 加 `move_speed: f32`，`parse_move_speed` |
| `AIMemory/design_movement-animation-standards_deepseek-v4.md` | 已有（设计文档） |
| `assets/card_defs.ron` | 所有动物卡加 `move_speed:xxx` 标签 |

## 验收

1. `cargo test --release` 全 PASS
2. `cargo run --release -- --smoke-test` SMOKE: PASS
3. 启动游戏：
   - 卡牌只在四个正方向上滑动，永远不斜走
   - 动画顺滑不卡顿
   - tick 等动画播完才推进
   - 狼比羊快、兔子最快

## 约束

- 不碰公理层（compose/traverse/perceive/transform）
- 不碰 tick_reactive 行为引擎
- 不碰 WorldRules/Rete
