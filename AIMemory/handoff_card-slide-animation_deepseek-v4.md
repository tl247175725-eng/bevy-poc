# 卡牌滑动动画 — 消除闪现感

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-09
**Priority**: P1（视觉体验——卡牌移动看上去是闪现，需要平滑滑动）

---

## 问题

当前 `sync_card_visuals` 直接把 `transform.translation = new_pos`，每 tick 瞬间跳到新位置。看起来像闪现/传送。

## 目标

卡牌从一个格子移动到相邻格子时，有一个 **0.15~0.2 秒的平滑滑动动画**（推箱子那种感觉），缓入缓出。

## 方案

使用 `bevy_tweening` crate。它在 `sync_card_visuals` 检测到位置变化时，用 `move_to()` 队列一个 tween 动画。

### Add dependency

`Cargo.toml`:
```toml
bevy_tweening = "0.15"
```

### Modify `sync_card_visuals` (src/card_visual.rs)

核心改动：当检测到卡牌需要移动时，不直接设 `transform.translation`，而是调用 `commands.entity(entity_id).move_to(new_pos, Duration::from_secs(0.15), EaseFunction::QuadraticOut)`。

具体做法：

```rust
// In sync_card_visuals, when updating existing card positions:

// Instead of:
// transform.translation = pos;

// Do:
if transform.translation.distance(pos) > 1.0 {
    // Position changed — animate the slide
    commands.entity(entity_id).move_to(
        pos,
        Duration::from_secs(0.15),
        EaseFunction::QuadraticOut,
    );
} else {
    // Same position — no animation needed (avoid tween churn)
    transform.translation = pos;
}
```

新卡（`spawn_card_visual`）直接设 `transform.translation = pos` 不需要动画（它还没被玩家看到过）。

### Bevy plugin registration (src/plugins.rs)

```rust
app.add_plugins(TweeningPlugin);
```

## 涉及文件

| 文件 | 改动 |
|---|---|
| `Cargo.toml` | 加 `bevy_tweening = "0.15"` |
| `src/card_visual.rs` | `sync_card_visuals` 中，已存在卡的移动改用 `move_to` |
| `src/plugins.rs` | 注册 `TweeningPlugin` |

## 验收

1. `cargo check` 通过
2. `cargo test --release` 全 PASS
3. 启动游戏 → 食草动物移动时看到平滑滑动，不是瞬移

## 约束

- 不碰 game_constants.rs
- 不碰 world_state.rs 或任何模拟层代码
- 纯视觉层改动
