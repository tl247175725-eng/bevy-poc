//! 天气公理 —— 五变量物理模型
//!
//! 核心公式: Clausius-Clapeyron 饱和水汽压
//!   e_s = 611.3 × exp(5423 × (1/273.15 - 1/T))
//!
//! 五变量:
//!   T   温度 (K)
//!   e   实际水汽压 (Pa)
//!   e_s 饱和水汽压 (Pa) — 从 T 推导
//!   RH  相对湿度 = e / e_s
//!   W   风矢量 — 从温度梯度推导
//!
//! 推导:
//!   RH > 1.0 → 凝结 → 云
//!   云 + 抬升 → 降水
//!   强对流 + 冰晶碰撞 → 电荷分离 → 闪电
//!   凝结释放潜热 → 反馈 T

use crate::meta_values;

// ── 常量 ──────────────────────────────────────────────

/// 水的三相点温度 (K)
pub const T0: f32 = 273.15;
/// 三相点饱和水汽压 (Pa)
pub const E0: f32 = 611.3;
/// Lv / Rv = 2.5×10⁶ / 461 = 5423 K
pub const L_OVER_RV: f32 = 5423.0;
/// 冰的 Ld / Rv = 6139 K
pub const LD_OVER_RV: f32 = 6139.0;

/// 热扩散系数 (格/帧)
pub const THERMAL_DIFFUSION: f32 = 0.02;
/// 水汽扩散系数
pub const VAPOR_DIFFUSION: f32 = 0.03;
/// 蒸发率系数
pub const EVAPORATION_RATE: f32 = 0.05;
/// 凝结降水系数
pub const PRECIPITATION_RATE: f32 = 0.1;
/// 风电转换系数
pub const WIND_FORCE: f32 = 0.005;

/// 每 tick 秒数（从 meta_values 派生）
pub fn tick_seconds() -> f32 { meta_values::TICK_SECONDS }

// ── 天气状态 ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WeatherCell {
    /// 温度 (K)
    pub temperature: f32,
    /// 实际水汽压 (Pa)
    pub vapor_pressure: f32,
    /// 云量 (0..1) — 凝结水质量
    pub cloud_cover: f32,
    /// 降水强度 (mm/tick)
    pub precipitation: f32,
    /// 风向 (弧度)
    pub wind_direction: f32,
    /// 风速 (m/s)
    pub wind_speed: f32,
    /// 闪电概率 (0..1)
    pub lightning_chance: f32,
    /// 潜热累积 (J) — 本 tick 凝结释放的能量
    pub latent_heat: f32,
}

impl Default for WeatherCell {
    fn default() -> Self {
        Self {
            temperature: 293.15,  // 20°C
            vapor_pressure: 1000.0,
            cloud_cover: 0.0,
            precipitation: 0.0,
            wind_direction: 0.0,
            wind_speed: 0.0,
            lightning_chance: 0.0,
            latent_heat: 0.0,
        }
    }
}

// ── 气候基准（各海拔/纬度/季节的"无天气时的温度"） ────

/// 地表基准温度 (K)，由纬度 + 季节 + 太阳高度决定
pub fn baseline_temperature(latitude_01: f32, sun_elevation: f32, elevation_m: f32) -> f32 {
    // 赤道基准 303K(30°C)，极地 263K(-10°C)
    let lat_base = 303.0 - latitude_01 * 40.0;
    // 太阳高度的日变化
    let solar_mod = sun_elevation * 15.0;
    // 干绝热递减率 ~10°C/km
    let elev_mod = elevation_m * -0.01;
    lat_base + solar_mod + elev_mod
}

// ── Clausius-Clapeyron ────────────────────────────────

/// 计算饱和水汽压 (Pa)
/// e_s = E0 × exp( L/Rv × (1/T0 - 1/T) )
pub fn saturation_vapor_pressure(temperature_k: f32, over_ice: bool) -> f32 {
    let l_rv = if over_ice { LD_OVER_RV } else { L_OVER_RV };
    let t = temperature_k.max(180.0).min(373.0); // 物理合理范围
    E0 * (l_rv * (1.0 / T0 - 1.0 / t)).exp()
}

/// 从温度和露点温度反算相对湿度
/// RH = e_s(T_dew) / e_s(T)
pub fn relative_humidity(temperature_k: f32, dew_point_k: f32) -> f32 {
    if dew_point_k >= temperature_k { return 1.0; }
    saturation_vapor_pressure(dew_point_k, false) / saturation_vapor_pressure(temperature_k, false)
}

// ── 蒸发 ──────────────────────────────────────────────

/// 计算单位时间蒸发量 (Pa/tick)
/// 蒸发速率 ∝ 水面饱和水汽压 - 当前水汽压
pub fn evaporation_rate(water_temp_k: f32, current_vapor_pressure: f32, wind_speed: f32) -> f32 {
    let e_sat = saturation_vapor_pressure(water_temp_k, false);
    let deficit = (e_sat - current_vapor_pressure).max(0.0);
    // 风加速蒸发
    deficit * EVAPORATION_RATE * (1.0 + wind_speed * 0.1) * tick_seconds()
}

// ── 凝结与降水 ────────────────────────────────────────

/// 计算凝结率和降水
/// 返回 (cloud_increase, precipitation_mm)
pub fn condensation_and_precip(
    vapor_pressure: f32,
    temperature_k: f32,
    cloud_cover: f32,
    lift: f32, // 抬升力 (0..1)
) -> (f32, f32) {
    let e_sat = saturation_vapor_pressure(temperature_k, false);
    let rh = vapor_pressure / e_sat;

    if rh <= 1.0 {
        // 未饱和 → 不凝结（云可能消散）
        let cloud_decay = cloud_cover * 0.01;
        return (cloud_decay, 0.0);
    }

    // 过饱和 → 凝结
    let excess = (vapor_pressure - e_sat).max(0.0);
    let cloud_add = excess * PRECIPITATION_RATE * tick_seconds();

    // 云层够厚 + 有抬升 → 降水
    let new_cloud = cloud_cover + cloud_add;
    let precip = if new_cloud > 0.3 && lift > 0.1 {
        (new_cloud - 0.3) * lift * 10.0 * tick_seconds()
    } else {
        0.0
    };

    (cloud_add, precip)
}

// ── 潜热释放 ──────────────────────────────────────────

/// 凝结释放的潜热 (J)
/// Lv = 2.5×10⁶ J/kg，降水 1mm = 1kg/m²
pub fn latent_heat_released(precipitation_mm: f32) -> f32 {
    precipitation_mm * 2.5e6 // J/m²
}

/// 潜热 → 温升 (K)
/// ΔT = Q / (m × cp), 取格子上方空气柱质量 ≈ 1.2kg/m³ × 格子高度
pub fn latent_heat_to_temp_rise(latent_heat_j: f32, air_column_mass_kg: f32) -> f32 {
    let cp = 1005.0; // 空气比热 J/(kg·K)
    latent_heat_j / (air_column_mass_kg * cp)
}

// ── 风 ────────────────────────────────────────────────

/// 从温度梯度计算风
/// 风从高温吹向低温（简化：从高压吹向低压，但 T ∝ P）
pub fn wind_from_temperature_gradient(
    temp_self: f32,
    temps_neighbor: &[(f32, f32, f32)], // (dx, dy, T)
) -> (f32, f32) { // (direction radians, speed m/s)
    let mut fx = 0.0f32;
    let mut fy = 0.0f32;

    for &(dx, dy, t_neighbor) in temps_neighbor {
        let grad = (temp_self - t_neighbor).max(0.0); // 从高温→低温
        let dist = (dx * dx + dy * dy).sqrt().max(0.1);
        fx += grad * dx / dist;
        fy += grad * dy / dist;
    }

    let speed = (fx * fx + fy * fy).sqrt() * WIND_FORCE;
    let direction = if speed > 0.001 {
        fy.atan2(fx)
    } else {
        0.0
    };

    (direction, speed.min(60.0)) // 60m/s 上限（超强台风级）
}

// ── 闪电 ──────────────────────────────────────────────

/// 闪电概率
/// 条件：强对流 + 云中有冰晶碰撞 → 电荷分离
pub fn lightning_probability(
    cloud_cover: f32,
    temperature_k: f32,
    convection_strength: f32, // 对流强度 (0..1)
) -> f32 {
    // 云要厚，温度要低于冰点，要有对流
    if cloud_cover < 0.5 || temperature_k > 273.15 || convection_strength < 0.3 {
        return 0.0;
    }
    (cloud_cover - 0.5) * (273.15 - temperature_k) / 30.0 * convection_strength
}

// ── 雾 ────────────────────────────────────────────────

/// 雾的可见度衰减
/// 条件：RH 接近 100% + 风小 + 温度接近露点
pub fn fog_density(
    relative_humidity: f32,
    wind_speed: f32,
    temp_dewpoint_spread_k: f32, // T - T_dew
) -> f32 {
    if relative_humidity < 0.85 || wind_speed > 5.0 {
        return 0.0;
    }
    // 露点差越小，雾越浓
    let dew_factor = (1.0 - temp_dewpoint_spread_k / 3.0).clamp(0.0, 1.0);
    ((relative_humidity - 0.85) / 0.15).clamp(0.0, 1.0) * dew_factor * (1.0 - wind_speed / 5.0)
}

// ── 天气格 → 表现参数 ─────────────────────────────────

/// 从天气状态推导视觉/游戏参数
pub fn weather_visual_params(cell: &WeatherCell) -> WeatherVisual {
    WeatherVisual {
        sky_tint: sky_tint_from_temp(cell.temperature),
        fog_opacity: fog_density(
            cell.vapor_pressure / saturation_vapor_pressure(cell.temperature, false),
            cell.wind_speed,
            cell.temperature - dew_point_from_vapor(cell.vapor_pressure),
        ),
        precipitation_particles: cell.precipitation > 0.001,
        precipitation_intensity: cell.precipitation,
        precipitation_type: if cell.temperature < 273.15 { PrecipType::Snow } else { PrecipType::Rain },
        lightning_active: cell.lightning_chance > 0.3,
        wind_sway: cell.wind_speed * 0.02,
    }
}

fn dew_point_from_vapor(e: f32) -> f32 {
    // 逆解 Clausius-Clapeyron
    // e = E0 × exp(L/Rv × (1/T0 - 1/Td))
    // → 1/Td = 1/T0 - ln(e/E0) / (L/Rv)
    let ratio = (e / E0).max(0.001).ln();
    1.0 / (1.0 / T0 - ratio / L_OVER_RV)
}

fn sky_tint_from_temp(_t: f32) -> [f32; 3] {
    // 占位——后续接入天空着色系统
    [0.0, 0.0, 0.0]
}

#[derive(Debug, Clone)]
pub struct WeatherVisual {
    pub sky_tint: [f32; 3],
    pub fog_opacity: f32,
    pub precipitation_particles: bool,
    pub precipitation_intensity: f32,
    pub precipitation_type: PrecipType,
    pub lightning_active: bool,
    pub wind_sway: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrecipType {
    Rain,
    Snow,
}

// ── 整场 tick ─────────────────────────────────────────

/// 推进一格子的天气状态
/// 输入：本格天气、邻格天气摘要、地表标签、太阳高度、卡牌 modifier
pub fn tick_weather_cell(
    cell: &mut WeatherCell,
    baseline_temp: f32,           // 气候基准温度
    neighbor_temps: &[(f32, f32, f32)], // (dx, dy, T_neighbor)
    neighbor_vapors: &[(f32, f32, f32)], // (dx, dy, e_neighbor)
    is_water_surface: bool,       // 该格是否水体
    water_surface_temp_k: f32,    // 水温（如有）
    lift: f32,                    // 地形抬升力 (0..1)
    temp_mod: f32,                // 卡牌温度修正 (K)
    humidity_mod: f32,            // 卡牌湿度修正 (Pa)
) {
    // 1. 温度：向基准值 + 扩散
    let target_temp = baseline_temp + temp_mod;
    cell.temperature += (target_temp - cell.temperature) * THERMAL_DIFFUSION;
    // 邻格热扩散
    for &(_dx, _dy, tn) in neighbor_temps {
        cell.temperature += (tn - cell.temperature) * THERMAL_DIFFUSION * 0.25;
    }
    // 潜热释放反馈
    cell.temperature += latent_heat_to_temp_rise(cell.latent_heat, 1200.0);

    // 2. 水汽：蒸发 + 扩散 + 凝结扣除
    if is_water_surface {
        let evap = evaporation_rate(water_surface_temp_k, cell.vapor_pressure, cell.wind_speed);
        cell.vapor_pressure += evap + humidity_mod * tick_seconds();
    }
    for &(_dx, _dy, en) in neighbor_vapors {
        cell.vapor_pressure += (en - cell.vapor_pressure) * VAPOR_DIFFUSION;
    }

    // 3. 凝结与降水
    let (cloud_delta, precip_mm) = condensation_and_precip(
        cell.vapor_pressure, cell.temperature, cell.cloud_cover, lift,
    );
    cell.cloud_cover = (cell.cloud_cover + cloud_delta - precip_mm * 0.01).clamp(0.0, 1.0);
    cell.precipitation = precip_mm;
    // 凝结消耗水汽
    cell.vapor_pressure -= cloud_delta * 100.0;

    // 4. 风
    let (wdir, wspd) = wind_from_temperature_gradient(cell.temperature, neighbor_temps);
    cell.wind_direction = wdir;
    cell.wind_speed = wspd;

    // 5. 潜热
    cell.latent_heat = latent_heat_released(precip_mm);

    // 6. 闪电
    let convection = if cell.cloud_cover > 0.5 { cell.cloud_cover * lift } else { 0.0 };
    cell.lightning_chance = lightning_probability(cell.cloud_cover, cell.temperature, convection);

    // 钳制
    cell.temperature = cell.temperature.clamp(180.0, 330.0);
    cell.vapor_pressure = cell.vapor_pressure.max(0.0);
}

// ── 测试 ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cc_saturation_increases_with_temp() {
        let cold = saturation_vapor_pressure(273.15, false);  // 0°C
        let warm = saturation_vapor_pressure(293.15, false);  // 20°C
        let hot = saturation_vapor_pressure(313.15, false);   // 40°C
        assert!(warm > cold, "warm air holds more water");
        assert!(hot > warm, "hot air holds even more");
    }

    #[test]
    fn rh_at_saturation_is_one() {
        let t = 293.15;
        let e_sat = saturation_vapor_pressure(t, false);
        let rh = relative_humidity(t, t); // T = T_dew → saturated
        assert!((rh - 1.0).abs() < 0.01);
    }

    #[test]
    fn evaporation_produces_vapor() {
        let rate = evaporation_rate(293.15, 500.0, 2.0); // 20°C 水面, 低湿度, 有风
        assert!(rate > 0.0);
    }

    #[test]
    fn supersaturated_air_condenses() {
        let (cloud, precip) = condensation_and_precip(2500.0, 283.15, 0.4, 0.5);
        // 10°C 的 e_s ≈ 1228 Pa, 2500 >> 1228 → 过饱和
        assert!(cloud > 0.0, "supersaturated air should form clouds");
        assert!(precip > 0.0, "thick cloud + lift should precipitate");
    }

    #[test]
    fn unsaturated_air_no_condensation() {
        let (cloud, precip) = condensation_and_precip(500.0, 293.15, 0.0, 0.0);
        // 20°C 的 e_s ≈ 2338 Pa, 500 << 2338 → 干燥
        assert!(cloud == 0.0);
        assert!(precip == 0.0);
    }

    #[test]
    fn lightning_requires_cold_thick_cloud() {
        let p1 = lightning_probability(0.6, 263.15, 0.5); // 好条件
        let p2 = lightning_probability(0.3, 283.15, 0.2); // 云太薄/太暖
        assert!(p1 > 0.0);
        assert!(p2 == 0.0);
    }

    #[test]
    fn fog_needs_high_humidity_low_wind() {
        let d1 = fog_density(0.98, 0.5, 0.3);  // 高湿/微风/露点极近 → 浓雾
        let d2 = fog_density(0.70, 10.0, 5.0); // 干燥/大风 → 无雾
        assert!(d1 > 0.5);
        assert!(d2 == 0.0);
    }

    #[test]
    fn full_tick_produces_reasonable_values() {
        let mut cell = WeatherCell::default();
        let neighbors_t = vec![(1.0, 0.0, 295.0), (0.0, 1.0, 290.0)];
        let neighbors_v = vec![(1.0, 0.0, 1200.0), (0.0, 1.0, 800.0)];

        tick_weather_cell(
            &mut cell, 295.0, &neighbors_t, &neighbors_v,
            true, 295.0, 0.3, 0.0, 0.0,
        );

        // 水温 295K → 蒸发 → 水汽上升
        assert!(cell.vapor_pressure > 1000.0, "evaporation should add vapor");
        // 温度应在合理范围内
        assert!(cell.temperature > 280.0 && cell.temperature < 310.0);
        assert!(cell.wind_speed >= 0.0);
    }
}
