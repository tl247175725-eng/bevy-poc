# 修复卡牌移动动画：消除闪现

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P1（视觉——卡牌瞬移而非滑动）

## 架构计划
当前 `bevy_tweening` 的 `TweenCompleted` 事件不触发，导致动画阻塞被禁用，卡牌瞬移。不使用外部 crate，手写一个轻量 lerp 系统：每帧将卡牌位置从旧值线性插值到目标值。与公理层无关，纯视觉层改动。

## 架构反馈
`bevy_tweening` 依赖不可靠。手写 lerp 更符合项目"依赖最小化"原则。

---

## 方案

不依赖 `bevy_tweening`。在 `CardVisual` 组件上加两个字段：
- `visual_pos: Vec3` — 当前视觉位置（每帧 lerp 更新）
- `target_pos: Vec3` — 目标位置（来自模拟层的格坐标）

在 `sync_card_visuals` 中，不再直接设 `transform.translation`，而是更新 `target_pos`。新增系统 `slide_cards` 每帧 lerp `visual_pos` 向 `target_pos`。

```rust
// CardVisual 新增
pub struct CardVisual {
    pub entity_id: u64,
    pub visual_pos: Vec3,
    pub target_pos: Vec3,
}

// 新增系统：每帧 lerp
fn slide_cards(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CardVisual)>,
) {
    let speed = 8.0; // 视觉移动速度（越高越快）
    for (mut transform, mut cv) in &mut query {
        let dt = time.delta_seconds() * speed;
        cv.visual_pos = cv.visual_pos.lerp(cv.target_pos, dt.clamp(0.0, 1.0));
        transform.translation = cv.visual_pos;
    }
}
```

## 修改范围

| 文件 | 改动 |
|---|---|
| `src/card_visual.rs` | `CardVisual` 加 visual_pos/target_pos；新增 `slide_cards` 系统；`sync_card_visuals` 改设 target_pos 而非直接设 transform |
| `src/plugins/mod.rs` | 注册 `slide_cards` |
| `src/sim_clock.rs` | 恢复动画阻塞（`playback.in_progress` 检测），但用简单计数器替代 TweenCompleted 事件 |
| `src/render/move_animation.rs` | 可以删除或简化（不再依赖 bevy_tweening） |

## 动画阻塞（简化版）

不用 `TweenCompleted` 事件。用帧计数：
- 动画启动时记录 `total_slide_duration = 0.15s`
- 每帧检查所有 `CardVisual`：如果 `visual_pos.distance(target_pos) < 0.1`，认为动画完成
- 全部完成后允许下一个 tick

## 移动速度标签保留

`move_speed: normal` → speed=8.0  
`sprint:*` → speed=15.0（追击/逃命时）

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS
- 启动游戏 → 卡牌移动时平滑滑动，不瞬移
- 动物移速不超过 0.15s 视觉滑动
