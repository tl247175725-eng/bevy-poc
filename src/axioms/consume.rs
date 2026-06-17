//! Consume 公理——判断食用者能否消化目标。
//!
//! 标签驱动，零硬编码映射表。
//! 食用者的 diet 标签 × 目标的物质组成标签 → 可消化性。

use crate::tags::{TagBits, TagRegistry, tag};

/// 判断食用者能否消化目标。
/// - actor_tags: 食用者的 TagBits（检查 diet:* 标签）
/// - target_tags: 目标的 TagBits（检查 body_plan:* / nutrition:* 标签）
/// - target_is_corpse: 目标是否已死亡（决定食腐/食碎屑）
pub fn can_digest(
    actor_tags: &TagBits,
    target_tags: &TagBits,
    target_is_corpse: bool,
    _registry: &TagRegistry,
) -> bool {
    // ── 目标物质组成 ──
    let is_plant = target_tags.has(tag::NUTRITION_AUTOTROPH.bit)
        || target_tags.has(tag::PLAN_PLANT.bit);

    let is_animal_tissue = target_tags.has(tag::PLAN_QUADRUPED.bit)
        || target_tags.has(tag::PLAN_BIPED.bit)
        || target_tags.has(tag::PLAN_SERPENTINE.bit)
        || target_tags.has(tag::PLAN_AVIAN.bit)
        || target_tags.has(tag::PLAN_FISH.bit)
        || target_tags.has(tag::PLAN_INSECTOID.bit);

    let is_fish = target_tags.has(tag::PLAN_FISH.bit);
    let is_insect = target_tags.has(tag::PLAN_INSECTOID.bit);

    // ── 食用者消化能力 ──
    let can_eat_plant = actor_tags.has(tag::DIET_HERBIVORE.bit)
        || actor_tags.has(tag::DIET_OMNIVORE.bit);

    let can_eat_meat = actor_tags.has(tag::DIET_CARNIVORE.bit)
        || actor_tags.has(tag::DIET_OMNIVORE.bit);

    let can_eat_corpse = actor_tags.has(tag::DIET_SCAVENGER.bit)
        || actor_tags.has(tag::DIET_DETRITIVORE.bit)
        || actor_tags.has(tag::DIET_CARNIVORE.bit); // 食肉动物也接受尸体

    let can_eat_fish = actor_tags.has(tag::DIET_PISCIVORE.bit);
    let can_eat_insect = actor_tags.has(tag::DIET_INSECTIVORE.bit);
    let can_eat_fruit = actor_tags.has(tag::DIET_FRUGIVORE.bit);

    // ── 匹配（特化优先于通用） ──
    if target_is_corpse && can_eat_corpse { return true; }
    if is_insect && can_eat_insect { return true; }
    if is_fish && can_eat_fish { return true; }
    if is_plant && can_eat_plant { return true; }
    if is_plant && can_eat_fruit { return true; }
    if is_animal_tissue && can_eat_meat { return true; }

    false
}

/// 从 body_size 标签估算实体质量（kg）
/// 用于 baseline_energy 计算。
pub fn estimate_mass_from_tags(tags: &TagBits) -> f32 {
    // TODO: 将以下生物学参考值移入 meta_values.rs（MASS_TINY/MASS_SMALL/MASS_MEDIUM/MASS_LARGE/MASS_HUGE）
    // 需要设计确认后再锁定。
    if tags.has(tag::SIZE_TINY.bit) { return 3.0; }
    if tags.has(tag::SIZE_SMALL.bit) { return 15.0; }
    if tags.has(tag::SIZE_MEDIUM.bit) { return 80.0; }
    if tags.has(tag::SIZE_LARGE.bit) { return 500.0; }
    if tags.has(tag::SIZE_HUGE.bit) { return 3000.0; }
    20.0 // 默认小型
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags::TagBits;
    use crate::tags::tag;

    fn make_actor_tags(bits: &[u16]) -> TagBits {
        let mut t = TagBits::new();
        for &b in bits { t.set(b); }
        t
    }

    #[test]
    fn herbivore_can_digest_plant() {
        let actor = make_actor_tags(&[tag::DIET_HERBIVORE.bit]);
        let target = make_actor_tags(&[tag::NUTRITION_AUTOTROPH.bit]);
        let registry = TagRegistry::default_registry();
        assert!(can_digest(&actor, &target, false, &registry));
    }

    #[test]
    fn herbivore_cannot_digest_meat() {
        let actor = make_actor_tags(&[tag::DIET_HERBIVORE.bit]);
        let target = make_actor_tags(&[tag::PLAN_QUADRUPED.bit]);
        let registry = TagRegistry::default_registry();
        assert!(!can_digest(&actor, &target, false, &registry));
    }

    #[test]
    fn carnivore_can_digest_meat() {
        let actor = make_actor_tags(&[tag::DIET_CARNIVORE.bit]);
        let target = make_actor_tags(&[tag::PLAN_QUADRUPED.bit]);
        let registry = TagRegistry::default_registry();
        assert!(can_digest(&actor, &target, false, &registry));
    }

    #[test]
    fn carnivore_cannot_digest_plant() {
        let actor = make_actor_tags(&[tag::DIET_CARNIVORE.bit]);
        let target = make_actor_tags(&[tag::NUTRITION_AUTOTROPH.bit]);
        let registry = TagRegistry::default_registry();
        assert!(!can_digest(&actor, &target, false, &registry));
    }

    #[test]
    fn omnivore_can_digest_both() {
        let actor = make_actor_tags(&[tag::DIET_OMNIVORE.bit]);
        let plant = make_actor_tags(&[tag::NUTRITION_AUTOTROPH.bit]);
        let meat = make_actor_tags(&[tag::PLAN_QUADRUPED.bit]);
        let registry = TagRegistry::default_registry();
        assert!(can_digest(&actor, &plant, false, &registry));
        assert!(can_digest(&actor, &meat, false, &registry));
    }

    #[test]
    fn scavenger_can_digest_corpse() {
        let actor = make_actor_tags(&[tag::DIET_SCAVENGER.bit]);
        let target = make_actor_tags(&[tag::PLAN_QUADRUPED.bit]);
        let registry = TagRegistry::default_registry();
        assert!(can_digest(&actor, &target, true, &registry));
    }

    #[test]
    fn piscivore_can_digest_fish() {
        let actor = make_actor_tags(&[tag::DIET_PISCIVORE.bit]);
        let target = make_actor_tags(&[tag::PLAN_FISH.bit]);
        let registry = TagRegistry::default_registry();
        assert!(can_digest(&actor, &target, false, &registry));
    }

    #[test]
    fn carnivore_can_digest_corpse() {
        let actor = make_actor_tags(&[tag::DIET_CARNIVORE.bit]);
        let target = make_actor_tags(&[tag::PLAN_QUADRUPED.bit]);
        let registry = TagRegistry::default_registry();
        assert!(can_digest(&actor, &target, true, &registry));
    }

    #[test]
    fn estimate_mass_from_body_size() {
        let tiny = make_actor_tags(&[tag::SIZE_TINY.bit]);
        let medium = make_actor_tags(&[tag::SIZE_MEDIUM.bit]);
        let huge = make_actor_tags(&[tag::SIZE_HUGE.bit]);
        assert_eq!(estimate_mass_from_tags(&tiny), 3.0);
        assert_eq!(estimate_mass_from_tags(&medium), 80.0);
        assert_eq!(estimate_mass_from_tags(&huge), 3000.0);
    }
}
