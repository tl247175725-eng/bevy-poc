//! 四通道感知系统 — 各自独立，直推需求急迫度
//!
//! 依据：PLOS Biology 2019 贝叶斯因果推理
//!       Game Creator 2 Awareness
//!       design_observer-architecture_deepseek-v4.md

use crate::need_match::data::NeedKind;
use crate::spatial_index::EntityId;
use crate::tags::TagBits;

const NONE_ID: EntityId = EntityId(0);

// ===== 感知结果 =====

/// 感知结果——不返回"觉察度"，直接返回对需求急迫度的贡献
#[derive(Debug, Clone)]
pub struct PerceptionResult {
    pub detected_entity: EntityId,
    /// (需求类型, 急迫度增量)
    pub need_contributions: Vec<(NeedKind, f32)>,
}

// ===== 视觉感知 =====

/// 视觉感知
///
/// 检查：在范围内？ 有视觉能力？ 有遮挡？
pub fn perceive_vision(
    _observer_id: EntityId,
    observer_pos: (u8, u8),
    observer_vision_range: u32,
    observer_tags: &TagBits, // 是否有 keen_eyed 等
    target_id: EntityId,
    target_pos: (u8, u8),
    target_tags: &TagBits,   // 是否有 hidden/camouflage 等
    target_size: u8,
    terrain_blocked: bool,   // 中间是否有遮挡
    light_level: f32,        // 环境光照 (0-1)
    distance: u32,
) -> Option<PerceptionResult> {
    let _ = (observer_pos, target_pos);

    // 1. 距离检查
    if distance > observer_vision_range {
        return None;
    }

    // 2. 遮挡
    if terrain_blocked {
        return None; // TODO: 后续支持 partial visibility
    }

    // 3. 被锁定——隐藏/伪装
    let hidden = target_tags.has(crate::tags::tag::DEF_HIDE.bit)
        || target_tags.has(crate::tags::tag::DEF_CAMO.bit);
    if hidden {
        return None; // TODO: keen_eyed 可部分穿透
    }

    // 4. 计算紧迫度贡献
    //   视觉置信度 = 距离衰减 × 光照修正 × 目标大小
    let distance_factor = 1.0 - (distance as f32 / observer_vision_range as f32);
    let size_factor = (target_size as f32 / 5.0).min(1.0);
    let confidence = distance_factor * light_level * size_factor;

    // 5. 直推需求
    //   如果目标是 predator → need:safety.urgency += confidence
    //   如果目标是 food → need:nutrition.urgency += confidence
    let mut contributions = Vec::new();
    if target_tags.has(crate::tags::tag::DIET_CARNIVORE.bit) {
        contributions.push((NeedKind::Safety, confidence * 0.8));
    }
    if target_tags.has(crate::tags::tag::DIET_HERBIVORE.bit)
        || target_tags.has(crate::tags::tag::DIET_OMNIVORE.bit)
    {
        contributions.push((NeedKind::Nutrition, confidence * 0.5));
    }
    // 对新奇物体 → curiosity
    contributions.push((NeedKind::Curiosity, confidence * 0.1));

    Some(PerceptionResult {
        detected_entity: target_id,
        need_contributions: contributions,
    })
}

// ===== 听觉感知 =====

/// 听觉感知
///
/// 不需要视线——球面范围
pub fn perceive_hearing(
    _observer_id: EntityId,
    observer_pos: (u8, u8),
    observer_hearing_range: u32,
    sound_source_pos: (u8, u8),
    sound_intensity: f32,   // 声音源强度 (0-1)
    sound_type: &str,       // "footstep", "growl", "scream", "shatter"
    ambient_noise: f32,     // 环境噪音 (0-1)
    distance: u32,
) -> Option<PerceptionResult> {
    let _ = observer_pos;
    let _ = sound_source_pos;

    if distance > observer_hearing_range {
        return None;
    }

    // 声音衰减 = 距离 × 环境噪音
    let distance_factor = 1.0 - (distance as f32 / observer_hearing_range as f32);
    let effective_volume = sound_intensity * distance_factor * (1.0 - ambient_noise);

    if effective_volume < 0.1 {
        return None;
    }

    let mut contributions = Vec::new();
    match sound_type {
        "growl" | "roar" => {
            contributions.push((NeedKind::Safety, effective_volume * 0.9));
        }
        "scream" | "cry" => {
            contributions.push((NeedKind::Safety, effective_volume * 0.7));
            contributions.push((NeedKind::Curiosity, effective_volume * 0.5));
        }
        "footstep" => {
            contributions.push((NeedKind::Curiosity, effective_volume * 0.2));
        }
        _ => {
            contributions.push((NeedKind::Curiosity, effective_volume * 0.1));
        }
    }

    Some(PerceptionResult {
        detected_entity: NONE_ID, // 声音可能不对应具体 entity
        need_contributions: contributions,
    })
}

// ===== 嗅觉感知 =====

/// 嗅觉感知
pub fn perceive_smell(
    observer_smell_range: u32,
    odor_intensity: f32,
    odor_type: &str,            // "blood", "food", "predator_scent"
    wind_direction: (f32, f32),  // 风向向量
    observer_to_source: (f32, f32),
    distance: u32,
) -> Option<PerceptionResult> {
    if distance > observer_smell_range {
        return None;
    }

    // 风向修正——顺风远、逆风近
    let dot = wind_direction.0 * observer_to_source.0 + wind_direction.1 * observer_to_source.1;
    let wind_factor = if dot > 0.0 {
        1.0 + dot
    } else {
        (1.0 + dot).max(0.1)
    };

    let effective =
        odor_intensity * wind_factor * (1.0 - distance as f32 / observer_smell_range as f32);
    if effective < 0.05 {
        return None;
    }

    let mut contributions = Vec::new();
    match odor_type {
        "blood" => {
            contributions.push((NeedKind::Safety, effective * 0.6));
            contributions.push((NeedKind::Curiosity, effective * 0.4));
        }
        "food" => {
            contributions.push((NeedKind::Nutrition, effective * 0.5));
        }
        "predator_scent" => {
            contributions.push((NeedKind::Safety, effective * 1.0));
        }
        _ => {}
    }

    Some(PerceptionResult {
        detected_entity: NONE_ID,
        need_contributions: contributions,
    })
}

// ===== 触觉感知 =====

/// 触觉——接触即触发
pub fn perceive_touch(
    touch_event: &str, // "bump", "attack", "grab"
) -> Option<PerceptionResult> {
    let contributions = match touch_event {
        "attack" => vec![(NeedKind::Safety, 1.0)],
        "bump" => vec![(NeedKind::Curiosity, 0.3)],
        _ => vec![],
    };
    if contributions.is_empty() {
        return None;
    }
    Some(PerceptionResult {
        detected_entity: NONE_ID,
        need_contributions: contributions,
    })
}

// ===== 测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vision_out_of_range_returns_none() {
        let tags = TagBits::new();
        let result = perceive_vision(
            EntityId(1),
            (5, 5),
            10, // vision_range
            &tags,
            EntityId(2),
            (15, 5),
            &tags,
            3,
            false,
            1.0,
            15, // distance > vision_range
        );
        assert!(result.is_none(), "超出视觉范围应返回 None");
    }

    #[test]
    fn vision_target_hidden_returns_none() {
        let mut target_tags = TagBits::new();
        target_tags.set(crate::tags::tag::DEF_HIDE.bit); // HIDDEN

        let observer_tags = TagBits::new();
        let result = perceive_vision(
            EntityId(1),
            (5, 5),
            10,
            &observer_tags,
            EntityId(2),
            (8, 5),
            &target_tags,
            3,
            false,
            1.0,
            3, // distance within range
        );
        assert!(
            result.is_none(),
            "目标隐藏时应返回 None (HIDDEN bit set)"
        );
    }

    #[test]
    fn hearing_drowned_by_noise_returns_none() {
        let _observer_tags = TagBits::new();
        let result = perceive_hearing(
            EntityId(1),
            (5, 5),
            20,
            (10, 5),
            0.3,       // low intensity
            "footstep",
            0.95,      // high ambient noise
            5,         // distance within range
        );
        assert!(result.is_none(), "高噪音应掩盖低强度声音");
    }

    #[test]
    fn smell_against_wind_attenuates() {
        // 逆风：风向指向 observer，observer_to_source 指向 source
        let wind = (1.0, 0.0);          // 风向右（东）
        let to_source = (-1.0, 0.0);     // 源在左边（西）→ 逆风
        let result = perceive_smell(
            20,
            0.5,
            "food",
            wind,
            to_source,
            5,
        );
        // 逆风时 wind_factor = (1 + (-1)).max(0.1) = 0.1
        // effective = 0.5 * 0.1 * (1 - 5/20) = 0.5 * 0.1 * 0.75 = 0.0375 < 0.05
        assert!(
            result.is_none(),
            "逆风应导致嗅觉衰减到阈值以下"
        );
    }

    #[test]
    fn vision_returns_curiosity_for_novel_target() {
        let tags = TagBits::new();
        let result = perceive_vision(
            EntityId(1),
            (5, 5),
            10,
            &tags,
            EntityId(2),
            (8, 5),
            &tags,
            3,
            false,
            1.0,
            3,
        );
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.detected_entity, EntityId(2));
        // 好奇心贡献始终存在
        let has_curiosity = r
            .need_contributions
            .iter()
            .any(|(k, _)| *k == NeedKind::Curiosity);
        assert!(has_curiosity, "新物体应触发好奇心");
    }
}
