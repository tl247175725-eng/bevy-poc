# 交互系统：砸与叠

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（底层交互机制——所有攻击和加工的基础）

## 架构计划
基于现有交互系统扩展：
- 左键砸：复用 `try_impact` + `DragState`，增加 HP 伤害和重新接触检测
- 右键叠：复用 `GhostPlaceMode`，增加半透明渲染
- AI 攻击：`hunt_kill` 改为逐次砸击（1 砸 = 1 HP）
- 视觉：新增红/蓝圆圈 sprite 组件

## 架构反馈
当前攻击是秒杀（wolf touches prey → instant kill）。改为逐次砸击后需要公理层的 `transform(Kill)` 改为 `transform(Smash)`，一次砸传递 1 点能量。

## 智能验收
- 断言：左键拖卡碰目标 → 红圈显示"砸"，目标 HP-1
- 断言：左键连续碰同一目标不松开 → 不触发第二砸
- 断言：右键幽灵放置 → 半透明、无碰撞
- 断言：`smoke test` 中 predation > 0（动物能通过砸击捕猎）

---

## P0：砸击系统

### 视觉

卡牌右上角红色圆形 badge：
```rust
#[derive(Component)]
pub struct SmashBadge {
    pub count: u32,  // 本次累计砸数
}
```

砸中时：
- 红圈出现，白字显示"砸"
- 目标卡牌抖动（`transform.translation.x += sin(tick * 30) * 2.0`，持续 5 帧）
- 连续砸 → 数字累加：2 → 3 → ...
- 鼠标拉开 → badge 消失，计数重置

### 碰撞检测

当前拖拽放置时 `try_place_entity` 检测目标格。砸需要额外检测：**拖动过程中是否碰到其他卡牌。** 当拖拽的卡与目标卡距离 < 碰撞阈值时触发砸。

```rust
// 在 update_drag_follow 中
let smash_distance = CARD_SIZE * 0.8;
if chebyshev_distance(drag_card.x, drag_card.y, target_card.x, target_card.y) <= smash_distance {
    if !smash_cooldown {
        // 触发砸击
        apply_smash(world, drag_entity, target_entity);
        smash_cooldown = true; // 防止连续蹭
    }
} else {
    smash_cooldown = false; // 拉开后可以再砸
}
```

### 砸击效果

| 场景 | 效果 |
|---|---|
| 攻击（石→羊、狼→羊） | 目标 HP -1，红圈计数 |
| 加工（石→石） | 红圈计数，2 次后 → 碎石 |
| 砸死 | HP 归零 → 变为尸体/碎片 |

---

## P0：叠放系统

### 视觉

右键拖拽时：
- 卡牌变为半透明（alpha = 0.5）
- 无碰撞（不触发 compose 检查）
- 放置时显示蓝色圆圈"叠"

```rust
// 右键幽灵
commands.entity(dragged_entity).insert(Sprite {
    color: original_color.with_alpha(0.5),
    ..
});
```

### 叠放效果

放在目标上 → 蓝色圆圈"叠" → 触发合成配方：
- 石 + 石 → 石堆（改变本质）
- 木 + 石 → 石斧

RecipeBook 已有配方系统，叠加叠放置触发。

---

## P1：AI 攻击改为砸

当前 `hunt_kill` 一碰就死。改为逐次砸：

```rust
fn hunt_kill(world, hunter_id, prey_id, hunter_def) {
    let prey_hp = world.entities[&prey_id].hp;
    // 每 tick 只能砸 1 次（模拟拉开再碰）
    let cooldown = world.entities[&hunter_id].hunt_cooldown > 0.0;
    if cooldown { return; }
    
    // 1 砸 = 1 HP
    world.entities.get_mut(&prey_id).hp -= 1;
    world.entities.get_mut(&hunter_id).hunt_cooldown = 1.0; // 1 tick cooldown
    
    if world.entities[&prey_id].hp <= 0 {
        // 猎物死亡
        transform(Kill, hunter, prey);
        remove_entity(prey_id);
        spawn_corpse(prey_type, px, py);
    }
}
```

HP 系统需要配合：小型动物 HP=1（一砸死），大型动物 HP=3+（需要多次砸）。

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/ui_interaction.rs` | 砸检测 + 叠幽灵透明 + SmashBadge |
| `src/interaction/mod.rs` | 砸/叠逻辑 + 配方触发 |
| `src/systems/tick_reactive.rs` | hunt_kill 改为逐次砸击 |
| `src/card_visual.rs` | 红/蓝圆圈渲染 |
| `assets/card_defs.ron` | 动物 HP 调整（小动物 HP=1，大型 3+） |
| `AIMemory/design_interaction-system_deepseek-v4.md` | 已有 |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS
- `cargo run --release -- --smoke-test` SMOKE: PASS
- 左键拖石碰石 ×2 → 紫色碎石出现
- 左键拖石碰羊 → 羊 HP-1，红圈砸
- 右键幽灵 → 半透明
- 狼追羊不再秒杀，逐次攻击
