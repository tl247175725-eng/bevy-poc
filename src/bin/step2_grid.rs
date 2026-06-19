//! Step 2: 全景天空 + 64×64×4 线框棋盘 + 完整相机操控
//! cargo run --bin step2_grid
//!
//! 操控: 左键旋转 | 右键平移 | 滚轮缩放(光标锚点) | 数字1-4切换层 | R重置 | 左右箭头调时间 | C切换相机模式(占位)

use bevy::color::Srgba;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// ── 世界 ──────────────────────────────────────────────
const GRID: u32 = 64;
const CELL: f32 = 158.0;
const WORLD_SIZE: f32 = GRID as f32 * CELL;
const WORLD_HALF: f32 = WORLD_SIZE * 0.5;
const LAYERS: u32 = 4;
const LAYER_HEIGHT: f32 = CELL;
// 天空球：覆盖最深层到最高建筑，取对角线
const MAX_DEPTH: f32 = LAYERS as f32 * LAYER_HEIGHT;
// 天空球半径必须 > 世界对角线 → √(10112² + 10112²) ≈ 14300，取 15000 有安全余量
const SKY_RADIUS: f32 = 15000.0;

const RES: u32 = 48;       // 球面分辨率
const DAY_TICKS: f32 = 2100.0;
const SUN_ANGLE: f32 = TAU / DAY_TICKS;

// ── 相机限制 ──────────────────────────────────────────
const ZOOM_MIN: f32 = 300.0;
const ZOOM_MAX: f32 = SKY_RADIUS * 0.9;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Step 2 — 64×64 棋盘".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_orbit, sky_update, sun_update, grid_bounds))
        .run();
}

// ── 资源/组件 ─────────────────────────────────────────

#[derive(Resource)]
struct Cam {
    yaw: f32,
    pitch: f32,
    dist: f32,
    focus: Vec3,          // 注视点（可以平移）
    last_mouse: Vec2,     // 上一帧光标（旋转用）
}
impl Default for Cam {
    fn default() -> Self {
        Self {
            // 3/4 等轴俯视角：从西南角上方看棋盘中心（pitch=正=俯视）
            yaw: -2.3, pitch: 0.55, dist: WORLD_SIZE * 0.8,
            focus: Vec3::new(WORLD_HALF, 0.0, WORLD_HALF),
            last_mouse: Vec2::ZERO,
        }
    }
}

#[derive(Resource)]
struct DayCycle { tick: f32 }

#[derive(Component)] struct SkyDome;
#[derive(Component)] struct SunLight;

// ── 全景天空球 ────────────────────────────────────────

fn build_sky_mesh() -> Mesh {
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut pos = Vec::new(); let mut nor = Vec::new();
    let mut col = Vec::new(); let mut idx = Vec::new();
    let rows = RES; let cols = RES * 2;
    // 完整球体：纬度 -π/2(底) → π/2(顶)
    for lat in 0..=rows {
        let t = lat as f32 / rows as f32 * PI - FRAC_PI_2;
        for lon in 0..=cols {
            let p = lon as f32 / cols as f32 * TAU;
            let x = SKY_RADIUS * t.cos() * p.cos();
            let y = SKY_RADIUS * t.sin();
            let z = SKY_RADIUS * t.cos() * p.sin();
            pos.push([x, y, z]);
            let l = (x*x + y*y + z*z).sqrt();
            nor.push([-x/l, -y/l, -z/l]);
            col.push([0.0, 0.0, 0.0, 1.0]);
        }
    }
    let c = cols + 1;
    for lat in 0..rows { for lon in 0..cols {
        let a=lat*c+lon; let b=a+1; let d=a+c; let e=d+1;
        idx.extend_from_slice(&[a,b,e,a,e,d]);
    }}
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nor);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
    m.insert_indices(Indices::U32(idx)); m
}

// ── 线框 ──────────────────────────────────────────────

fn build_grid_lines() -> Mesh {
    let mut v = Vec::new(); let mut i = Vec::new();
    let n = GRID as usize;
    for layer in 0..=LAYERS {
        let y = layer as f32 * LAYER_HEIGHT;
        for zi in 0..=n {
            let z=zi as f32*CELL; let a=v.len() as u32;
            v.push([0.0,y,z]); v.push([WORLD_SIZE,y,z]); i.push(a); i.push(a+1);
        }
        for xi in 0..=n {
            let x=xi as f32*CELL; let a=v.len() as u32;
            v.push([x,y,0.0]); v.push([x,y,WORLD_SIZE]); i.push(a); i.push(a+1);
        }
    }
    for xi in 0..=n { let x=xi as f32*CELL;
        for zi in 0..=n { let z=zi as f32*CELL; let a=v.len() as u32;
            v.push([x,0.0,z]); v.push([x,LAYERS as f32*LAYER_HEIGHT,z]); i.push(a); i.push(a+1);
        }
    }
    let mut m = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, v); m.insert_indices(Indices::U32(i)); m
}

// ── 启动 ──────────────────────────────────────────────

fn setup(mut cmds: Commands, mut meshes: ResMut<Assets<Mesh>>,
         mut mats: ResMut<Assets<StandardMaterial>>) {
    // 全景天空球
    cmds.spawn((Mesh3d(meshes.add(build_sky_mesh())),
        MeshMaterial3d(mats.add(StandardMaterial { unlit: true, cull_mode: None, ..default() })),
        Transform::from_xyz(WORLD_HALF, 0.0, WORLD_HALF), SkyDome));

    // 线框
    cmds.spawn((Mesh3d(meshes.add(build_grid_lines())),
        MeshMaterial3d(mats.add(StandardMaterial { base_color: Color::srgb(0.85, 0.85, 0.85), unlit: true, ..default() })),
        Transform::from_xyz(0.0, 0.0, 0.0)));

    // 太阳
    cmds.spawn((DirectionalLight { color: Color::srgb(1.0, 0.9, 0.7),
        illuminance: 8000.0, shadows_enabled: false, ..default() }, Transform::default(), SunLight));
    cmds.insert_resource(AmbientLight { color: Color::srgb(0.35, 0.4, 0.55), brightness: 500.0 });

    // 相机
    cmds.spawn((Camera3d::default(),
        Projection::Perspective(PerspectiveProjection { fov: 50.0_f32.to_radians(), ..default() }),
        Transform::from_xyz(WORLD_HALF, 4000.0, WORLD_HALF * 1.6)
            .looking_at(Vec3::new(WORLD_HALF, 0.0, WORLD_HALF), Vec3::Y)));

    cmds.insert_resource(Cam::default());
    cmds.insert_resource(DayCycle { tick: 800.0 });
}

// ── 天空 ──────────────────────────────────────────────

fn sun_dir(tick: f32) -> Vec3 {
    let a = tick * SUN_ANGLE;
    Vec3::new(a.cos(), a.sin(), 0.0) // 简化为 XZ 平面上的圆（以后补倾角）
}
fn sun_elev(tick: f32) -> f32 { (tick * SUN_ANGLE).sin() }

fn sky_update(day: Res<DayCycle>, q: Query<&Mesh3d, With<SkyDome>>,
              mut meshes: ResMut<Assets<Mesh>>) {
    let Ok(h) = q.get_single() else { return };
    let Some(m) = meshes.get_mut(h) else { return };
    let Some(VertexAttributeValues::Float32x3(pos)) = m.attribute(Mesh::ATTRIBUTE_POSITION) else { return };
    let elev = sun_elev(day.tick); let sun = sun_dir(day.tick);
    let mut colors = Vec::with_capacity(pos.len());
    for p in pos {
        let dir = Vec3::new(p[0] - WORLD_HALF, p[1], p[2] - WORLD_HALF).normalize();
        let h = dir.y.clamp(0.0, 1.0);
        let sky = Srgba::new(0.15, 0.32, 0.78, 1.0);
        let hor = if elev > -0.2 {
            let t = (elev + 0.2).clamp(0.0, 1.0);
            Srgba::new(0.6 + t*0.4, 0.6 + t*0.35, 0.4 + t*0.5, 1.0)
        } else { Srgba::new(0.03, 0.03, 0.1, 1.0) };
        let dot = dir.dot(sun).max(0.0);
        let glow = if elev > 0.0 { ((dot - 0.9).max(0.0) * 10.0).min(1.0) } else { 0.0 };
        // 全方向统一天空色：天顶蓝渐变到地平线白
        colors.push([
            (sky.red * (1.0-h) + hor.red * h + glow * 0.5).min(1.0),
            (sky.green * (1.0-h) + hor.green * h + glow * 0.3).min(1.0),
            (sky.blue * (1.0-h) + hor.blue * h + glow * 0.1).min(1.0), 1.0,
        ]);
    }
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
}

fn sun_update(day: Res<DayCycle>, mut q: Query<(&mut DirectionalLight, &mut Transform), With<SunLight>>,
              mut amb: ResMut<AmbientLight>) {
    let elev = sun_elev(day.tick); let dir = sun_dir(day.tick);
    if let Ok((mut l, mut t)) = q.get_single_mut() {
        l.color = if elev > 0.0 { Color::srgb(1.0, 0.7+elev*0.3, 0.35+elev*0.45) }
                  else { Color::srgb(0.3, 0.35, 0.65) };
        l.illuminance = if elev > 0.0 { 1500.0 + elev * 6500.0 } else { 150.0 };
        t.look_to(-dir, Vec3::Y);
    }
    amb.brightness = if elev > 0.0 { 150.0 + elev * 400.0 } else { 60.0 };
}

// ── 相机 ──────────────────────────────────────────────

fn camera_orbit(
    mut cam: ResMut<Cam>,
    mut q: Query<&mut Transform, With<Camera3d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut scroll: EventReader<MouseWheel>,
    mut motion: EventReader<MouseMotion>,
    windows: Query<&Window>,
    keys: Res<ButtonInput<KeyCode>>,
    mut day: ResMut<DayCycle>,
) {
    let Ok(mut t) = q.get_single_mut() else { return };
    let Ok(w) = windows.get_single() else { return };
    let Some(_cursor) = w.cursor_position() else { return };

    let mut rot_changed = false;
    let mut pan_changed = false;

    // 鼠标 delta（用于旋转和平移）
    let mut dx = 0.0f32; let mut dy = 0.0f32;
    for ev in motion.read() { dx += ev.delta.x; dy += ev.delta.y; }

    // ── 左键：旋转视角 ──
    if mouse.pressed(MouseButton::Left) {
        cam.yaw -= dx * 0.004;
        cam.pitch = cam.pitch - dy * 0.004;
        rot_changed = true;
    }

    // ── 右键：平移注视点 ──
    if mouse.pressed(MouseButton::Right) {
        // 平移速度 = 距离 × 比例（放大近 → 慢移，缩小远 → 快移）
        let pan_speed = cam.dist * 0.00015;
        // 相机右方向
        let forward = (cam.focus - cam_pos(&cam)).normalize();
        let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right = forward_xz.cross(Vec3::Y).normalize_or_zero();
        cam.focus += right * (-dx) * pan_speed;
        cam.focus += forward_xz * dy * pan_speed;
        // 限制焦点在世界范围内
        cam.focus.x = cam.focus.x.clamp(-WORLD_SIZE*0.1, WORLD_SIZE*1.1);
        cam.focus.z = cam.focus.z.clamp(-WORLD_SIZE*0.1, WORLD_SIZE*1.1);
        cam.focus.y = cam.focus.y.clamp(-MAX_DEPTH, WORLD_SIZE * 0.3);
        pan_changed = true;
    }

    // ── 滚轮：缩放 —— 保持焦点不动 ──
    for ev in scroll.read() {
        let dir = Vec3::new(
            cam.pitch.cos() * cam.yaw.sin(),
            cam.pitch.sin(),
            cam.pitch.cos() * cam.yaw.cos(),
        );
        let old_pos = cam.focus + dir * cam.dist;
        cam.dist = (cam.dist - ev.y * cam.dist * 0.1).clamp(ZOOM_MIN, ZOOM_MAX);
        let new_pos = cam.focus + dir * cam.dist;
        cam.focus += old_pos - new_pos; // 补偿相机位移，焦点不动
    }

    // ── WASD：微调旋转 ──
    if keys.pressed(KeyCode::KeyA) { cam.yaw -= 0.03; rot_changed = true; }
    if keys.pressed(KeyCode::KeyD) { cam.yaw += 0.03; rot_changed = true; }
    if keys.pressed(KeyCode::KeyW) { cam.pitch = cam.pitch + 0.03; rot_changed = true; }
    if keys.pressed(KeyCode::KeyS) { cam.pitch = cam.pitch - 0.03; rot_changed = true; }

    // 焦点平移快捷键：方向键
    let pan_k = cam.dist * 0.01;
    let fwd = {
        let f = (cam.focus - cam_pos(&cam)).normalize();
        Vec3::new(f.x, 0.0, f.z).normalize_or_zero()
    };
    let rgt = fwd.cross(Vec3::Y).normalize_or_zero();
    if keys.pressed(KeyCode::KeyQ) { cam.focus += rgt * pan_k; }
    if keys.pressed(KeyCode::KeyE) { cam.focus -= rgt * pan_k; }

    // 层切换
    let lk = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4];
    for i in 0..4 { if keys.just_pressed(lk[i]) {
        cam.pitch = -0.2; cam.dist = LAYER_HEIGHT * (i+1) as f32 * 2.5;
        cam.focus = Vec3::new(WORLD_HALF, LAYER_HEIGHT * (i+1) as f32 * 0.5, WORLD_HALF);
    }}

    // 重置
    if keys.just_pressed(KeyCode::KeyR) { *cam = Cam::default(); }

    // 时间
    if keys.pressed(KeyCode::ArrowRight) { day.tick = (day.tick + 4.0) % DAY_TICKS; }
    if keys.pressed(KeyCode::ArrowLeft) { day.tick = (day.tick - 4.0 + DAY_TICKS) % DAY_TICKS; }

    // ── 更新相机 —— 软刹车不超出天空球 ──
    let sky_center = Vec3::new(WORLD_HALF, 0.0, WORLD_HALF);

    // 计算期望的相机位置
    let dir = Vec3::new(
        cam.pitch.cos() * cam.yaw.sin(),
        cam.pitch.sin(),
        cam.pitch.cos() * cam.yaw.cos(),
    );
    let desired_pos = cam.focus + dir * cam.dist;

    // 检查是否超出天空球
    let to_center = desired_pos - sky_center;
    let dist_from_center = to_center.length();
    let max_allowed = SKY_RADIUS * 0.92;

    let actual_pos = if dist_from_center > max_allowed {
        // 投影到球面上
        let on_sphere = sky_center + to_center.normalize_or_zero() * max_allowed;
        // 软刹车：越靠近边界减速越明显
        let overshoot = (dist_from_center - max_allowed) / max_allowed;
        let t_factor = (1.0 - (overshoot * 3.0).clamp(0.0, 0.95)).max(0.05);
        desired_pos.lerp(on_sphere, 1.0 - t_factor)
    } else {
        desired_pos
    };

    t.translation = actual_pos;
    t.look_at(cam.focus, Vec3::Y);
}

fn cam_pos(cam: &Cam) -> Vec3 {
    cam.focus + Vec3::new(
        cam.dist * cam.pitch.cos() * cam.yaw.sin(),
        cam.dist * cam.pitch.sin(),
        cam.dist * cam.pitch.cos() * cam.yaw.cos(),
    )
}

// ── 世界边框 ──────────────────────────────────────────

fn grid_bounds(mut gizmos: Gizmos) {
    let c = [Vec3::new(0.,0.,0.), Vec3::new(WORLD_SIZE,0.,0.),
             Vec3::new(WORLD_SIZE,0.,WORLD_SIZE), Vec3::new(0.,0.,WORLD_SIZE)];
    for i in 0..4 { gizmos.line(c[i], c[(i+1)%4], Color::srgb(0.2, 0.65, 0.2)); }
}
