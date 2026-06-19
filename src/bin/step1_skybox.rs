//! Step 1: 天空盒 + 视角旋转
//! cargo run --bin step1_skybox

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::{FRAC_PI_2, TAU};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Step 1 — 天空盒".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, camera_orbit)
        .run();
}

// ── 天空盒半球 mesh ──────────────────────────────────

const SKY_RADIUS: f32 = 500.0;
const HEMI_RES: u32 = 48;

fn build_sky_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions = Vec::new();
    let mut normals = Vec::new();
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
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

// ── 轨道相机 ──────────────────────────────────────────

#[derive(Resource)]
struct OrbitCam {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl Default for OrbitCam {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: 0.25, distance: 15.0 }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 天空半球
    commands.spawn((
        Mesh3d(meshes.add(build_sky_mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            cull_mode: None,
            base_color: Color::srgb(0.35, 0.55, 0.85),
            ..default()
        })),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));

    // 透视相机
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 70.0_f32.to_radians(),
            ..default()
        }),
    ));

    commands.insert_resource(OrbitCam::default());
}

fn camera_orbit(
    mut cam: ResMut<OrbitCam>,
    mut q_camera: Query<&mut Transform, With<Camera3d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut scroll: EventReader<MouseWheel>,
    mut motion: EventReader<MouseMotion>,
    windows: Query<&Window>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut camera_transform) = q_camera.get_single_mut() else { return };
    let Ok(window) = windows.get_single() else { return };

    // 滚轮缩放
    for ev in scroll.read() {
        cam.distance = (cam.distance - ev.y * 2.0).clamp(3.0, 80.0);
    }

    // 鼠标左键拖动旋转（只在窗口聚焦时）
    let dragging = mouse.pressed(MouseButton::Left);
    let focused = window.cursor_position().is_some();

    if dragging && focused {
        for ev in motion.read() {
            cam.yaw -= ev.delta.x * 0.005;
            cam.pitch = (cam.pitch - ev.delta.y * 0.005).clamp(-1.4, 1.4);
        }
    }

    // 键盘备用：WASD 旋转
    if keys.pressed(KeyCode::KeyA) { cam.yaw -= 0.03; }
    if keys.pressed(KeyCode::KeyD) { cam.yaw += 0.03; }
    if keys.pressed(KeyCode::KeyW) { cam.pitch = (cam.pitch + 0.03).clamp(-1.4, 1.4); }
    if keys.pressed(KeyCode::KeyS) { cam.pitch = (cam.pitch - 0.03).clamp(-1.4, 1.4); }

    // R 重置视角
    if keys.just_pressed(KeyCode::KeyR) {
        cam.yaw = 0.0;
        cam.pitch = 0.25;
        cam.distance = 15.0;
    }

    let pos = Vec3::new(
        cam.distance * cam.pitch.cos() * cam.yaw.sin(),
        cam.distance * cam.pitch.sin(),
        cam.distance * cam.pitch.cos() * cam.yaw.cos(),
    );

    camera_transform.translation = pos;
    camera_transform.look_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y);
}
