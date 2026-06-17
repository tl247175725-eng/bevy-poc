# Handoff 020 — 天空盒补季节/浊度/月相

## 架构计划

**改什么：** `src/render/skybox.rs`（1 文件）
**做什么：** 补三个缺失要素，不改 mesh 结构，只改参数和颜色

### 1. 季节太阳弧

```rust
/// 从 sim_clock 计算季节修正的太阳参数
fn seasonal_sun_params(day_of_year: u16) -> (f32, f32) {
    // 28°N 桃花源纬度
    // 夏至(day 180): 昼长 14.4h, 太阳最高角 85°
    // 冬至(day 0/360):  昼长 9.8h,  太阳最高角 38°
    let year_fraction = day_of_year as f32 / 360.0 * std::f32::consts::TAU;
    let declination = 0.409 * year_fraction.sin(); // 赤纬 −23.45°~+23.45°
    let day_length_ratio = 0.5 + 0.19 * year_fraction.sin(); // 0.41(冬至) ~ 0.69(夏至)
    let max_sun_alt = 0.62 + 0.41 * year_fraction.sin(); // 38° ~ 85°
    (day_length_ratio, max_sun_alt)
}
```

太阳从东升西落改为：昼长 × 420 tick = 白天时长，剩余时间为黑夜。太阳最高角随季节变化。

### 2. 浊度/雾

```rust
/// 从季节和时辰计算天空浊度
fn seasonal_turbidity(season: Season, hour: f32) -> f32 {
    match season {
        Season::Winter => 2.0,  // 旱季清透——深蓝
        Season::Summer => 5.0,  // 雨季高湿——偏白
        Season::Spring => 3.5,  // 适中
        Season::Autumn => 3.0,
    }
    // 晨雾加成：日出前后 2 小时内 +2 浊度
    // 火后加成：如果有 fire_nearby → +3 浊度（后续对接）
}

/// 浊度 → 天空颜色偏移
fn turbidity_to_sky_tint(turbidity: f32) -> [f32; 3] {
    // T=2: 深蓝 [0.3, 0.5, 0.9]
    // T=5: 淡蓝白 [0.5, 0.6, 0.85]
    // T=10: 灰白 [0.6, 0.65, 0.8]
}
```

天空渐变从固定颜色改为随浊度微调。

### 3. 月相

```rust
/// 月相——双盘错位法
fn moon_phase_offset(phase: f32) -> f32 {
    // phase 0.0 = 新月(不画)，0.5 = 满月(全盘画)
    // 0.25 = 上弦(左亮右暗)，0.75 = 下弦(右亮左暗)
    // 用两个圆盘错位相减——第一个盘画月亮，第二个盘偏移遮挡
    phase * 0.2 // 最大偏移量 20% 盘面直径
}
```

### 系统改动

`skybox_system` 新增：
- 读 `sim_clock.day_of_year`（或从 game_time_seconds 换算）
- 读 `SeasonInfo`（从 `season_from_game_seconds` 获取）
- 浊度写入天空顶点色（所有顶点色微调）
- 月相偏移传进 `apply_moon`

## 架构反馈

- 纯参数改动——不改 mesh 结构 ✅
- 全部从 sim_clock 驱动 ✅
- 季节性太阳 + 浊度 + 月相，核心天空变化全了 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
