//! Animation components and systems — attack lunge, move slide, eat breathe, emoji popup.
//!
//! Each animation type is an independent Component; the per-frame system advances
//! timers and writes `Transform`. Completed one-shots remove their component.

use bevy::prelude::*;

use crate::game_ui_panel::UiFont;

// ── Components ─────────────────────────────────────────────────

/// Attack lunge: dash → shake → snap-back to source.
#[derive(Component)]
pub struct AttackAnimation {
    pub target_pos: Vec3,
    pub source_pos: Vec3,
    /// 0→1 progress over ~0.6 s
    pub timer: f32,
    /// 0 = dash, 1–3 = shake intensity tier
    pub shake_phase: u8,
}

/// Grid-slide from one cell to another (chess-like translation).
#[derive(Component)]
pub struct MoveAnimation {
    pub from: Vec3,
    pub to: Vec3,
    /// 0→1 progress over ~0.4 s
    pub timer: f32,
}

/// Breath-cycle scale oscillation (continuous).
#[derive(Component)]
pub struct EatAnimation {
    /// Accumulated phase for the sine loop
    pub phase: f32,
    pub timer: f32,
}

/// Floating world-space emoji tag that fades out.
#[derive(Component)]
pub struct EmojiLabel {
    pub emoji: String,
    /// Countdown from 1.5 → 0.0
    pub timer: f32,
    /// Starting Y (above entity) used for float-up offset
    pub birth_y: f32,
}

// ── Easing ─────────────────────────────────────────────────────

pub fn ease_in_quad(t: f32) -> f32 {
    t * t
}

pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

pub fn ease_in_out_sine(t: f32) -> f32 {
    -((t * std::f32::consts::PI).cos() - 1.0) / 2.0
}

// ── Motion formulas ────────────────────────────────────────────

/// Attack: dash (0–0.3) → shake (0.3–0.7) → snap-back (0.7–1.0).
pub fn attack_motion(timer: f32, from: Vec3, to: Vec3, shake_intensity: f32) -> Vec3 {
    if timer < 0.3 {
        // dash toward target
        let t = timer / 0.3;
        from.lerp(to, ease_in_quad(t))
    } else if timer < 0.7 {
        // shake around target
        let shake_t = (timer - 0.3) / 0.4;
        let shake = (shake_t * 10.0).sin() * shake_intensity * (1.0 - shake_t);
        to + Vec3::new(shake, shake * 0.5, 0.0)
    } else {
        // snap back toward source
        let t = (timer - 0.7) / 0.3;
        to.lerp(from, ease_out_cubic(t))
    }
}

/// Move: straight slide with ease-in-out.
pub fn move_motion(timer: f32, from: Vec3, to: Vec3) -> Vec3 {
    from.lerp(to, ease_in_out_sine(timer))
}

/// Eat: gentle breathing scale oscillation (±5 %).
pub fn eat_breathe(phase: f32) -> f32 {
    1.0 + (phase * std::f32::consts::TAU).sin() * 0.05
}

// ── Spawn helpers ──────────────────────────────────────────────

/// Spawn a floating emoji label that drifts up and despawns.
pub fn spawn_emoji(commands: &mut Commands, pos: Vec3, emoji: &str, font: &UiFont) {
    commands.spawn((
        Text2d::new(emoji),
        TextFont {
            font: font.0.clone(),
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(pos + Vec3::Y * 2.0),
        EmojiLabel {
            emoji: emoji.into(),
            timer: 1.5,
            birth_y: pos.y + 2.0,
        },
    ));
}

// ── Systems ────────────────────────────────────────────────────

/// Advance all animation timers and write transforms each frame.
pub fn animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut attack_q: Query<(Entity, &mut Transform, &mut AttackAnimation)>,
    mut move_q: Query<(Entity, &mut Transform, &mut MoveAnimation)>,
    mut eat_q: Query<(&mut Transform, &mut EatAnimation)>,
    mut emoji_q: Query<(Entity, &mut Transform, &mut EmojiLabel, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    let one_shot_duration = 0.6_f32;
    let move_duration = 0.4_f32;

    // Attack — one-shot ~0.6 s
    for (entity, mut transform, mut anim) in &mut attack_q {
        if anim.timer >= 1.0 {
            transform.translation = anim.source_pos;
            commands.entity(entity).remove::<AttackAnimation>();
            continue;
        }
        anim.timer = (anim.timer + dt / one_shot_duration).min(1.0);
        let shake = if anim.shake_phase > 0 { 2.0 } else { 0.0 };
        transform.translation =
            attack_motion(anim.timer, anim.source_pos, anim.target_pos, shake);
    }

    // Move — one-shot ~0.4 s
    for (entity, mut transform, mut anim) in &mut move_q {
        if anim.timer >= 1.0 {
            transform.translation = anim.to;
            commands.entity(entity).remove::<MoveAnimation>();
            continue;
        }
        anim.timer = (anim.timer + dt / move_duration).min(1.0);
        transform.translation = move_motion(anim.timer, anim.from, anim.to);
    }

    // Eat — continuous breathing oscillation
    for (mut transform, mut anim) in &mut eat_q {
        anim.phase += dt * 3.0; // ~3 cycles / s
        let scale = eat_breathe(anim.phase);
        transform.scale = Vec3::splat(scale);
    }

    // Emoji — drift up and fade alpha
    for (_entity, mut transform, mut label, mut text_color) in &mut emoji_q {
        label.timer -= dt;
        let t = (label.timer / 1.5).clamp(0.0, 1.0);
        transform.translation.y = label.birth_y + (1.0 - t) * 0.8;
        text_color.0 = text_color.0.with_alpha(t);
    }
}

/// Despawn expired emoji labels.
pub fn remove_completed_animations(
    mut commands: Commands,
    q: Query<(Entity, &EmojiLabel)>,
) {
    for (entity, label) in &q {
        if label.timer <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
