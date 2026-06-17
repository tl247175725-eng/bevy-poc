# Handoff 022 — 动画系统（刚体运动 + Emoji 标签）

## 架构计划

**改什么：** 新建 `src/render/animation.rs`，修改 `src/render/mod.rs`（2 文件）
**做什么：** 四个刚体动画 + Emoji 浮动标签

### 动画类型

```rust
#[derive(Component)]
pub struct AttackAnimation {
    pub target_pos: Vec3,
    pub source_pos: Vec3,
    pub timer: f32,       // 0→1 进度
    pub shake_phase: u8,  // 0=冲刺, 1-3=震动
}

#[derive(Component)]
pub struct MoveAnimation {
    pub from: Vec3,
    pub to: Vec3,
    pub timer: f32,
}

#[derive(Component)]
pub struct EatAnimation {
    pub phase: f32,       // 呼吸循环相位
    pub timer: f32,
}

#[derive(Component)]
pub struct EmojiLabel {
    pub emoji: String,    // "💥" "😋" "💀"
    pub timer: f32,       // 逐渐消失
    pub birth_y: f32,     // 起始高度
}
```

### 运动公式

```rust
// 攻击：冲刺→震动→回弹
fn attack_motion(timer: f32, from: Vec3, to: Vec3, shake_intensity: f32) -> Vec3 {
    if timer < 0.3 {  // 冲刺阶段
        let t = timer / 0.3;
        from.lerp(to, ease_in_quad(t))
    } else if timer < 0.7 {  // 震动阶段
        let shake_t = (timer - 0.3) / 0.4;
        let shake = (shake_t * 10.0).sin() * shake_intensity * (1.0 - shake_t);
        to + Vec3::new(shake, shake * 0.5, 0.0)
    } else {  // 回弹
        let t = (timer - 0.7) / 0.3;
        to.lerp(from, ease_out_cubic(t))
    }
}

// 移动：直线滑移（象棋平移）
fn move_motion(timer: f32, from: Vec3, to: Vec3) -> Vec3 {
    from.lerp(to, ease_in_out_sine(timer))
}

// 进食：呼吸缩放
fn eat_breathe(phase: f32) -> f32 {
    1.0 + (phase * std::f32::consts::TAU).sin() * 0.05
}
```

### 缓动函数

```rust
fn ease_in_quad(t: f32) -> f32 { t * t }
fn ease_out_cubic(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }
fn ease_in_out_sine(t: f32) -> f32 { -((t * std::f32::consts::PI).cos() - 1.0) / 2.0 }
```

### Emoji 渲染

每个浮动标签 = 一个 Text2d 或世界空间 billboard（始终面向相机）。出现时从模型顶上升浮起 → 缩小 → 消失。

```rust
pub fn spawn_emoji(commands: &mut Commands, pos: Vec3, emoji: &str) {
    commands.spawn((
        Text2d::new(emoji),
        Transform::from_translation(pos + Vec3::Y * 2.0),
        EmojiLabel { emoji: emoji.into(), timer: 1.5, birth_y: pos.y + 2.0 },
    ));
}
```

### 系统注册

```rust
pub fn animation_system(
    time: Res<Time>,
    mut attack_q: Query<(&mut Transform, &mut AttackAnimation)>,
    mut move_q: Query<(&mut Transform, &mut MoveAnimation)>,
    mut eat_q: Query<(&mut Transform, &mut EatAnimation)>,
    mut emoji_q: Query<(Entity, &mut Transform, &mut EmojiLabel)>,
    mut commands: Commands,
) {
    // 每帧更新所有动画的 Transform + 清理完成动画
}
pub fn remove_completed_animations(mut commands: Commands, q: Query<(Entity, &EmojiLabel)>) {
    for (e, label) in q.iter() {
        if label.timer <= 0.0 { commands.entity(e).despawn_recursive(); }
    }
}
```

## 架构反馈

- 纯刚体运动——Transform 修改，零骨骼 ✅
- Emoji 免费无版权 ✅
- 所有动画独立 Component——互不干扰 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
