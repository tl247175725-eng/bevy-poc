# Handoff 031 — Consume 公理贯通：三根柱子首次连接

## 架构计划

**改什么：** 新建 `src/axioms/consume.rs` + 改 `src/axioms/mod.rs`（1 行）+ 改 `src/systems/main_tick.rs` Consume 分支（3 文件）

**为什么：** Consume 是生态系统的根基行为。当前实现完全绕过三根柱子——幽灵字符串查表 + 布尔标记，不经过元数值/元动作/公理。

### 三柱对照（已通过强制检查 ✓）

| 柱子 | 用哪个 |
|---|---|
| 标签 | 食用者：`DIET_HERBIVORE/CARNIVORE/OMNIVORE/PISCIVORE/INSECTIVORE/FRUGIVORE/SCAVENGER/DETRITIVORE.bit`（tags.rs 常量，TagBits::has 查询）。目标：`PLAN_PLANT/QUADRUPED/BIPED/SERPENTINE/AVIAN/FISH/INSECTOID.bit` + `NUTRITION_AUTOTROPH.bit` + `Entity.is_corpse`（运行时字段） |
| 元数值 | `baseline_energy(mass, metabolism_rate)` — meta_values.rs:116。metabolism_rate 从 METAB_HIGH/MEDIUM/LOW 标签映射（1.5/1.0/0.5） |
| 元动作 | `MetaAction::Consume { target }` — meta_actions.rs:22 |
| 公理 | 新建 `can_digest()` 在 `src/axioms/consume.rs` — 纯函数，输入 TagBits + is_corpse + TagRegistry，返回 bool |

### 改动 1：新建 `src/axioms/consume.rs`

```rust
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
```

**注意：** `TagRegistry::new_empty()` 如果不存在，在测试中传 `&TagRegistry::default()` 或创建一个空实例。`_registry` 参数在 can_digest 中标记为未使用（当前逻辑用 TagBits::has 直接查 bit，不需要 registry 的 descendants。保留该参数为未来 has_descendant_of 扩展预留）。

### 改动 2：`src/axioms/mod.rs` 加 1 行

在 `pub mod laws;` 后加：
```rust
pub mod consume;
```

在 `pub use laws::{` 块后加：
```rust
pub use consume::can_digest;
```

### 改动 3：`src/systems/main_tick.rs` 重写 Consume 分支

找到 `apply_meta_action()` 中的：
```rust
MetaAction::Consume { target } => {
    // 标记目标被消耗
    if let Some(target_entity) = world.entities.get_mut(&target) {
        target_entity.consumed = true;
    }
}
```

替换为：
```rust
MetaAction::Consume { target } => {
    use crate::axioms::consume;
    use crate::meta_values::baseline_energy;
    use crate::tags::tag;
    use crate::need_match::data::NeedKind;

    let Some(actor_entity) = world.entities.get(&entity_id) else { return };
    let Some(target_entity) = world.entities.get(&target) else { return };

    // 1. 获取 TagBits
    let Some(actor_def) = world.card_defs.get(&actor_entity.type_name) else { return };
    let Some(target_def) = world.card_defs.get(&target_entity.type_name) else { return };
    let actor_tags = &actor_def.tag_bits;
    let target_tags = &target_def.tag_bits;

    // 2. 公理检查：可消化？
    let registry = match crate::world_rules::TAG_REGISTRY.get() {
        Some(r) => r,
        None => return,
    };
    if !consume::can_digest(actor_tags, target_tags, target_entity.is_corpse, registry) {
        return; // 不可消化，静默跳过
    }

    // 3. 元数值：计算能量转移
    let target_mass = consume::estimate_mass_from_tags(target_tags);
    let actor_metab = if actor_tags.has(tag::METAB_HIGH.bit) { 1.5 }
        else if actor_tags.has(tag::METAB_LOW.bit) { 0.5 }
        else { 1.0 };
    let energy = baseline_energy(target_mass, actor_metab);

    // 4. 更新食用者 Nutrition need（需要重新借用 entity）
    drop(actor_entity);
    drop(target_entity);
    if let Some(actor) = world.entities.get_mut(&entity_id) {
        for need in &mut actor.needs {
            if need.kind == NeedKind::Nutrition {
                need.current = (need.current - energy * 0.5).max(0.0);
            }
        }
    }

    // 5. 标记目标被消耗
    if let Some(target_e) = world.entities.get_mut(&target) {
        target_e.consumed = true;
    }
}
```

## 架构反馈

1. **三根柱子首次在运行时贯通**：Consume 路径 = `标签(TagBits::has 判断消化性) → 元数值(baseline_energy + estimate_mass) → 元动作(MetaAction::Consume) → 公理(can_digest)`
2. **消解了 6 个幽灵字符串中的 4 个**：公理层用 TagBits 直接判断，不再依赖卡牌上是否存在 "fish"/"insectoid"/"fruit"/"corpse" 字符串标签
3. **diet 标签从行为驱动器降级为消化能力标识**：食性不再映射到"吃什么"，而是直接定义"能消化什么物质类型"
4. **保留了 modify 而非 replace 策略**：`init_animal_knowledge()` 的搜索过滤逻辑暂不改动（下个 handoff），避免一次改太多文件
5. **测试用 `TagRegistry::default_registry()`**：consuming 测试用的 `can_digest` 只用 `TagBits::has()`（不依赖 descendants），所以传任意有效 TagRegistry 即可

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS（包含 consume.rs 中 8 个新测试）
- 新测试覆盖：食草→植物✓、食草→肉✗、食肉→肉✓、食肉→植物✗、杂食→两者✓、食腐→尸体✓、食鱼→鱼✓、食肉→尸体✓
- `apply_meta_action` 的 Consume 分支不再有直接 `target.consumed = true`（必须先过 can_digest）
- `can_digest()` 函数内无幽灵字符串——所有标签引用来自 `tag::CONSTANT.bit`

## 不改的（后续 handoff）

- `init_animal_knowledge()` 的幽灵字符串修复（032）
- `MaterialProperties.tags` → TagBits 统一（032 或 033）
- `state:dead` 标签位修复
