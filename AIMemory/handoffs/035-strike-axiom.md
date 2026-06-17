# Handoff 035 — Strike 公理：同构物理替代 hp -= 1

## 三柱强制检查

| 柱子 | 用哪个 |
|---|---|
| 标签 | 攻击者：`body_size:*`(质量基础) + `body_plan:*`(有效质量比例) + `capability:bite/claw/constrict/tool_use`(攻击方式→物理参数)。目标：`defense:armor`(目标硬度) |
| 元数值 | `strike_force()`(新增, meta_values.rs)——映射 capability→{速度, 接触面, 硬度} + `body_plan→有效质量比例`。打击型用 ½mv²/A×硬度比，压力型用 CSA×300kPa |
| 元动作 | `MetaAction::Strike { target }`——已有，替换 `hp -= 1` |
| 公理 | `strike_force()` 就是物理公理——纯函数，从标签参数算伤害 |

## 架构计划

**改什么：** `src/meta_values.rs` + `src/systems/main_tick.rs` + 新建 `src/axioms/strike.rs`（3 文件）

**为什么：** Strike 当前 `hp -= 1`，无论老虎咬鹿还是鹿踢虎伤害一样。同构模型要求：身体有什么标签 → 物理参数是什么 → 公式算出伤害。

### 设计决策（策划已确认）

- **所有动物都能攻击**。没有 capability 标签 = 走默认参数（冲撞/踢），公式照样算，结果小就是小
- **两条公式**：打击型（咬/爪/踢/角顶）用 ½mv²/A×硬度比，压力型（绞杀/握）用肌肉截面积×300kPa。capability 标签决定走哪条
- **有效质量比例从 body_plan 推导**，不是独立标签。同一 capability 在不同 body_plan 上参数不同（如 bite 在四足=8%体重，在鱼形=3%）
- **不做武器投资标签**。同一体型的鳄鱼比老虎咬力大，是因为鳄鱼的 body_plan 决定了头部占体重比例更高——这是形态差异，自然传导到参数
- **不做年龄/性别/人格修正**——后续 handoff
- **不做防御**——defense:armor 后续 handoff

### 改动 1：新建 `src/axioms/strike.rs`

```rust
//! Strike 公理——同构物理伤害计算
//! 标签驱动，零硬编码。身体有什么 → 参数是什么 → 公式算伤害。

use crate::tags::TagBits;

/// 攻击方式 → 物理参数
pub struct StrikeParams {
    pub velocity: f32,       // m/s
    pub contact_area: f32,   // m²
    pub hardness: f32,       // 摩斯硬度
    pub is_pressure: bool,   // true=压力型(CSA×300kPa), false=打击型(½mv²/A)
    pub pressure_csa: f32,   // cm², 仅压力型用
}

/// 从 capability 标签映射到物理参数
pub fn capability_params(tags: &TagBits) -> StrikeParams {
    // 压力型
    if tags.has(tag::CAPABILITY_CONSTRICT) {
        return StrikeParams { velocity: 0.0, contact_area: 0.0, hardness: 1.0,
            is_pressure: true, pressure_csa: 0.0 }; // CSA 从 body_plan 推导
    }
    // 打击型——咬
    if tags.has(tag::CAPABILITY_BITE) {
        return StrikeParams { velocity: 5.0, contact_area: 0.000001, hardness: 5.0,
            is_pressure: false, pressure_csa: 0.0 };
    }
    // 打击型——爪
    if tags.has(tag::CAPABILITY_CLAW) {
        return StrikeParams { velocity: 8.0, contact_area: 0.000003, hardness: 2.5,
            is_pressure: false, pressure_csa: 0.0 };
    }
    // 工具使用 → 读取装备卡标签（后续 handoff）
    if tags.has(tag::CAPABILITY_TOOL_USE) {
        return StrikeParams { velocity: 10.0, contact_area: 0.0000001, hardness: 8.0,
            is_pressure: false, pressure_csa: 0.0 };
    }
    // 默认：冲撞/踢
    StrikeParams { velocity: 5.0, contact_area: 0.01, hardness: 1.0,
        is_pressure: false, pressure_csa: 0.0 }
}

/// 从 body_plan 推导：该攻击方式的有效质量占体重的比例
pub fn effective_mass_ratio(tags: &TagBits, for_bite: bool) -> f32 {
    if tags.has(tag::PLAN_QUADRUPED) {
        if for_bite { return 0.08; }
        // 默认冲撞用全身，踢用 0.10
        return 0.10;
    }
    if tags.has(tag::PLAN_BIPED) {
        if for_bite { return 0.06; }
        return 0.15; // 双足踢更强
    }
    if tags.has(tag::PLAN_SERPENTINE) {
        return 0.40; // 绞杀用全身肌肉
    }
    if tags.has(tag::PLAN_FISH) {
        if for_bite { return 0.03; }
        return 0.30; // 尾击
    }
    0.10 // 默认
}

/// 同构 Strike 伤害计算
pub fn strike_force(
    attacker_tags: &TagBits,
    attacker_mass_kg: f32,
    _defender_tags: &TagBits,  // 后续 handoff 查 defense:armor
) -> f32 {
    let params = capability_params(attacker_tags);
    let ratio = effective_mass_ratio(attacker_tags, attacker_tags.has(tag::CAPABILITY_BITE));
    let effective_mass = attacker_mass_kg * ratio;

    if params.is_pressure {
        // 压力型：肌肉截面积 × 300 kPa
        // 截面积从 body_plan + body_size 推导（简化：CSA ≈ 有效质量/肌肉密度/典型纤维长度）
        let csa = effective_mass / 1060.0 / 0.15 * 10000.0; // m³→cm²
        csa * 30.0  // N/cm² → N
    } else {
        // 打击型：½ × m × v² / A × 硬度比
        let hardness_ratio = params.hardness / 1.0; // 目标默认硬度=1.0，后续 defense:armor 提高
        0.5 * effective_mass * params.velocity.powi(2) / params.contact_area * hardness_ratio
    }
}
```

### 改动 2：重写 `apply_meta_action` 的 Strike 分支

```rust
MetaAction::Strike { target } => {
    let Some(attacker_entity) = world.entities.get(&entity_id) else { return };
    let Some(target_entity) = world.entities.get(&target) else { return };
    
    let Some(attacker_def) = world.card_defs.get(&attacker_entity.type_name) else { return };
    let attacker_tags = &attacker_def.tag_bits;
    let mass = crate::axioms::consume::estimate_mass_from_tags(attacker_tags);
    
    let force = crate::axioms::strike::strike_force(attacker_tags, mass, &TagBits::new());
    
    // 伤害 = 力 / 1000 (N→伤害单位, 经验换算)
    let damage = (force / 1000.0).ceil() as i32;
    
    if let Some(target_e) = world.entities.get_mut(&target) {
        target_e.hp = target_e.hp.saturating_sub(damage.max(1));
    }
}
```

### 改动 3：`src/meta_values.rs` 

添加 strike 相关常量（capability 参数、body_plan 比例移到 axioms/strike.rs 因为它们和标签查询耦合，不是纯数值常量）。同时标记旧的 `STRIKE_BASE_DAMAGE` 为 deprecated。

## 架构反馈

1. **标签驱动，零硬编码**：加新攻击方式 = 在 `capability_params()` 加一行参数 + 在 `tags.ron` 加标签。不改公式
2. **body_size × body_plan × capability 三层联动**：不同体型的同种攻击方式自动出不同伤害，不同身体结构的动物攻击力差异自然涌现
3. **预计算缓存后续 handoff**：cached_attack_power 不在本次范围——当前每 tick 重算标签查询约需几微秒，对 100 实体的生态不构成瓶颈
4. **defense:armor 后续 handoff**：当前目标硬度固定 1.0

## 本体变更

- [ ] `AIMemory/ontology.md` strike 公理状态从 ❌ 更新为 ✅
- [ ] `capability_params` 和 `effective_mass_ratio` 映射表加入元数值节
- [ ] cross_references 补充 capability → strike_force 关联

## 智能验收

- [ ] `cargo check` 零错误
- [ ] `cargo test` 全 PASS
- [ ] 新增单元测试：老虎咬鹿 > 鹿踢虎（体型差体现）
- [ ] 新增单元测试：同体型鳄鱼咬力 > 同体型老虎咬力（body_plan 差异体现）
- [ ] 新增单元测试：蟒蛇绞杀走压力公式（非打击公式）
- [ ] `apply_meta_action` 的 Strike 分支不再包含 `hp -= 1`
