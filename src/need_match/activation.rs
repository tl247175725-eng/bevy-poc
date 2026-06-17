//! 需求激活逻辑 — tick 衰减 + 安全阻断
//!
//! 依据 design-philosophy-v5.md §3.2、§6.5

use super::data::{NeedKind, NeedState};

/// 需求激活阈值——紧迫度超过此值视为"激活"
pub const URGENCY_ACTIVATION_THRESHOLD: f32 = crate::meta_values::URGENCY_ACTIVATION_THRESHOLD;

/// 安全阻断阈值——安全紧迫度超过此值阻断所有非安全需求
pub const SAFETY_BLOCK_THRESHOLD: f32 = crate::meta_values::SAFETY_BLOCK_THRESHOLD;

/// 每个 tick 衰减需求，计算紧迫度。
///
/// urgency = sigmoid(raw * 5.0), raw = (current - baseline) / baseline
pub fn tick_need(need: &mut NeedState, delta: f32) {
    // 需求值向 1.0（完全匮乏）衰减
    need.current = (need.current + need.decay_rate * delta).min(1.0);
    // sigmoid 紧迫度
    let raw = (need.current - need.baseline) / need.baseline;
    need.urgency = 1.0 / (1.0 + (-raw * 5.0).exp());
}

/// 安全阻断逻辑：如果安全需求紧迫度超过阈值，阻断所有非安全需求。
pub fn apply_safety_block(needs: &mut [NeedState]) {
    let safety_urgent = needs
        .iter()
        .any(|n| n.kind == NeedKind::Safety && n.urgency > SAFETY_BLOCK_THRESHOLD);
    if safety_urgent {
        for need in needs.iter_mut() {
            if need.kind != NeedKind::Safety {
                need.blocked = true;
            }
        }
    } else {
        for need in needs.iter_mut() {
            need.blocked = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_need(kind: NeedKind, current: f32, baseline: f32, decay_rate: f32) -> NeedState {
        NeedState {
            kind,
            current,
            baseline,
            urgency: 0.0,
            blocked: false,
            decay_rate,
        }
    }

    #[test]
    fn sigmoid_low_when_satisfied() {
        // current=0, baseline=1: raw=-1, urgency = 1/(1+e^5) ≈ 0.007
        let mut need = make_need(NeedKind::Nutrition, 0.0, 1.0, 0.5);
        tick_need(&mut need, 0.0);
        assert!(need.urgency < 0.01);
    }

    #[test]
    fn sigmoid_mid_at_threshold() {
        // current=0.9, baseline=0.6 → raw=(0.9-0.6)/0.6=0.5 → urgency≈0.924
        let mut need = make_need(NeedKind::Nutrition, 0.9, 0.6, 0.5);
        tick_need(&mut need, 0.0); // delta=0 → no decay, just compute urgency
        assert!(need.urgency > 0.9);
        assert!(need.urgency < 0.95);
    }

    #[test]
    fn sigmoid_high_at_deficit() {
        // raw = 1.0: urgency = 1/(1+e^(-5)) ≈ 0.993
        let mut need = make_need(NeedKind::Nutrition, 1.0, 0.5, 0.5);
        tick_need(&mut need, 0.0);
        assert!(need.urgency > 0.98);
    }

    #[test]
    fn tick_decays_current_toward_one() {
        let mut need = make_need(NeedKind::Hydration, 0.0, 1.0, 0.7);
        tick_need(&mut need, 0.5);
        // current: 0.0 + 0.7*0.5 = 0.35
        assert!((need.current - 0.35).abs() < 0.001);
    }

    #[test]
    fn tick_clamps_current_at_one() {
        let mut need = make_need(NeedKind::Hydration, 0.9, 1.0, 0.7);
        tick_need(&mut need, 1.0);
        assert_eq!(need.current, 1.0);
    }

    #[test]
    fn safety_block_blocks_non_safety() {
        let mut needs = vec![
            make_need(NeedKind::Safety, 1.0, 0.5, 0.2),
            make_need(NeedKind::Nutrition, 0.0, 1.0, 0.5),
            make_need(NeedKind::Rest, 0.0, 1.0, 0.3),
        ];
        // Manually set safety urgency above threshold
        needs[0].urgency = 0.8;
        apply_safety_block(&mut needs);
        assert!(!needs[0].blocked); // Safety itself not blocked
        assert!(needs[1].blocked);  // Nutrition blocked
        assert!(needs[2].blocked);  // Rest blocked
    }

    #[test]
    fn safety_block_releases_when_safe() {
        let mut needs = vec![
            make_need(NeedKind::Safety, 0.0, 0.5, 0.2),
            make_need(NeedKind::Nutrition, 0.0, 1.0, 0.5),
        ];
        needs[0].urgency = 0.3; // below threshold
        needs[1].blocked = true;
        apply_safety_block(&mut needs);
        assert!(!needs[1].blocked); // Released
    }

    #[test]
    fn decay_rate_differs_by_kind() {
        let hydration = super::super::data::default_decay_rate(&NeedKind::Hydration);
        let social = super::super::data::default_decay_rate(&NeedKind::Social);
        let expected_hydration = crate::meta_values::nutrition_decay_per_tick(1.5);
        let expected_social = crate::meta_values::SOCIAL_DECAY_RATE;
        assert!((hydration - expected_hydration).abs() < 1e-6);
        assert!((social - expected_social).abs() < 1e-6);
    }
}