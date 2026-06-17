# Handoff 017 — 天空盒（低多边形半球 + 太阳/月亮/星星）

## 架构计划

**改什么：** 新建 `src/render/skybox.rs` + `src/render/mod.rs`（2 文件）
**做什么：** 低多边形半球天空盒，纯顶点色。太阳/月亮=发光球。星星=小亮点

### 渲染管线

```
天空盒渲染顺序: 最先画（在所有地形/物体背后）
  → Bevy Camera 设置 clear_color 为透明
  → 天空盒半球永远在相机位置，包围整个世界
```

### skybox.rs

```rust
/// 生成低多边形天空盒半球 mesh
/// 顶点 = 球面坐标，顶点色 = 天空梯度的颜色
pub fn generate_skybox_mesh(resolution: u32) -> Mesh {
    // 半球顶点 + 顶点色（天顶→地平线渐变）
    // 顶点色在白天: 天顶=淡蓝(0.4,0.6,1.0) → 地平线=淡白(0.8,0.85,1.0)
    // 颜色梯度暂时固定，后续 handoff 对接 sim_clock 做昼夜渐变
}

/// 太阳——天空盒上一个发光圆盘
/// sun_dir: 太阳方向向量（来自方向光）
/// sun_size: 太阳盘面大小
/// 顶点色叠加: 靠近 sun_dir 的顶点加暖色
pub fn apply_sun_to_mesh(mesh: &mut Mesh, sun_angle_deg: f32) {
    // sun_angle: 0°=东(升起) / 90°=天顶 / 180°=西(落下) / >180°=夜(不画)
    // 太阳靠近地平线(<30° or >150°) → 橙红色
    // 太阳靠近天顶 → 白黄色
    // 夜间(sun_angle > 180°) → 太阳不可见
}

/// 月亮——天空盒上一个白色圆盘
/// 月相 = 两个圆盘错位相减实现新月→弦月→满月
pub fn apply_moon_to_mesh(mesh: &mut Mesh, moon_angle_deg: f32, phase: f32) {
    // phase: 0.0=新月 / 0.25=上弦 / 0.5=满月 / 0.75=下弦
    // moon_angle: 月亮在天空的角度
    // 只在夜间绘制（sun_angle > 180° 时 moon visible）
}

/// 星星——散点小亮点，夜间可见
pub fn apply_stars_to_mesh(mesh: &mut Mesh, star_count: u32, seed: u64) {
    // 随机撒 star_count 个点
    // 每个点 = 一个微小的亮顶点（白色）
    // 夜间 alpha=1，白天 alpha=0
}

/// 天空盒系统——插入 Bevy Update，读 sim_clock 更新天空盒
pub fn skybox_system(
    sim_clock: Res<SimClock>,
    mut skybox_query: Query<&mut Mesh, With<SkyboxMarker>>,
) {
    let tick_of_day = sim_clock.tick_count % 420;
    let sun_angle = tick_of_day as f32 / 420.0 * 360.0;
    let moon_angle = (sun_angle + 180.0) % 360.0;
    let day_count = sim_clock.tick_count / 420;
    let moon_phase = (day_count % 30) as f32 / 30.0;

    for mut mesh in skybox_query.iter_mut() {
        apply_sun_to_mesh(&mut mesh, sun_angle);
        apply_moon_to_mesh(&mut mesh, moon_angle, moon_phase);
        // apply_stars 同理
    }
}

/// 标记组件
#[derive(Component)]
pub struct SkyboxMarker;
```

### mod.rs

```rust
pub mod skybox;
```

### lib.rs

```rust
pub mod render;
```

## 架构反馈

- 天空盒 = 纯渲染，只读 sim_clock ✅
- 太阳/月亮位置 = 方向向量→球面坐标→像素叠加 ✅
- 顶点色实现天空渐变，不需要贴图 ✅
- 独立模块，不对任何游戏系统产生依赖 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 预设：天空盒 mesh 正确生成（顶点数 > 0，有顶点色）
