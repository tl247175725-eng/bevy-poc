# Handoff 017 v2 — 天空盒（纯 Bevy 0.15 Mesh API）

> v1 失败：写概念描述让 DeepSeek 从零实现 → 26 分钟未完成。
> v2：精确指定 Bevy 0.15 API 调用，不引入新 crate。

## 架构计划

**改什么：** 新建 `src/render/skybox.rs`，修改 `src/render/mod.rs`（2 文件）
**Bevy 版本：** 0.15（项目当前版本）
**无新 crate 依赖。**

### skybox.rs 核心逻辑

**1. 生成半球 mesh：**

```rust
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

/// 天空盒半球：半径 500 单位
const SKY_RADIUS: f32 = 500.0;
const HEMI_RES: u32 = 32; // 纬度分 32 段，经度分 64 段

pub fn generate_skybox_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    
    for lat in 0..=HEMI_RES {
        let theta = lat as f32 / HEMI_RES as f32 * std::f32::consts::FRAC_PI_2; // 0 → π/2（地平线→天顶）
        for lon in 0..=HEMI_RES * 2 {
            let phi = lon as f32 / (HEMI_RES * 2) as f32 * std::f32::consts::TAU; // 0 → 2π
            let x = SKY_RADIUS * theta.cos() * phi.cos();
            let y = SKY_RADIUS * theta.sin(); // y=0 地平线, y=SKY_RADIUS 天顶
            let z = SKY_RADIUS * theta.cos() * phi.sin();
            positions.push([x, y, z]);
            
            // 初始天空渐变：天顶蓝 → 地平线白
            let t = theta / std::f32::consts::FRAC_PI_2; // 0(地平线) → 1(天顶)
            let r = 0.4 + t * 0.1;   // 地平线 0.4 → 天顶 0.5
            let g = 0.6 + t * 0.2;   // 地平线 0.6 → 天顶 0.8
            let b = 0.8 + t * 0.2;   // 地平线 0.8 → 天顶 1.0
            colors.push([r, g, b, 1.0]);
        }
    }
    
    // 构建三角形索引
    let cols = HEMI_RES * 2 + 1;
    for lat in 0..HEMI_RES {
        for lon in 0..HEMI_RES * 2 {
            let a = lat * cols + lon;
            let b = a + 1;
            let c = a + cols;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
```

**2. 太阳/月亮渲染——改顶点色（不是新 mesh）：**

```rust
/// 在已有 mesh 的顶点色上叠加太阳光晕
/// sun_dir: 太阳方向归一化向量
/// 靠近 sun_dir 的顶点加暖黄色
pub fn apply_sun(mesh: &mut Mesh, sun_dir: Vec3) {
    apply_light_disk(mesh, sun_dir, [1.0, 0.9, 0.4, 1.0], 0.15, 0.08);
}

/// 月亮——同上，冷白色
pub fn apply_moon(mesh: &mut Mesh, moon_dir: Vec3, phase: f32) {
    apply_light_disk(mesh, moon_dir, [0.9, 0.9, 1.0, 1.0], 0.12, 0.06);
    // phase: 0=新月(不画), 0.5=满月, 1=新月
    // 月相通过双盘偏移实现——当期先简化为满月
}

fn apply_light_disk(
    mesh: &mut Mesh,
    light_dir: Vec3,
    color: [f32; 4],
    outer_radius: f32,
    inner_radius: f32,
) {
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
    let colors = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR).unwrap();
    
    if let (bevy::render::mesh::VertexAttributeValues::Float32x3(positions), 
            bevy::render::mesh::VertexAttributeValues::Float32x4(colors)) = (positions, colors) {
        for i in 0..positions.len() {
            let pos = Vec3::new(positions[i][0], positions[i][1], positions[i][2]);
            let dir = pos.normalize();
            let dot = dir.dot(light_dir);
            
            if dot > inner_radius {
                let factor = ((dot - inner_radius) / (outer_radius - inner_radius)).clamp(0.0, 1.0);
                // 平滑光晕
                let smooth = factor * factor * (3.0 - 2.0 * factor);
                colors[i][0] = (colors[i][0] + color[0] * smooth).min(1.0);
                colors[i][1] = (colors[i][1] + color[1] * smooth).min(1.0);
                colors[i][2] = (colors[i][2] + color[2] * smooth).min(1.0);
            }
        }
    }
}
```

**3. 星星——稀疏亮顶点：**

```rust
const STAR_COUNT: usize = 200;

pub fn apply_stars(mesh: &mut Mesh, seed: u64) {
    // 随机散布亮白点
    let colors = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR).unwrap();
    if let bevy::render::mesh::VertexAttributeValues::Float32x4(colors) = colors {
        let mut rng = /* seeded RNG from seed */;
        for _ in 0..STAR_COUNT {
            let idx = rng % colors.len();
            colors[idx] = [1.0, 1.0, 1.0, 1.0];
        }
    }
}
```

**4. 天空盒系统——读 sim_clock 更新：**

```rust
#[derive(Component)]
pub struct SkyboxTag;

/// 每帧运行，读 sim_clock → 算角度 → 覆盖顶点色
pub fn skybox_system(
    sim_clock: Res<crate::sim_clock::SimClock>,
    skybox_query: Query<&Handle<Mesh>, With<SkyboxTag>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // SimClock 字段是 game_time_seconds: f64（不是 tick_count）
    let tick = (sim_clock.game_time_seconds / 0.5) as u64;
    let tick_of_day = tick % 420;  // TICKS_PER_DAY = 420
    let day_angle = tick_of_day as f32 / 420.0 * std::f32::consts::TAU;
    
    // 太阳方向：在天球上转一圈。0=东方升起，π/2=天顶，π=西方落下
    let sun_dir = Vec3::new(-day_angle.sin(), day_angle.cos().max(0.0), day_angle.cos());
    
    for handle in skybox_query.iter() {
        if let Some(mesh) = meshes.get_mut(handle) {
            reset_colors(mesh); // 重置为初始天空渐变
            if sun_dir.y > -0.1 { // 太阳在地下时不画
                apply_sun(mesh, sun_dir.normalize());
            } else {
                // 夜晚：画月亮和星星
                let moon_dir = -sun_dir.normalize(); // 月亮在太阳对面
                apply_moon(mesh, moon_dir, 0.5);
                apply_stars(mesh, tick);
            }
        }
    }
}

fn reset_colors(mesh: &mut Mesh) {
    // 重置所有顶点色为初始天空渐变
}
```

### mod.rs 更新

在已有 `pub mod` 列表末尾添加 `pub mod skybox;`

### 注册系统

在应用初始化处调用 `.add_systems(Update, skybox_system)`

## 架构反馈

- Bevy 0.15 原生 Mesh API ✅
- 零新 crate ✅
- 天空盒 = 半球 mesh + 顶点色渐变 + 太阳/月亮/星星覆盖 ✅
- 读 sim_clock，不碰任何游戏逻辑 ✅
- SimClock 字段已核实：`game_time_seconds: f64`（无 `tick_count`）。用 `game_time_seconds / 0.5` 换算 tick

## 预估完成时间

3-5 分钟。纯顶点色操作，无 shader 复杂性。

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
