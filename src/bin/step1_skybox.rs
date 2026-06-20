//! Step 1: 天空盒 + 昼夜循环 + 视角旋转
//! cargo run --bin step1_skybox

use bevy::color::Srgba;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::render::render_resource::PrimitiveTopology;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// ── 世界尺度 ──────────────────────────────────────────
// 64×64 棋盘，每格 158m → 边长 ≈ 10112m
// 天空球半径需覆盖棋盘对角线 ≈ 14300m，取 15000

const GRID_COUNT: f32 = 64.0;
const CELL_SIZE: f32 = 158.0;
const WORLD_HALF: f32 = GRID_COUNT * CELL_SIZE * 0.5; // ≈ 5056m
const SKY_RADIUS: f32 = WORLD_HALF * 2.8;             // ≈ 14157，天空包围世界

const HEMI_RES: u32 = 64;

// ── 天体周期 ──────────────────────────────────────────
const DAY_TICKS: f32 = 2100.0;      // 一天 tick 数（同构 420 tick/phase）
const SUN_ANGLE_PER_TICK: f32 = TAU / DAY_TICKS;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Step 1 — 天空盒（昼夜）".into(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK)) // 地平线以下是黑的 → 后面放棋盘
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_orbit, sky_color_update, sun_light_update))
        .run();
}

// ── 资源 ──────────────────────────────────────────────

#[derive(Resource)]
struct OrbitCam {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl Default for OrbitCam {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: 0.3, distance: WORLD_HALF * 3.0 }
    }
}

#[derive(Resource)]
struct DayCycle {
    tick: f32, // 当前 tick（0 → DAY_TICKS）
}

// ── 组件标记 ──────────────────────────────────────────

#[derive(Component)]
struct SkyDome;

#[derive(Component)]
struct SunLight;

// ── 网格生成 ──────────────────────────────────────────

fn build_sky_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new(); // 顶点色——每帧更新
    let mut indices = Vec::new();

    for lat in 0..=HEMI_RES {
        let theta = lat as f32 / HEMI_RES as f32 * FRAC_PI_2;
        for lon in 0..=HEMI_RES * 2 {
            let phi = lon as f32 / (HEMI_RES * 2) as f32 * TAU;
            let x = SKY_RADIUS * theta.cos() * phi.cos();
            let y = SKY_RADIUS * theta.sin();
            let z = SKY_RADIUS * theta.cos() * phi.sin();
            positions.push([x, y, z]);
            let len = (x * x + y * y + z * z).sqrt();
            normals.push([-x / len, -y / len, -z / len]);
            colors.push([0.0, 0.0, 0.0, 1.0]); // 初始黑色——首帧 sky_color_update 会写
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
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

// ── 启动 ──────────────────────────────────────────────

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 天空半球（顶点色，每帧重算）
    commands.spawn((
        Mesh3d(meshes.add(build_sky_mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_xyz(0.0, -WORLD_HALF * 0.15, 0.0), // 球心略低于地表
        SkyDome,
    ));

    // 太阳方向光（色温根据高度变化）
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.9, 0.7),
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::default(),
        SunLight,
    ));

    // 环境光（模拟天空散射——夜晚暗、白天亮）
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.3, 0.4, 0.6),
        brightness: 800.0,
        affects_lightmapped_meshes: false,
    });

    // 透视相机——看向棋盘中心
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(WORLD_HALF, WORLD_HALF * 0.8, WORLD_HALF * 1.5)
            .looking_at(Vec3::new(WORLD_HALF, 0.0, WORLD_HALF), Vec3::Y),
    ));

    commands.insert_resource(OrbitCam::default());
    commands.insert_resource(DayCycle { tick: 0.0 });
}

// ── 太阳位置计算 ──────────────────────────────────────

fn sun_angle(tick: f32) -> f32 {
    // 0 tick = 日出（东），tick 增加 = 太阳从东向南→西→北→东
    // sun_angle = 0 → 太阳在正东地平线
    // sun_angle = PI/2 → 太阳在正南天顶
    // sun_angle = PI → 太阳在正西地平线
    // sun_angle = 3PI/2 → 太阳在正北（地下=夜晚）
    tick * SUN_ANGLE_PER_TICK
}

fn sun_dir(tick: f32) -> Vec3 {
    let a = sun_angle(tick);
    // X = 东(+)/西(-)，Y = 高度，Z = 南(+)/北(-)
    Vec3::new(a.cos(), a.sin(), 0.0).normalize()
}

fn sun_elevation(tick: f32) -> f32 {
    sun_angle(tick).sin() // -1(最低/午夜) → 1(最高/正午)
}

// ── 太阳光更新 ────────────────────────────────────

fn sun_light_update(
    day: Res<DayCycle>,
    mut q_sun: Query<(&mut DirectionalLight, &mut Transform), With<SunLight>>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let elev = sun_elevation(day.tick);
    let dir = sun_dir(day.tick);

    // 白天：暖黄 → 正午白 → 傍晚橙
    let (r, g, b, bright) = if elev > 0.0 {
        let t = elev; // 0→1 地平线→天顶
        let r = 1.0;
        let g = 0.75 + t * 0.25;
        let b = 0.4 + t * 0.4;
        let bright = 2000.0 + elev * 8000.0;
        (r, g, b, bright)
    } else {
        // 夜晚：月光蓝白
        (0.3, 0.4, 0.7, 200.0)
    };

    if let Ok((mut light, mut transform)) = q_sun.single_mut() {
        light.color = Color::srgb(r, g, b);
        light.illuminance = bright;
        transform.look_to(-dir, Vec3::Y);
    }

    ambient.brightness = if elev > 0.0 { 200.0 + elev * 600.0 } else { 80.0 };
}

// ── 天空顶点色更新 ────────────────────────────────────

fn sky_color_update(
    day: Res<DayCycle>,
    mut q_sky: Query<&Mesh3d, With<SkyDome>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let elev = sun_elevation(day.tick);
    let sun = sun_dir(day.tick);
    let Ok(mesh_handle) = q_sky.single() else { return };
    let Some(mesh) = meshes.get_mut(mesh_handle) else { return };

    let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return };

    let count = positions.len();
    let mut colors = Vec::with_capacity(count);

    for pos in positions {
        let dir = Vec3::new(pos[0], pos[1], pos[2]).normalize();
        let height = dir.y.clamp(0.0, 1.0); // 0=地平线 1=天顶

        // 天顶蓝 → 地平线渐变
        let sky_blue = Srgba::new(0.2, 0.4, 0.9, 1.0);
        let horizon = if elev > -0.2 {
            let t = (elev + 0.2).clamp(0.0, 1.0);
            Srgba::new(0.7 + t * 0.3, 0.7 + t * 0.25, 0.5 + t * 0.4, 1.0)
        } else {
            Srgba::new(0.05, 0.05, 0.15, 1.0)
        };

        // 太阳附近暖光斑
        let sun_dot = dir.dot(sun).max(0.0);
        let sun_glow = (sun_dot - 0.9).max(0.0) * 10.0;
        let glow = if elev > 0.0 { sun_glow.min(1.0) } else { 0.0 };

        let r = sky_blue.red * (1.0 - height) + horizon.red * height;
        let g = sky_blue.green * (1.0 - height) + horizon.green * height;
        let b = sky_blue.blue * (1.0 - height) + horizon.blue * height;

        colors.push([
            (r + glow * 0.5).min(1.0),
            (g + glow * 0.3).min(1.0),
            (b + glow * 0.1).min(1.0),
            1.0,
        ]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
}

// ── 轨道相机 ──────────────────────────────────────────

fn camera_orbit(
    mut cam: ResMut<OrbitCam>,
    mut q_camera: Query<&mut Transform, With<Camera3d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut scroll: MessageReader<MouseWheel>,
    mut motion: MessageReader<MouseMotion>,
    windows: Query<&Window>,
    keys: Res<ButtonInput<KeyCode>>,
    mut day: ResMut<DayCycle>,
) {
    let Ok(mut cam_tr) = q_camera.single_mut() else { return };
    let Ok(window) = windows.single() else { return };

    // 滚轮缩放
    for ev in scroll.read() {
        cam.distance = (cam.distance - ev.y * WORLD_HALF * 0.001).clamp(WORLD_HALF * 0.5, WORLD_HALF * 6.0);
    }

    // 鼠标左键旋转
    if mouse.pressed(MouseButton::Left) && window.cursor_position().is_some() {
        for ev in motion.read() {
            cam.yaw -= ev.delta.x * 0.005;
            cam.pitch = (cam.pitch - ev.delta.y * 0.005).clamp(-1.4, 1.4);
        }
    }

    // 键盘：WASD 旋转
    if keys.pressed(KeyCode::KeyA) { cam.yaw -= 0.03; }
    if keys.pressed(KeyCode::KeyD) { cam.yaw += 0.03; }
    if keys.pressed(KeyCode::KeyW) { cam.pitch = (cam.pitch + 0.03).clamp(-1.4, 1.4); }
    if keys.pressed(KeyCode::KeyS) { cam.pitch = (cam.pitch - 0.03).clamp(-1.4, 1.4); }
    if keys.just_pressed(KeyCode::KeyR) { cam.yaw = 0.0; cam.pitch = 0.3; cam.distance = WORLD_HALF * 3.0; }

    // 时间控制：左右箭头推进/回退时间
    if keys.pressed(KeyCode::ArrowRight) { day.tick = (day.tick + 4.0) % DAY_TICKS; }
    if keys.pressed(KeyCode::ArrowLeft) { day.tick = (day.tick - 4.0 + DAY_TICKS) % DAY_TICKS; }

    // 相机轨道中心 = 棋盘中心
    let center = Vec3::new(WORLD_HALF, 0.0, WORLD_HALF);
    let pos = center + Vec3::new(
        cam.distance * cam.pitch.cos() * cam.yaw.sin(),
        cam.distance * cam.pitch.sin(),
        cam.distance * cam.pitch.cos() * cam.yaw.cos(),
    );
    cam_tr.translation = pos;
    cam_tr.look_at(center, Vec3::Y);
}
