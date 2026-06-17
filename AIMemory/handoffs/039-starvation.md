# Handoff 039 — 饥饿致死：同构热力学公式

## 三柱强制检查

| 柱子 | 用哪个 |
|---|---|
| 标签 | `body_size:*`(质量) + `thermo:endotherm/ectotherm`(β和B₀) + `metab:torpor`(休眠系数) |
| 元数值 | `fasting_endurance_ticks()`(新增) + `FASTING_COEFFICIENT=100` + `estimate_mass_from_tags()` |
| 元动作 | 不涉及——饥饿致死是被动过程，不是元动作 |
| 公理 | 热力学第一定律：能量储备 ÷ 代谢消耗 = 存活时间 |

## 架构计划

**改什么：** `src/meta_values.rs` + `src/systems/main_tick.rs`（2 文件）

**为什么：** 动物不吃不死 → 种群永不减少 → 生态不闭合。饥饿致死是生态循环的第一个闭合条件。

### 设计决策（策划已确认）

- **公式**：`fasting_days = 100 × M^(1-β) / B₀ × torpor_mult`
- **FASTING_COEFFICIENT = 100**（同构匹配真实数据，验证过：鼠4天、猫14天、人29天、鹿30天、鳄鱼232天）
- **β**：恒温 0.75，变温 0.84（从 `thermo` 标签推）
- **B₀**：恒温 10.0，变温 1.0（从 `thermo` 标签推）
- **torpor_mult**：有 `metab:torpor` = 3.0，无 = 1.0
- **死亡条件**：Nutrition need.current 持续 >= 1.0 超过 fasting_endurance ticks → is_corpse = true
- **吃了就重置**：need.current 降到 < 1.0 → starve_ticks 归零

### 改动 1：`src/meta_values.rs`

```rust
/// 饥饿致死系数（同构：匹配真实数据）
pub const FASTING_COEFFICIENT: f32 = 100.0;

/// 恒温动物代谢缩放指数
pub const METABOLIC_EXPONENT_ENDOTHERM: f32 = 0.75;
/// 变温动物代谢缩放指数
pub const METABOLIC_EXPONENT_ECTOTHERM: f32 = 0.84;

/// 恒温动物基础代谢常数
pub const BMR_CONSTANT_ENDOTHERM: f32 = 10.0;
/// 变温动物基础代谢常数
pub const BMR_CONSTANT_ECTOTHERM: f32 = 1.0;

/// 休眠耐饿倍数
pub const TORPOR_FASTING_MULTIPLIER: f32 = 3.0;

/// 从标签计算饥饿致死天数
pub fn fasting_endurance_days(mass_kg: f32, is_ectotherm: bool, can_torpor: bool) -> f32 {
    let beta = if is_ectotherm { METABOLIC_EXPONENT_ECTOTHERM } else { METABOLIC_EXPONENT_ENDOTHERM };
    let b0 = if is_ectotherm { BMR_CONSTANT_ECTOTHERM } else { BMR_CONSTANT_ENDOTHERM };
    let torpor = if can_torpor { TORPOR_FASTING_MULTIPLIER } else { 1.0 };
    FASTING_COEFFICIENT * mass_kg.powf(1.0 - beta) / b0 * torpor
}

/// 从标签计算饥饿致死 tick 数
pub fn fasting_endurance_ticks(mass_kg: f32, is_ectotherm: bool, can_torpor: bool) -> u64 {
    (fasting_endurance_days(mass_kg, is_ectotherm, can_torpor) * TICKS_PER_DAY as f32) as u64
}
```

### 改动 2：`src/systems/main_tick.rs`

在 Phase 2（需求衰减）之后、Phase 3 之前，加饥饿致死检查：

```rust
// ===== Phase 2.5: 饥饿致死检查 =====
let starvation_deaths: Vec<EntityId> = world.entities.iter()
    .filter_map(|(&id, entity)| {
        // 只检查有 Nutrition 需求的活实体
        if entity.is_corpse { return None; }
        let nutrition = entity.needs.iter()
            .find(|n| matches!(n.kind, NeedKind::Nutrition))?;
        if nutrition.current < 1.0 { return None; } // 还没饿到极限
        
        // 查标签
        let def = world.card_defs.get(&entity.type_name)?;
        let tags = &def.tag_bits;
        let mass = crate::axioms::consume::estimate_mass_from_tags(tags);
        let is_ecto = tags.has(tag::THERMO_ECTOTHERM.bit);
        let can_torpor = tags.has(tag::METAB_TORPOR.bit);
        let limit = crate::meta_values::fasting_endurance_ticks(mass, is_ecto, can_torpor);
        
        if entity.starve_days as u64 >= limit {
            Some(id)
        } else {
            None
        }
    })
    .collect();

for id in starvation_deaths {
    if let Some(entity) = world.entities.get_mut(&id) {
        entity.is_corpse = true;
        entity.hp = 0;
    }
}

// starve_days 递增/重置逻辑
for entity in world.entities.values_mut() {
    if entity.is_corpse { continue; }
    let starving = entity.needs.iter()
        .any(|n| matches!(n.kind, NeedKind::Nutrition) && n.current >= 1.0);
    if starving {
        entity.starve_days += 1;
    } else {
        entity.starve_days = 0;
    }
}
```

注意：`starve_days` 字段已存在于 Entity 上（`pub starve_days: i32`），直接复用。

需要检查 `tag::THERMO_ECTOTHERM` 和 `tag::METAB_TORPOR` 常量名是否正确。

## 本体变更

- [ ] ontology.md 公理节：加 starvation_check 状态 ✅
- [ ] cross_references：body_size + thermo + metab:torpor → 饥饿致死

## 架构反馈

1. **第一个生态闭合条件**：动物不吃 → 饿死 → 种群减少 → 捕食压力变化
2. **公式天然可扩展**：将来加温度（多乘 Q10 项）、加体况（多乘 body_fat 项），不改结构
3. **starve_days 复用**：Entity 上已有该字段，不需要加新字段

## 智能验收

- [ ] `cargo check` 零错误
- [ ] `cargo test` 全 PASS
- [ ] 新增测试：`fasting_endurance_days(80.0, false, false)` ≈ 30（鹿）
- [ ] 新增测试：`fasting_endurance_days(3.0, true, false)` ≈ 119（鲤鱼）
- [ ] 新增测试：`fasting_endurance_days(200.0, false, true)` ≈ 113（熊+冬眠）
- [ ] 集成测试意图：跑 N tick 后有实体 is_corpse=true（可选，如性能允许）
