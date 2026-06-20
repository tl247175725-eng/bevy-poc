//! Step 2: 全景天空 + 64x64x4 线框棋盘
//! cargo run --bin step2_grid

use bevy::asset::RenderAssetUsages;
use bevy::color::Srgba;
use bevy::input::mouse::MouseWheel;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::window::WindowResolution;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// ── 世界常量 ───────────────────────────────────────────
const GRID: u32 = 64;
const CELL: f32 = 158.0;
const WORLD_SIZE: f32 = GRID as f32 * CELL;
const WH: f32 = WORLD_SIZE * 0.5;
const LAYERS: u32 = 4;
const LAYER_H: f32 = CELL;
const SKY_R: f32 = 15000.0;
const DAY_TICKS: f32 = 2100.0;
const SUN_A: f32 = TAU / DAY_TICKS;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Step 2 — 棋盘".into(), resolution: WindowResolution::new(1280, 720), ..default() }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, sky_tick, sun_tick, bounds_gizmo))
        .run();
}

// ── 轨道相机 ── 标准球坐标方案，只加天空球边界

#[derive(Resource)]
struct OC { yaw: f32, pitch: f32, radius: f32, focus: Vec3 }

fn orbit_camera(
    mut oc: ResMut<OC>,
    mut qc: Query<&mut Transform, With<Camera3d>>,
    btn: Res<ButtonInput<MouseButton>>,
    mut scr: MessageReader<MouseWheel>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>,
    wq: Query<&Window>,
    mut day: ResMut<DayCycle>,
) {
    let Ok(mut t) = qc.single_mut() else { return };
    let Ok(w) = wq.single() else { return };
    let Some(cursor) = w.cursor_position() else { return };

    // 鼠标 delta
    let mut dx = 0.0f32; let mut dy = 0.0f32;
    for ev in motion.read() { dx += ev.delta.x; dy += ev.delta.y; }

    // ── 滚轮缩放 ──
    for ev in scr.read() {
        oc.radius = (oc.radius - ev.y * oc.radius * 0.1).clamp(300.0, SKY_R * 0.85);
    }

    // ── 左键：旋转 yaw/pitch ──
    if btn.pressed(MouseButton::Left) {
        oc.yaw -= dx * 0.003;
        oc.pitch = (oc.pitch + dy * 0.003).clamp(-1.5, 1.5);
    }

    // ── 右键：平移 focus ──
    if btn.pressed(MouseButton::Right) {
        let fwd = (oc.focus - spherical_to_cartesian(oc.yaw, oc.pitch, oc.radius)).normalize();
        let right = Vec3::new(fwd.x, 0.0, fwd.z).normalize_or_zero().cross(Vec3::Y);
        let up = right.cross(fwd);
        let speed = oc.radius * 0.0003;
        oc.focus += right * (-dx * speed) + up * (-dy * speed);
    }

    // ── WASD 旋转 ──
    if keys.pressed(KeyCode::KeyA) { oc.yaw -= 0.03; }
    if keys.pressed(KeyCode::KeyD) { oc.yaw += 0.03; }
    if keys.pressed(KeyCode::KeyW) { oc.pitch = (oc.pitch + 0.03).min(1.5); }
    if keys.pressed(KeyCode::KeyS) { oc.pitch = (oc.pitch - 0.03).max(-1.5); }

    // ── 层切换 / 重置 ──
    let lk = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4];
    for i in 0..4 { if keys.just_pressed(lk[i]) {
        oc.focus = Vec3::new(WH, LAYER_H * (i+1) as f32 * 0.5, WH);
        oc.pitch = 0.4; oc.radius = LAYER_H * (i+1) as f32 * 3.0;
    }}
    if keys.just_pressed(KeyCode::KeyR) {
        oc.yaw = -2.3; oc.pitch = 0.55; oc.radius = WORLD_SIZE * 0.8;
        oc.focus = Vec3::new(WH, 0.0, WH);
    }

    // ── 时间 ──
    if keys.pressed(KeyCode::ArrowRight) { day.tick = (day.tick + 4.0) % DAY_TICKS; }
    if keys.pressed(KeyCode::ArrowLeft) { day.tick = (day.tick - 4.0 + DAY_TICKS) % DAY_TICKS; }

    // ── 应用相机位置（天空球边界） ──
    let mut cam_pos = oc.focus + spherical_to_cartesian(oc.yaw, oc.pitch, oc.radius);
    let sc = Vec3::new(WH, 0.0, WH);
    let to_sc = cam_pos - sc;
    if to_sc.length() > SKY_R * 0.92 {
        cam_pos = sc + to_sc.normalize_or_zero() * SKY_R * 0.92;
        // 同步缩减 radius 使后续操作合理
        oc.radius = (cam_pos - oc.focus).length().max(300.0);
    }

    t.translation = cam_pos;
    t.look_at(oc.focus, Vec3::Y);
}

fn spherical_to_cartesian(yaw: f32, pitch: f32, r: f32) -> Vec3 {
    Vec3::new(r * pitch.cos() * yaw.sin(), r * pitch.sin(), r * pitch.cos() * yaw.cos())
}

// ── 资源 ───────────────────────────────────────────────

#[derive(Resource)] struct DayCycle { tick: f32 }
#[derive(Component)] struct Sky;

// ── 天空球 mesh ────────────────────────────────────────

fn sky_mesh() -> Mesh {
    let (res, r) = (48u32, SKY_R);
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut p=vec![]; let mut n=vec![]; let mut c=vec![]; let mut idx=vec![];
    for lat in 0..=res {
        let t = lat as f32/res as f32*PI - FRAC_PI_2;
        for lon in 0..=res*2 {
            let phi = lon as f32/(res*2) as f32*TAU;
            let x=r*t.cos()*phi.cos(); let y=r*t.sin(); let z=r*t.cos()*phi.sin();
            p.push([x,y,z]); let l=(x*x+y*y+z*z).sqrt(); n.push([-x/l,-y/l,-z/l]);
            c.push([0.0,0.0,0.0,1.0]);
        }
    }
    let cc=res*2+1;
    for lat in 0..res { for lon in 0..res*2 {
        let a=lat*cc+lon; idx.extend_from_slice(&[a,a+1,a+cc+1,a,a+cc+1,a+cc]);
    }}
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,p); m.insert_attribute(Mesh::ATTRIBUTE_NORMAL,n);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR,c); m.insert_indices(Indices::U32(idx)); m
}

// ── 线框棋盘 ───────────────────────────────────────────

fn grid_mesh() -> Mesh {
    let mut v=vec![]; let mut i=vec![]; let n=GRID as usize;
    for l in 0..=LAYERS { let y=l as f32*LAYER_H;
        for zi in 0..=n { let z=zi as f32*CELL; let a=v.len() as u32; v.push([0.,y,z]); v.push([WORLD_SIZE,y,z]); i.push(a); i.push(a+1); }
        for xi in 0..=n { let x=xi as f32*CELL; let a=v.len() as u32; v.push([x,y,0.]); v.push([x,y,WORLD_SIZE]); i.push(a); i.push(a+1); }
    }
    for xi in 0..=n { let x=xi as f32*CELL;
        for zi in 0..=n { let z=zi as f32*CELL; let a=v.len() as u32; v.push([x,0.,z]); v.push([x,LAYERS as f32*LAYER_H,z]); i.push(a); i.push(a+1); }
    }
    let mut m = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,v); m.insert_indices(Indices::U32(i)); m
}

// ── 启动 ───────────────────────────────────────────────

fn setup(mut c: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    c.spawn((Mesh3d(meshes.add(sky_mesh())), MeshMaterial3d(mats.add(StandardMaterial{unlit:true,cull_mode:None,..default()})),
        Transform::from_xyz(WH,0.,WH), Sky));
    c.spawn((Mesh3d(meshes.add(grid_mesh())), MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(0.85,0.85,0.85),unlit:true,..default()})), Transform::default()));
    c.spawn((DirectionalLight{color:Color::srgb(1.,0.9,0.7),illuminance:8000.,shadows_enabled:false,..default()}, Transform::default()));
    c.insert_resource(GlobalAmbientLight{color:Color::srgb(0.35,0.4,0.55),brightness:500.,affects_lightmapped_meshes:false});
    c.spawn((Camera3d::default(), Projection::Perspective(PerspectiveProjection{fov:50_f32.to_radians(),..default()})));
    c.insert_resource(OC{yaw:-2.3,pitch:0.55,radius:WORLD_SIZE*0.8,focus:Vec3::new(WH,0.,WH)});
    c.insert_resource(DayCycle{tick:800.});
}

// ── 天空更新 ───────────────────────────────────────────

fn sun_elev(t: f32) -> f32 { (t * SUN_A).sin() }
fn sun_dir(t: f32) -> Vec3 { let a = t * SUN_A; Vec3::new(a.cos(), a.sin(), 0.0) }

fn sky_tick(day: Res<DayCycle>, q: Query<&Mesh3d,With<Sky>>, mut meshes: ResMut<Assets<Mesh>>) {
    let Ok(h)=q.single() else{return}; let Some(m)=meshes.get_mut(h) else{return};
    let Some(VertexAttributeValues::Float32x3(pos))=m.attribute(Mesh::ATTRIBUTE_POSITION) else{return};
    let elev=sun_elev(day.tick); let sun=sun_dir(day.tick);
    let mut colors=Vec::with_capacity(pos.len());
    for p in pos {
        let dir=Vec3::new(p[0]-WH,p[1],p[2]-WH).normalize();
        let h=dir.y.clamp(0.,1.);
        let sky=Srgba::new(0.15,0.32,0.78,1.);
        let hor=if elev>-0.2{let t=(elev+0.2).clamp(0.,1.);Srgba::new(0.6+t*0.4,0.6+t*0.35,0.4+t*0.5,1.)}
                else{Srgba::new(0.03,0.03,0.1,1.)};
        let dot=dir.dot(sun).max(0.); let glow=if elev>0.{((dot-0.9).max(0.)*10.).min(1.)}else{0.};
        colors.push([(sky.red*(1.-h)+hor.red*h+glow*0.5).min(1.),(sky.green*(1.-h)+hor.green*h+glow*0.3).min(1.),
            (sky.blue*(1.-h)+hor.blue*h+glow*0.1).min(1.),1.]);
    }
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR,colors);
}

fn sun_tick(day: Res<DayCycle>, mut q: Query<(&mut DirectionalLight,&mut Transform)>, mut amb: ResMut<GlobalAmbientLight>) {
    let elev=sun_elev(day.tick); let dir=sun_dir(day.tick);
    if let Ok((mut l,mut t))=q.single_mut() {
        l.color=if elev>0.{Color::srgb(1.,0.7+elev*0.3,0.35+elev*0.45)}else{Color::srgb(0.3,0.35,0.65)};
        l.illuminance=if elev>0.{1500.+elev*6500.}else{150.}; t.look_to(-dir,Vec3::Y);
    }
    amb.brightness=if elev>0.{150.+elev*400.}else{60.};
}

fn bounds_gizmo(mut gizmos: Gizmos) {
    let c=[Vec3::new(0.,0.,0.),Vec3::new(WORLD_SIZE,0.,0.),Vec3::new(WORLD_SIZE,0.,WORLD_SIZE),Vec3::new(0.,0.,WORLD_SIZE)];
    for i in 0..4{gizmos.line(c[i],c[(i+1)%4],Color::srgb(0.2,0.65,0.2));}
}
