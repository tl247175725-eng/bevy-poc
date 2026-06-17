use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

// ── 天空盒半球 mesh ──────────────────────────────────────────

const SKY_RADIUS: f32 = 500.0;
const HEMI_RES: u32 = 32;

/// 生成半球 mesh：纬度 0(地平线)→π/2(天顶)，经度 0→2π。
/// 顶点色初始天空渐变：天顶蓝 → 地平线白。
pub fn generate_skybox_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    for lat in 0..=HEMI_RES {
        let theta = lat as f32 / HEMI_RES as f32 * std::f32::consts::FRAC_PI_2;
        for lon in 0..=HEMI_RES * 2 {
            let phi = lon as f32 / (HEMI_RES * 2) as f32 * std::f32::consts::TAU;
            let x = SKY_RADIUS * theta.cos() * phi.cos();
            let y = SKY_RADIUS * theta.sin();
            let z = SKY_RADIUS * theta.cos() * phi.sin();
            positions.push([x, y, z]);

            // 初始天空渐变：天顶蓝 → 地平线白
            let t = theta / std::f32::consts::FRAC_PI_2;
            let r = 0.4 + t * 0.1;
            let g = 0.6 + t * 0.2;
            let b = 0.8 + t * 0.2;
            colors.push([r, g, b, 1.0]);
        }
    }

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

// ── 太阳 / 月亮 ──────────────────────────────────────────────

/// 在已有 mesh 的顶点色上叠加暖黄色光晕。
pub fn apply_sun(mesh: &mut Mesh, sun_dir: Vec3) {
    apply_light_disk(mesh, sun_dir, [1.0, 0.9, 0.4, 1.0], 0.15, 0.08);
}

/// 冷白色月亮光晕，phase 0=新月 0.5=满月 1.0→新月。
/// 使用双盘偏移模拟月相：亮盘 + 偏移阴影盘重叠产生盈亏效果。
pub fn apply_moon(mesh: &mut Mesh, moon_dir: Vec3, phase: f32) {
    use bevy::render::mesh::VertexAttributeValues;

    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(p)) => p.clone(),
        _ => return,
    };
    let colors = match mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(c)) => c,
        _ => return,
    };

    // 月亮水平右方向（用于偏移阴影盘）
    let up = Vec3::Y;
    let right = moon_dir.cross(up).normalize_or_zero();
    if right.length_squared() < 0.001 {
        return; // 月亮在天顶/天底时无水平偏移
    }

    // 双盘偏移量：新月=0(阴影完全覆盖)，满月=最大(无重叠)
    let offset_mag = 1.0 - (2.0 * phase - 1.0).abs(); // 0 → 1 → 0
    let offset_sign = if phase <= 0.5 { 1.0 } else { -1.0 };
    let offset_rad = offset_mag * 0.12;
    let shadow_dir = (moon_dir + right * offset_sign * offset_rad).normalize();

    const OUTER: f32 = 0.14;
    const INNER: f32 = 0.05;

    for i in 0..positions.len() {
        let pos = Vec3::from_array(positions[i]);
        let dir = pos.normalize();
        let dot = dir.dot(moon_dir);

        // 亮盘（冷白）
        if dot > INNER {
            let factor = ((dot - INNER) / (OUTER - INNER)).clamp(0.0, 1.0);
            let smooth = factor * factor * (3.0 - 2.0 * factor);
            colors[i][0] = (colors[i][0] + 0.9 * smooth).min(1.0);
            colors[i][1] = (colors[i][1] + 0.9 * smooth).min(1.0);
            colors[i][2] = (colors[i][2] + 1.0 * smooth).min(1.0);
        }

        // 阴影盘：重叠区域减去亮度
        let shadow_dot = dir.dot(shadow_dir);
        if shadow_dot > INNER && dot > INNER {
            let factor = ((shadow_dot - INNER) / (OUTER - INNER)).clamp(0.0, 1.0);
            let smooth = factor * factor * (3.0 - 2.0 * factor);
            colors[i][0] = (colors[i][0] - 0.9 * smooth * 0.9).max(0.0);
            colors[i][1] = (colors[i][1] - 0.9 * smooth * 0.9).max(0.0);
            colors[i][2] = (colors[i][2] - 1.0 * smooth * 0.9).max(0.0);
        }
    }
}

fn apply_light_disk(
    mesh: &mut Mesh,
    light_dir: Vec3,
    color: [f32; 4],
    outer_radius: f32,
    inner_radius: f32,
) {
    use bevy::render::mesh::VertexAttributeValues;

    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(p)) => p.clone(),
        _ => return,
    };
    let colors = match mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(c)) => c,
        _ => return,
    };

    for i in 0..positions.len() {
        let pos = Vec3::from_array(positions[i]);
        let dir = pos.normalize();
        let dot = dir.dot(light_dir);

        if dot > inner_radius {
            let factor = ((dot - inner_radius) / (outer_radius - inner_radius)).clamp(0.0, 1.0);
            let smooth = factor * factor * (3.0 - 2.0 * factor);
            colors[i][0] = (colors[i][0] + color[0] * smooth).min(1.0);
            colors[i][1] = (colors[i][1] + color[1] * smooth).min(1.0);
            colors[i][2] = (colors[i][2] + color[2] * smooth).min(1.0);
        }
    }
}

// ── 星星 ─────────────────────────────────────────────────────

const STAR_COUNT: usize = 200;

pub fn apply_stars(mesh: &mut Mesh, seed: u64) {
    use bevy::render::mesh::VertexAttributeValues;

    let colors = match mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(c)) => c,
        _ => return,
    };
    if colors.is_empty() {
        return;
    }

    let mut rng = XorShift::new(seed);
    for _ in 0..STAR_COUNT {
        let idx = rng.next() as usize % colors.len();
        colors[idx] = [1.0, 1.0, 1.0, 1.0];
    }
}

// ── 天空盒系统 ────────────────────────────────────────────────

/// 标记组件：天空盒实体。
#[derive(Component)]
pub struct SkyboxTag;

/// 每帧运行，读 sim_clock → 算太阳/月亮方向 → 重置并覆盖顶点色。
pub fn skybox_system(
    sim_clock: Res<crate::sim_clock::SimClock>,
    skybox_query: Query<&Mesh3d, With<SkyboxTag>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let tick = (sim_clock.game_time_seconds / 0.5) as u64;
    let ticks_per_day = crate::meta_values::TICKS_PER_DAY;
    let tick_of_day = tick % ticks_per_day;
    let day_number = tick / ticks_per_day;
    let day_of_year = (day_number % 360) as f32; // 0–359
    let day_angle = tick_of_day as f32 / ticks_per_day as f32 * std::f32::consts::TAU;

    // 太阳方向：在天球上转一圈。0=东方升起，π/2=天顶，π=西方落下
    // 季节倾斜：夏季(+0.25)太阳更高，冬季(-0.25)更低
    let season_tilt =
        0.25 * ((day_of_year / 360.0) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2).sin();
    let sun_base = day_angle.cos();
    let sun_dir = Vec3::new(
        -day_angle.sin(),
        (sun_base + season_tilt).max(-0.15),
        day_angle.cos(),
    );

    // 季节浊度：夏季混浊(暖黄 1.0)，冬季清澈(深蓝 0.0)
    let turbidity = 0.5 + 0.5 * ((day_of_year / 360.0) * std::f32::consts::TAU).sin();

    // 月相：30天一周期，0=新月，0.5=满月，1.0→新月
    let moon_phase = (day_number % 30) as f32 / 30.0;

    for handle in skybox_query.iter() {
        if let Some(mesh) = meshes.get_mut(handle) {
            reset_colors(mesh, turbidity);
            if sun_dir.y > -0.1 {
                // 太阳在地下时不画
                apply_sun(mesh, sun_dir.normalize());
            } else {
                // 夜晚：画月亮和星星
                let moon_dir = -sun_dir.normalize(); // 月亮在太阳对面
                apply_moon(mesh, moon_dir, moon_phase);
                apply_stars(mesh, tick);
            }
        }
    }
}

// ── 重置为初始天空渐变 ───────────────────────────────────────

fn reset_colors(mesh: &mut Mesh, turbidity: f32) {
    use bevy::render::mesh::VertexAttributeValues;

    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(p)) => p.clone(),
        _ => return,
    };
    let colors = match mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(c)) => c,
        _ => return,
    };

    for i in 0..positions.len().min(colors.len()) {
        let pos = Vec3::from_array(positions[i]);
        let y_norm = (pos.y / SKY_RADIUS).clamp(0.0, 1.0); // 0=地平线，1=天顶
        // turbidity 0=清澈(冬季深蓝), 1=混浊(夏季暖黄)
        let r = 0.4 + y_norm * 0.1 + turbidity * 0.15;
        let g = 0.6 + y_norm * 0.2 + turbidity * 0.05;
        let b = (0.8 + y_norm * 0.2 - turbidity * 0.25).max(0.3);
        colors[i] = [r, g, b, 1.0];
    }
}

// ── 微型 seeded RNG（无外部 crate） ───────────────────────────

struct XorShift {
    state: u64,
}

impl XorShift {
    fn new(seed: u64) -> Self {
        Self {
            state: seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407),
        }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}
