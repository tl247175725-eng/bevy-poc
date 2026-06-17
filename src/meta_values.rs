//! 元数值 — 世界基础量纲
//!
//! 所有游戏中的数值都必须从此文件的常量派生。
//! 禁止在任何其他文件中出现无法追溯到此处常量的裸数字。
//!
//! 设计哲学 §8.1/§8.2: A 层（必须）+ B 层（强推荐）
//! 铁律：1 day = 2100 tick
//! 渲染采样率：TICK_SECONDS = 0.5（非游戏内逻辑时间）

use crate::tags::{TagBits, tag};

// ===== A 层·必须 =====

// --- 时间 ---
pub const TICK_SECONDS: f32 = 0.5;
pub const TICKS_PER_DAY: u64 = 2100;     // 17.5 分钟一天
pub const TICKS_PER_PHASE: u64 = 300;    // 2100/7
pub const PHASES_PER_DAY: u64 = 7;

// --- 空间 ---
pub const GRID_CELL_SIZE: f32 = 1.0;
pub const CELL_AREA: f32 = 1.0;
pub const CELL_VOLUME: f32 = 1.0;

// --- 材料物理参考值（密度 kg/m³） ---
pub const DENSITY_WATER: f32 = 1000.0;
pub const DENSITY_WOOD: f32 = 700.0;
pub const DENSITY_STONE: f32 = 2700.0;
pub const DENSITY_IRON: f32 = 7874.0;
pub const DENSITY_COPPER: f32 = 8960.0;
pub const DENSITY_BRONZE: f32 = 8700.0;
pub const DENSITY_STEEL: f32 = 7850.0;
pub const DENSITY_GOLD: f32 = 19320.0;
pub const DENSITY_SILVER: f32 = 10490.0;
pub const DENSITY_LEATHER: f32 = 860.0;
pub const DENSITY_BONE: f32 = 1900.0;
pub const DENSITY_GLASS: f32 = 2500.0;
pub const DENSITY_CLAY: f32 = 1800.0;
pub const DENSITY_FLESH: f32 = 1050.0;
pub const DENSITY_ICE: f32 = 917.0;

// --- 材料物理参考值（摩斯硬度） ---
pub const HARDNESS_WOOD: f32 = 1.0;
pub const HARDNESS_COPPER: f32 = 3.0;
pub const HARDNESS_BRONZE: f32 = 3.5;
pub const HARDNESS_IRON: f32 = 4.0;
pub const HARDNESS_STEEL: f32 = 6.5;
pub const HARDNESS_STONE: f32 = 7.0;
pub const HARDNESS_GLASS: f32 = 5.5;

// --- 材料物理参考值（屈服强度 MPa） ---
pub const YIELD_WOOD: f32 = 40.0;
pub const YIELD_COPPER: f32 = 200.0;
pub const YIELD_BRONZE: f32 = 350.0;
pub const YIELD_IRON: f32 = 500.0;
pub const YIELD_STEEL: f32 = 800.0;

// --- 材料物理参考值（断裂强度 MPa） ---
pub const FRACTURE_WOOD: f32 = 70.0;
pub const FRACTURE_COPPER: f32 = 250.0;
pub const FRACTURE_BRONZE: f32 = 500.0;
pub const FRACTURE_IRON: f32 = 800.0;
pub const FRACTURE_STEEL: f32 = 1200.0;

// --- 材料物理参考值（韧性 J/m³） ---
pub const TOUGHNESS_STEEL: f32 = 150.0;
pub const TOUGHNESS_GLASS: f32 = 0.01;
pub const TOUGHNESS_WOOD: f32 = 2.0;

// --- 热 ---
pub const TEMP_ABSOLUTE_ZERO: f32 = -273.15;
pub const TEMP_FREEZING_WATER: f32 = 0.0;
pub const TEMP_BOILING_WATER: f32 = 100.0;
pub const TEMP_IGNITION_WOOD: f32 = 300.0;
pub const TEMP_BODY_MAMMAL: f32 = 37.0;

// --- 感官默认 ---
pub const VISION_RANGE_DEFAULT: u32 = 6;
pub const HEARING_RANGE_DEFAULT: u32 = 8;
pub const SMELL_RANGE_DEFAULT: u32 = 4;

// ===== B 层·强推荐 =====

// --- 生命 ---
/// 工程保留——逻辑上由 wound/infection/toxin/hydration 等共同决定，此处为妥协。
pub const HP_BASELINE: u64 = 1;
pub const METABOLISM_BASELINE: f32 = 1.0;

// --- 心智 ---
pub const DECISION_THRESHOLD_DEFAULT: f32 = 1.2;

// --- 社会 ---
pub const NORM_STRENGTH_DEFAULT: f32 = 0.5;

// === 需求衰减率（同构推导，从 TICKS_PER_DAY 分母计算） ===
pub const SOCIAL_DECAY_RATE: f32 = 0.1 / (TICKS_PER_DAY as f32 * TICK_SECONDS);
pub const CURIOSITY_DECAY_RATE: f32 = 0.05 / (TICKS_PER_DAY as f32 * TICK_SECONDS);

// === 需求基线值 ===
pub const NUTRITION_BASELINE: f32 = 0.3;
pub const SAFETY_BASELINE: f32 = 1.0;
pub const SOCIAL_BASELINE: f32 = 0.5;
pub const CURIOSITY_BASELINE: f32 = 0.2;

// === 感知范围默认值 ===
pub const MAX_SENSE_RANGE: u8 = 20;

// === 需求匹配阈值 ===
pub const URGENCY_ACTIVATION_THRESHOLD: f32 = 0.3;
pub const SAFETY_BLOCK_THRESHOLD: f32 = 0.7;

// === 消化效率 ===
pub const DIGESTION_EFFICIENCY: f32 = 0.5;

// === 攻击基础值 ===
/// Strike 基础伤害（后续将被 impact_force() 公式替代）
pub const STRIKE_BASE_DAMAGE: i32 = 1;

// === 补水基线比例 ===
pub const HYDRATION_BASELINE_RATIO: f32 = 0.03;

// === 饥饿致死（同构热力学公式） ===

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

// ===== 手牌 =====

/// 玩家手牌上限
pub const HAND_SIZE_MAX: usize = 5;

// ===== 辅助常量 =====

pub const GRAVITY: f32 = 9.8;

// ===== 元数值推导函数 =====

/// 代谢→衰减率推导（同构：从每日能量需求算每tick衰减）
/// 中代谢动物约1游戏天从饱到饿
pub fn nutrition_decay_per_tick(metab_rate: f32) -> f32 {
    metab_rate / (TICKS_PER_DAY as f32 * TICK_SECONDS)
}

/// 代谢率从 metab 标签推导
pub fn metab_rate_from_tags(tags: &TagBits) -> f32 {
    if tags.has(tag::METAB_HIGH.bit) { return 1.5; }
    if tags.has(tag::METAB_LOW.bit)  { return 0.5; }
    1.0  // medium 默认
}

/// 重量 = 质量 × 重力常数
pub fn weight_from_mass_density(mass_kg: f32, _density: f32) -> f32 {
    mass_kg * GRAVITY
}

/// 速度 = base_speed × (1.0 / sqrt(mass))
pub fn speed_from_mass(mass_kg: f32) -> f32 {
    GRID_CELL_SIZE * (1.0 / mass_kg.sqrt())
}

/// 撞击力 = 0.5 × mass × v² / contact_area × hardness_ratio
pub fn impact_force(mass_kg: f32, velocity: f32, contact_area: f32, hardness_ratio: f32) -> f32 {
    0.5 * mass_kg * velocity.powi(2) / contact_area * hardness_ratio
}

/// baseline_energy = mass × metabolism_rate
pub fn baseline_energy(mass_kg: f32, metabolism_rate: f32) -> f32 {
    mass_kg * metabolism_rate
}

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

// ===== 测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_per_day_is_2100() {
        assert_eq!(TICKS_PER_DAY, 2100);
    }

    #[test]
    fn ticks_per_phase_is_300() {
        assert_eq!(TICKS_PER_PHASE, 300);
    }

    #[test]
    fn weight_derives_from_mass_and_gravity() {
        assert_eq!(weight_from_mass_density(10.0, 1000.0), 10.0 * 9.8);
    }

    #[test]
    fn speed_decreases_with_heavier_mass() {
        assert!(speed_from_mass(100.0) < speed_from_mass(10.0));
    }

    #[test]
    fn impact_force_increases_with_mass() {
        assert!(impact_force(100.0, 1.0, 1.0, 1.0) > impact_force(10.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn all_constants_positive() {
        assert!(TICKS_PER_DAY > 0);
        assert!(GRID_CELL_SIZE > 0.0);
        assert!(GRAVITY > 0.0);
    }

    #[test]
    fn all_speeds_are_positive() {
        for mass in [1.0, 10.0, 50.0, 100.0, 500.0] {
            assert!(speed_from_mass(mass) > 0.0);
        }
    }

    #[test]
    fn fasting_endurance_deer_about_30_days() {
        // 鹿: 80kg 恒温 无休眠 → ~30天
        let days = fasting_endurance_days(80.0, false, false);
        assert!((days - 30.0).abs() < 2.0, "deer fasting days = {days}, expected ~30");
    }

    #[test]
    fn fasting_endurance_carp_about_119_days() {
        // 鲤鱼: 3kg 变温 无休眠 → ~119天
        let days = fasting_endurance_days(3.0, true, false);
        assert!((days - 119.0).abs() < 5.0, "carp fasting days = {days}, expected ~119");
    }

    #[test]
    fn fasting_endurance_bear_with_torpor_about_113_days() {
        // 熊: 200kg 恒温 冬眠 → ~113天
        let days = fasting_endurance_days(200.0, false, true);
        assert!((days - 113.0).abs() < 5.0, "bear fasting days = {days}, expected ~113");
    }
}