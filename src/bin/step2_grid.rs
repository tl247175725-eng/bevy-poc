//! Step 2: 天空盒 + 64×64×4 线框棋盘
//! cargo run --bin step2_grid

use bevy::color::Srgba;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::{FRAC_PI_2, TAU};

// ── 世界尺度 ──────────────────────────────────────────
const GRID: u32 = 64;
const CELL: f32 = 158.0;
const WORLD_SIZE: f32 = GRID as f32 * CELL;              // 10112
const WORLD_HALF: f32 = WORLD_SIZE * 0.5;
const SKY_RADIUS: f32 = WORLD_SIZE * 1.4;
const LAYERS: u32 = 4;
const LAYER_HEIGHT: f32 = CELL;                           // 每层 158m

const HEMI_RES: u32 = 64;

// ── 时间 ──────────────────────────────────────────────
const DAY_TICKS: f32 = 2100.0;
const SUN_ANGLE: f32 = TAU / DAY_TICKS;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Step 2 — 64×64 棋盘".into(),
                    resolution: (1280., 720.).into(),
                    ..default()
                }),
                ..default()
            }),
            WireframePlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_orbit, sky_update, sun_update, grid_bounds))
        .run();
}

// ── 资源 / 组件 ───────────────────────────────────────

#[derive(Resource)]
struct OrbitCam {
    yaw: f32,
    pitch: f32,
    distance: f32,
}
impl Default for OrbitCam {
    fn default() -> Self { Self { yaw: -0.5, pitch: 0.5, distance: WORLD_SIZE * 1.2 } }
}

#[derive(Resource)]
struct DayCycle { tick: f32 }

#[derive(Component)] struct SkyDome;
#[derive(Component)] struct SunLight;
#[derive(Component)] struct GridEntity;

// ── 天空网格 ──────────────────────────────────────────

fn build_sky_mesh() -> Mesh {
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut pos = Vec::new();
    let mut nor = Vec::new();
    let mut col = Vec::new();
    let mut idx = Vec::new();
    for lat in 0..=HEMI_RES {
        let t = lat as f32 / HEMI_RES as f32 * FRAC_PI_2;
        for lon in 0..=HEMI_RES * 2 {
            let p = lon as f32 / (HEMI_RES * 2) as f32 * TAU;
            let x = SKY_RADIUS * t.cos() * p.cos();
            let y = SKY_RADIUS * t.sin();
            let z = SKY_RADIUS * t.cos() * p.sin();
            pos.push([x, y, z]);
            let l = (x*x + y*y + z*z).sqrt();
            nor.push([-x/l, -y/l, -z/l]);
            col.push([0.0, 0.0, 0.0, 1.0]);
        }
    }
    let c = HEMI_RES * 2 + 1;
    for lat in 0..HEMI_RES { for lon in 0..HEMI_RES*2 {
        let a=lat*c+lon; let b=a+1; let d=a+c; let e=d+1;
        idx.extend_from_slice(&[a,b,e,a,e,d]);
    }}
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nor);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
    m.insert_indices(Indices::U32(idx)); m
}

// ── 棋盘线框 ──────────────────────────────────────────
// 64×64 格，每格 158³，共 4 层。只画可见棱线，不画内部面。
// 结构：水平格线（每层 XZ 面）+ 垂直柱（交点处上下贯通）

fn build_grid_lines() -> Mesh {
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut idx: Vec<u32> = Vec::new();
    let n = GRID as usize;
    let s = CELL;

    // 水平格线：每层画 X 方向和 Z 方向的线
    for layer in 0..=LAYERS {
        let y = layer as f32 * LAYER_HEIGHT;
        // X 方向线（沿 Z 轴排列）
        for zi in 0..=n {
            let z = zi as f32 * s;
            let a = verts.len() as u32;
            verts.push([0.0, y, z]);
            verts.push([WORLD_SIZE, y, z]);
            idx.push(a); idx.push(a + 1);
        }
        // Z 方向线（沿 X 轴排列）
        for xi in 0..=n {
            let x = xi as f32 * s;
            let a = verts.len() as u32;
            verts.push([x, y, 0.0]);
            verts.push([x, y, WORLD_SIZE]);
            idx.push(a); idx.push(a + 1);
        }
    }

    // 垂直柱：每个交点从底层拉到顶层
    for xi in 0..=n {
        let x = xi as f32 * s;
        for zi in 0..=n {
            let z = zi as f32 * s;
            let a = verts.len() as u32;
            verts.push([x, 0.0, z]);
            verts.push([x, LAYERS as f32 * LAYER_HEIGHT, z]);
            idx.push(a); idx.push(a + 1);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_indices(Indices::U32(idx));
    mesh
}

// ── 世界边界框（地面透明底） ──────────────────────────

fn build_ground_plane() -> Mesh {
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let hw = WORLD_SIZE * 0.5;
    let y = -0.5;
    let pos = vec![
        [-hw, y, -hw], [hw, y, -hw], [hw, y, hw], [-hw, y, hw]
    ];
    let nor = vec![[0.0, 1.0, 0.0]; 4];
    let idx = vec![0u32, 1, 2, 0, 2, 3];
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nor);
    m.insert_indices(Indices::U32(idx)); m
}

// ── 启动 ──────────────────────────────────────────────

fn setup(mut cmds: Commands, mut meshes: ResMut<Assets<Mesh>>,
         mut mats: ResMut<Assets<StandardMaterial>>,
         mut config_store: ResMut<GizmoConfigStore>) {
    // 天空半球
    cmds.spawn((Mesh3d(meshes.add(build_sky_mesh())),
        MeshMaterial3d(mats.add(StandardMaterial { unlit: true, cull_mode: None, ..default() })),
        Transform::from_xyz(WORLD_HALF, 0.0, WORLD_HALF), SkyDome));

    // 地面底板（半透明，帮助看到线框深度）
    cmds.spawn((Mesh3d(meshes.add(build_ground_plane())),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.15, 0.2, 0.3), alpha_mode: AlphaMode::Blend,
            unlit: true, cull_mode: None, ..default()
        })),
        Transform::from_xyz(WORLD_HALF, 0.0, WORLD_HALF)));

    // 线框棋盘
    cmds.spawn((Mesh3d(meshes.add(build_grid_lines())),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.9, 0.9), unlit: true, ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0), Wireframe, GridEntity));

    // 太阳
    cmds.spawn((DirectionalLight { color: Color::srgb(1.0, 0.9, 0.7),
        illuminance: 10000.0, shadows_enabled: false, ..default() },
        Transform::default(), SunLight));

    cmds.insert_resource(AmbientLight { color: Color::srgb(0.4, 0.45, 0.6), brightness: 600.0 });

    // 相机：透视，俯瞰棋盘
    cmds.spawn((Camera3d::default(),
        Projection::Perspective(PerspectiveProjection { fov: 55.0_f32.to_radians(), ..default() }),
        Transform::from_xyz(WORLD_HALF, WORLD_SIZE * 0.4, WORLD_HALF * 1.8)
            .looking_at(Vec3::new(WORLD_HALF, 0.0, WORLD_HALF), Vec3::Y)));

    cmds.insert_resource(OrbitCam::default());
    cmds.insert_resource(DayCycle { tick: 800.0 }); // 从上午开始

    // 线框 gizmo 默认配置
    let cfg = config_store.config_mut::<DefaultGizmoConfigGroup>().0;
    cfg.line_width = 1.5;
    cfg.depth_bias = -0.0001;
}

// ── 天空颜色更新 ──────────────────────────────────────

fn sun_dir(tick: f32) -> Vec3 {
    let a = tick * SUN_ANGLE;
    Vec3::new(a.cos(), a.sin(), 0.0).normalize()
}
fn sun_elev(tick: f32) -> f32 { (tick * SUN_ANGLE).sin() }

fn sky_update(day: Res<DayCycle>, q: Query<&Mesh3d, With<SkyDome>>,
              mut meshes: ResMut<Assets<Mesh>>) {
    let Ok(h) = q.get_single() else { return };
    let Some(m) = meshes.get_mut(h) else { return };
    let Some(VertexAttributeValues::Float32x3(pos)) = m.attribute(Mesh::ATTRIBUTE_POSITION) else { return };
    let elev = sun_elev(day.tick);
    let sun = sun_dir(day.tick);
    let mut colors = Vec::with_capacity(pos.len());
    for p in pos {
        let dir = Vec3::new(p[0] - WORLD_HALF, p[1], p[2] - WORLD_HALF).normalize();
        let h = dir.y.clamp(0.0, 1.0);
        let sky = Srgba::new(0.18, 0.38, 0.82, 1.0);
        let hor = if elev > -0.2 {
            let t = (elev + 0.2).clamp(0.0, 1.0);
            Srgba::new(0.65 + t*0.35, 0.65 + t*0.3, 0.45 + t*0.45, 1.0)
        } else { Srgba::new(0.04, 0.04, 0.12, 1.0) };
        let dot = dir.dot(sun).max(0.0);
        let glow = if elev > 0.0 { ((dot - 0.92).max(0.0) * 12.0).min(1.0) } else { 0.0 };
        colors.push([
            (sky.red * (1.0-h) + hor.red * h + glow * 0.5).min(1.0),
            (sky.green * (1.0-h) + hor.green * h + glow * 0.3).min(1.0),
            (sky.blue * (1.0-h) + hor.blue * h + glow * 0.1).min(1.0),
            1.0,
        ]);
    }
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
}

fn sun_update(day: Res<DayCycle>, mut q: Query<(&mut DirectionalLight, &mut Transform), With<SunLight>>,
              mut amb: ResMut<AmbientLight>) {
    let elev = sun_elev(day.tick);
    let dir = sun_dir(day.tick);
    if let Ok((mut l, mut t)) = q.get_single_mut() {
        l.color = if elev > 0.0 {
            Color::srgb(1.0, 0.75 + elev * 0.25, 0.4 + elev * 0.4)
        } else { Color::srgb(0.3, 0.4, 0.7) };
        l.illuminance = if elev > 0.0 { 2000.0 + elev * 8000.0 } else { 200.0 };
        t.look_to(-dir, Vec3::Y);
    }
    amb.brightness = if elev > 0.0 { 200.0 + elev * 500.0 } else { 80.0 };
}

// ── 相机 ──────────────────────────────────────────────

fn camera_orbit(mut cam: ResMut<OrbitCam>, mut q: Query<&mut Transform, With<Camera3d>>,
                mouse: Res<ButtonInput<MouseButton>>, mut scroll: EventReader<MouseWheel>,
                mut motion: EventReader<MouseMotion>, windows: Query<&Window>,
                keys: Res<ButtonInput<KeyCode>>, mut day: ResMut<DayCycle>) {
    let Ok(mut t) = q.get_single_mut() else { return };
    let Ok(w) = windows.get_single() else { return };
    for ev in scroll.read() { cam.distance = (cam.distance - ev.y * 500.0).clamp(1000.0, 50000.0); }
    if mouse.pressed(MouseButton::Left) && w.cursor_position().is_some() {
        for ev in motion.read() {
            cam.yaw -= ev.delta.x * 0.005; cam.pitch = (cam.pitch - ev.delta.y * 0.005).clamp(-1.4, 1.4);
        }
    }
    if keys.pressed(KeyCode::KeyA) { cam.yaw -= 0.03; } if keys.pressed(KeyCode::KeyD) { cam.yaw += 0.03; }
    if keys.pressed(KeyCode::KeyW) { cam.pitch = (cam.pitch + 0.03).min(1.4); }
    if keys.pressed(KeyCode::KeyS) { cam.pitch = (cam.pitch - 0.03).max(-1.4); }
    if keys.just_pressed(KeyCode::KeyR) { cam.yaw = -0.5; cam.pitch = 0.5; cam.distance = WORLD_SIZE * 1.2; }
    if keys.pressed(KeyCode::ArrowRight) { day.tick = (day.tick + 4.0) % DAY_TICKS; }
    if keys.pressed(KeyCode::ArrowLeft) { day.tick = (day.tick - 4.0 + DAY_TICKS) % DAY_TICKS; }
    // 层切换：数字键 1-4
    let layer_keys = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4];
    for i in 0..4 { if keys.just_pressed(layer_keys[i]) {
        cam.pitch = 0.15; cam.distance = LAYER_HEIGHT * (i+1) as f32 * 3.0;
    }}
    let c = Vec3::new(WORLD_HALF, 0.0, WORLD_HALF);
    let pos = c + Vec3::new(cam.distance * cam.pitch.cos() * cam.yaw.sin(),
        cam.distance * cam.pitch.sin(), cam.distance * cam.pitch.cos() * cam.yaw.cos());
    t.translation = pos; t.look_at(c + Vec3::new(0.0, WORLD_SIZE * 0.05, 0.0), Vec3::Y);
}

// ── 网格边界调试（可选） ──────────────────────────────

fn grid_bounds(mut gizmos: Gizmos) {
    // 画世界边界框
    let _hw = WORLD_HALF;
    let corners = [
        Vec3::new(0.0, 0.0, 0.0), Vec3::new(WORLD_SIZE, 0.0, 0.0),
        Vec3::new(WORLD_SIZE, 0.0, WORLD_SIZE), Vec3::new(0.0, 0.0, WORLD_SIZE),
    ];
    for i in 0..4 {
        gizmos.line(corners[i], corners[(i+1)%4], Color::srgb(0.3, 0.7, 0.3));
    }
}
