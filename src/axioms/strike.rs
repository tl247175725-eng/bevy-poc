//! Strike 公理——同构物理伤害计算
//! 标签驱动，零硬编码。身体有什么 → 参数是什么 → 公式算伤害。

use crate::tags::{TagBits, tag};

// ── Strike-specific physics constants (not in meta_values.rs) ──

/// Bite velocity (m/s) — jaw closing speed
const BITE_VELOCITY: f32 = 5.0;
/// Bite contact area (m²) — tooth tip
const BITE_CONTACT_AREA: f32 = 0.000_001;
/// Tooth enamel hardness (Mohs)
const BITE_HARDNESS: f32 = 5.0;

/// Tool-use velocity (m/s)
const TOOL_VELOCITY: f32 = 10.0;
/// Tool-use contact area (m²) — sharp edge
const TOOL_CONTACT_AREA: f32 = 0.000_000_1;
/// Tool material hardness (Mohs)
const TOOL_HARDNESS: f32 = 8.0;

/// Default strike velocity (m/s) — ram/kick
const DEFAULT_VELOCITY: f32 = 5.0;
/// Default contact area (m²) — hoof/foot
const DEFAULT_CONTACT_AREA: f32 = 0.01;
/// Default hardness (Mohs) — keratin/bone/flesh
const DEFAULT_HARDNESS: f32 = 1.0;
const TARGET_DEFAULT_HARDNESS: f32 = 1.0;

/// Muscle pressure capacity (Pa) for constriction
const MUSCLE_PRESSURE_PA: f32 = 300_000.0;
/// Typical muscle fiber length (m)
const MUSCLE_FIBER_LENGTH_M: f32 = 0.15;
/// m² → cm² conversion factor
const M2_TO_CM2: f32 = 10_000.0;

/// Effective mass ratio for bite on quadruped
const RATIO_BITE_QUADRUPED: f32 = 0.08;
/// Effective mass ratio for non-bite (kick/ram) on quadruped
const RATIO_KICK_QUADRUPED: f32 = 0.10;
/// Effective mass ratio for bite on biped
const RATIO_BITE_BIPED: f32 = 0.06;
/// Effective mass ratio for non-bite on biped
const RATIO_KICK_BIPED: f32 = 0.15;
/// Effective mass ratio for serpentine (full body for constrict/slither)
const RATIO_SERPENTINE: f32 = 0.40;
/// Effective mass ratio for bite on fish
const RATIO_BITE_FISH: f32 = 0.03;
/// Effective mass ratio for non-bite (tail strike) on fish
const RATIO_TAIL_FISH: f32 = 0.30;
/// Default effective mass ratio
const RATIO_DEFAULT: f32 = 0.10;

// ── Public types ──

/// 攻击方式 → 物理参数
pub struct StrikeParams {
    pub velocity: f32,       // m/s
    pub contact_area: f32,   // m²
    pub hardness: f32,       // 摩斯硬度
    pub is_pressure: bool,   // true=压力型(CSA×300kPa), false=打击型(½mv²/A)
    pub pressure_csa: f32,   // cm², 仅压力型用
}

// ── Public functions ──

/// 从 capability 标签映射到物理参数。
/// 只处理 BITE/CONSTRICT/TOOL_USE，其他走默认（冲撞/踢）。
pub fn capability_params(tags: &TagBits) -> StrikeParams {
    // 压力型——绞杀
    if tags.has(tag::CAP_CONSTRICT.bit) {
        return StrikeParams {
            velocity: 0.0,
            contact_area: 0.0,
            hardness: 1.0,
            is_pressure: true,
            pressure_csa: 0.0, // CSA 从 body_plan 推导
        };
    }
    // 打击型——咬
    if tags.has(tag::CAP_BITE.bit) {
        return StrikeParams {
            velocity: BITE_VELOCITY,
            contact_area: BITE_CONTACT_AREA,
            hardness: BITE_HARDNESS,
            is_pressure: false,
            pressure_csa: 0.0,
        };
    }
    // 工具使用 → 读取装备卡标签（后续 handoff）
    if tags.has(tag::CAP_TOOL_USE.bit) {
        return StrikeParams {
            velocity: TOOL_VELOCITY,
            contact_area: TOOL_CONTACT_AREA,
            hardness: TOOL_HARDNESS,
            is_pressure: false,
            pressure_csa: 0.0,
        };
    }
    // 默认：冲撞/踢（无 capability 标签的动物也能攻击）
    StrikeParams {
        velocity: DEFAULT_VELOCITY,
        contact_area: DEFAULT_CONTACT_AREA,
        hardness: DEFAULT_HARDNESS,
        is_pressure: false,
        pressure_csa: 0.0,
    }
}

/// 从 body_plan 推导：该攻击方式的有效质量占体重的比例。
/// `for_bite` 为 true 时使用头部比例（咬合），false 时使用四肢/尾部比例。
pub fn effective_mass_ratio(tags: &TagBits, for_bite: bool) -> f32 {
    if tags.has(tag::PLAN_QUADRUPED.bit) {
        if for_bite {
            return RATIO_BITE_QUADRUPED;
        }
        return RATIO_KICK_QUADRUPED;
    }
    if tags.has(tag::PLAN_BIPED.bit) {
        if for_bite {
            return RATIO_BITE_BIPED;
        }
        return RATIO_KICK_BIPED;
    }
    if tags.has(tag::PLAN_SERPENTINE.bit) {
        return RATIO_SERPENTINE; // 绞杀用全身肌肉
    }
    if tags.has(tag::PLAN_FISH.bit) {
        if for_bite {
            return RATIO_BITE_FISH;
        }
        return RATIO_TAIL_FISH; // 尾击
    }
    RATIO_DEFAULT // 默认
}

/// 同构 Strike 伤害计算——纯函数，从标签参数算伤害。
///
/// # Parameters
/// - `attacker_tags`: 攻击者的 TagBits（body_size, body_plan, capability）
/// - `attacker_mass_kg`: 攻击者质量 (kg)
/// - `_defender_tags`: 目标 TagBits（后续 handoff 查 defense:armor）
///
/// # Returns
/// 冲击力 (N)。调用方除以 1000 并 ceil 得伤害值。
pub fn strike_force(
    attacker_tags: &TagBits,
    attacker_mass_kg: f32,
    _defender_tags: &TagBits,
) -> f32 {
    let params = capability_params(attacker_tags);
    let is_bite = attacker_tags.has(tag::CAP_BITE.bit);
    let ratio = effective_mass_ratio(attacker_tags, is_bite);
    let effective_mass = attacker_mass_kg * ratio;

    if params.is_pressure {
        // 压力型：肌肉截面积 × 300 kPa
        // 截面积从 body_plan + body_size 推导（简化：CSA ≈ 有效质量/肌肉密度/典型纤维长度）
        let csa = effective_mass / crate::meta_values::DENSITY_FLESH / MUSCLE_FIBER_LENGTH_M
            * M2_TO_CM2;
        let pressure_n_per_cm2 = MUSCLE_PRESSURE_PA / M2_TO_CM2; // 300 kPa → 30 N/cm²
        csa * pressure_n_per_cm2
    } else {
        // 打击型：½ × m × v² / A × 硬度比
        let hardness_ratio = params.hardness / TARGET_DEFAULT_HARDNESS;
        0.5 * effective_mass * params.velocity.powi(2) / params.contact_area * hardness_ratio
    }
}

// ===== 测试 =====

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags::{TagBits, tag};

    fn make_tags(bits: &[u16]) -> TagBits {
        let mut t = TagBits::new();
        for &b in bits {
            t.set(b);
        }
        t
    }

    // ── capability_params ──

    #[test]
    fn bite_yields_strike_params() {
        let tags = make_tags(&[tag::CAP_BITE.bit]);
        let p = capability_params(&tags);
        assert!(!p.is_pressure);
        assert!(p.velocity > 0.0);
        assert!(p.contact_area < 1.0);
    }

    #[test]
    fn constrict_yields_pressure_params() {
        let tags = make_tags(&[tag::CAP_CONSTRICT.bit]);
        let p = capability_params(&tags);
        assert!(p.is_pressure);
        assert_eq!(p.velocity, 0.0);
    }

    #[test]
    fn tool_use_yields_strike_params() {
        let tags = make_tags(&[tag::CAP_TOOL_USE.bit]);
        let p = capability_params(&tags);
        assert!(!p.is_pressure);
        assert!(p.velocity > 0.0);
    }

    #[test]
    fn no_capability_yields_default_strike_params() {
        let tags = TagBits::new(); // 无任何标签
        let p = capability_params(&tags);
        assert!(!p.is_pressure);
        assert!(p.velocity > 0.0);
    }

    // ── effective_mass_ratio ──

    #[test]
    fn quadruped_bite_ratio() {
        let tags = make_tags(&[tag::PLAN_QUADRUPED.bit]);
        assert_eq!(effective_mass_ratio(&tags, true), 0.08);
        assert_eq!(effective_mass_ratio(&tags, false), 0.10);
    }

    #[test]
    fn biped_bite_ratio() {
        let tags = make_tags(&[tag::PLAN_BIPED.bit]);
        assert_eq!(effective_mass_ratio(&tags, true), 0.06);
        assert_eq!(effective_mass_ratio(&tags, false), 0.15);
    }

    #[test]
    fn serpentine_ratio_ignores_bite_flag() {
        let tags = make_tags(&[tag::PLAN_SERPENTINE.bit]);
        assert_eq!(effective_mass_ratio(&tags, true), 0.40);
        assert_eq!(effective_mass_ratio(&tags, false), 0.40);
    }

    #[test]
    fn fish_bite_vs_tail_ratio() {
        let tags = make_tags(&[tag::PLAN_FISH.bit]);
        assert_eq!(effective_mass_ratio(&tags, true), 0.03);
        assert_eq!(effective_mass_ratio(&tags, false), 0.30);
    }

    // ── strike_force 集成测试 ──

    /// 老虎咬鹿 > 鹿踢虎（体型差体现）
    #[test]
    fn tiger_bite_deer_stronger_than_deer_kick_tiger() {
        // 老虎: SIZE_MEDIUM (80kg), CAP_BITE, PLAN_QUADRUPED
        let tiger = make_tags(&[
            tag::SIZE_MEDIUM.bit,
            tag::CAP_BITE.bit,
            tag::PLAN_QUADRUPED.bit,
        ]);
        let tiger_mass = crate::axioms::consume::estimate_mass_from_tags(&tiger);

        // 鹿: SIZE_SMALL (15kg), PLAN_QUADRUPED, 无 capability (默认踢)
        let deer = make_tags(&[
            tag::SIZE_SMALL.bit,
            tag::PLAN_QUADRUPED.bit,
        ]);
        let deer_mass = crate::axioms::consume::estimate_mass_from_tags(&deer);

        let tiger_force = strike_force(&tiger, tiger_mass, &deer);
        let deer_force = strike_force(&deer, deer_mass, &tiger);

        assert!(
            tiger_force > deer_force,
            "老虎({} kg) 咬力 {} > 鹿({} kg) 踢力 {}",
            tiger_mass, tiger_force, deer_mass, deer_force
        );
    }

    /// 同体型咬击——不同 body_plan 导致不同咬力
    /// 模拟：serpentine 体型（鳄鱼式，大头部比例） bite > quadruped 体型（老虎式）
    #[test]
    fn same_size_different_bodyplan_bite_differs() {
        // "鳄鱼式": SIZE_MEDIUM (80kg), CAP_BITE, PLAN_SERPENTINE
        let croc = make_tags(&[
            tag::SIZE_MEDIUM.bit,
            tag::CAP_BITE.bit,
            tag::PLAN_SERPENTINE.bit,
        ]);
        let croc_mass = crate::axioms::consume::estimate_mass_from_tags(&croc);

        // "老虎式": SIZE_MEDIUM (80kg), CAP_BITE, PLAN_QUADRUPED
        let tiger = make_tags(&[
            tag::SIZE_MEDIUM.bit,
            tag::CAP_BITE.bit,
            tag::PLAN_QUADRUPED.bit,
        ]);
        let tiger_mass = crate::axioms::consume::estimate_mass_from_tags(&tiger);

        let dummy_defender = TagBits::new();

        let croc_force = strike_force(&croc, croc_mass, &dummy_defender);
        let tiger_force = strike_force(&tiger, tiger_mass, &dummy_defender);

        assert!(
            croc_force > tiger_force,
            "serpentine bite {} > quadruped bite {} (body_plan 差异)",
            croc_force, tiger_force
        );
    }

    /// 蟒蛇绞杀走压力公式（非打击公式）
    #[test]
    fn constrict_uses_pressure_formula() {
        // 蟒蛇: SIZE_SMALL (15kg), CAP_CONSTRICT, PLAN_SERPENTINE
        let python = make_tags(&[
            tag::SIZE_SMALL.bit,
            tag::CAP_CONSTRICT.bit,
            tag::PLAN_SERPENTINE.bit,
        ]);
        let python_mass = crate::axioms::consume::estimate_mass_from_tags(&python);
        let dummy_defender = TagBits::new();

        let force = strike_force(&python, python_mass, &dummy_defender);
        // 压力型公式应产出非零力
        assert!(force > 0.0, "绞杀压力型应产出正力，got {}", force);

        // 验证走的是压力公式而非打击公式：
        // 打击型公式里 velocity=0 会导致力=0，但压力型不受 velocity 影响。
        // 所以 force > 0 已经证明走了压力型路径。
    }

    /// 打击型：力随质量增大而增大
    #[test]
    fn strike_force_increases_with_mass() {
        let small = make_tags(&[
            tag::SIZE_SMALL.bit,
            tag::CAP_BITE.bit,
            tag::PLAN_QUADRUPED.bit,
        ]);
        let huge = make_tags(&[
            tag::SIZE_HUGE.bit,
            tag::CAP_BITE.bit,
            tag::PLAN_QUADRUPED.bit,
        ]);

        let small_mass = crate::axioms::consume::estimate_mass_from_tags(&small);
        let huge_mass = crate::axioms::consume::estimate_mass_from_tags(&huge);
        let dummy = TagBits::new();

        let small_force = strike_force(&small, small_mass, &dummy);
        let huge_force = strike_force(&huge, huge_mass, &dummy);

        assert!(
            huge_force > small_force,
            "大质量 {} kg 的力 {} 应 > 小质量 {} kg 的力 {}",
            huge_mass, huge_force, small_mass, small_force
        );
    }

    /// 无 capability 的默认攻击在 PLAN_QUADRUPED 上也产出正力
    #[test]
    fn default_strike_produces_positive_force() {
        let tags = make_tags(&[
            tag::SIZE_MEDIUM.bit,
            tag::PLAN_QUADRUPED.bit,
        ]);
        let mass = crate::axioms::consume::estimate_mass_from_tags(&tags);
        let dummy = TagBits::new();
        let force = strike_force(&tags, mass, &dummy);
        assert!(force > 0.0);
    }
}
