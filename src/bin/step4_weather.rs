//! Step 4: 天气公理可视化 —— 温度/湿度/云/降水 彩色热力图
//! cargo run --bin step4_weather

use bevy::color::Srgba;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_poc::weather::WeatherCell;
use std::f32::consts::TAU;

// ── 世界 ──────────────────────────────────────────────
const GRID: u32 = 64; const CELL: f32 = 158.0;
const WS: f32 = GRID as f32 * CELL; const WH: f32 = WS * 0.5;
const LAYERS: u32 = 4; const LH: f32 = CELL;
const SKY_R: f32 = 15000.0;
const DAY_TICKS: f32 = 2100.0;
const TICK_TO_ANGLE: f32 = TAU / DAY_TICKS;

// 显示模式
#[derive(Resource, PartialEq, Clone, Copy)]
enum VisMode { Temperature, Humidity, Cloud, Precipitation }

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Step 4 — 天气".into(), resolution: (1280.,720.).into(), ..default() }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(VisMode::Temperature)
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, weather_tick, update_grid_colors, mode_switch))
        .run();
}

// ── 资源/组件 ─────────────────────────────────────────

#[derive(Resource)] struct OC { yaw:f32, pitch:f32, radius:f32, focus:Vec3 }
#[derive(Resource)] struct DayCycle { tick: f32 }
#[derive(Resource)] struct WeatherGrid { cells: Vec<WeatherCell>, width: u32, height: u32 }
#[derive(Component)] struct GridPlane;

// ── 简化地形 ──────────────────────────────────────────

fn is_water(x:u32, y:u32) -> bool {
    let cx = GRID/2; let cy = GRID/2;
    let dx = x as i32 - cx as i32; let dy = y as i32 - cy as i32;
    (dx*dx + dy*dy) < 200 // 中心湖区
}
fn elevation(x:u32, y:u32) -> f32 {
    let cx = GRID/2; let cy = GRID/2;
    let dx = x as i32 - cx as i32; let dy = y as i32 - cy as i32;
    let dist = ((dx*dx + dy*dy) as f32).sqrt();
    // 中央低洼湖区 → 外围逐渐升高 → 边缘崖壁
    if dist < 14.0 { -100.0 }                           // 湖底
    else if dist < 18.0 { (dist - 14.0) * 25.0 }        // 湖岸坡
    else { 100.0 + (dist - 18.0) * 50.0 }               // 向外升高
}

fn lift(x:u32, y:u32) -> f32 {
    let e = elevation(x, y);
    if e > 3000.0 { 0.8 } else if e > 1000.0 { 0.3 } else { 0.1 }
}

// ── 天气 tick ─────────────────────────────────────────

fn weather_tick(
    day: Res<DayCycle>,
    mut grid: ResMut<WeatherGrid>,
    mut tick_count: Local<u64>,
) {
    *tick_count += 1;
    let se = (day.tick * TICK_TO_ANGLE).sin();
    let w = grid.width; let h = grid.height;

    let old_temps: Vec<f32> = grid.cells.iter().map(|c| c.temperature).collect();
    let old_vapors: Vec<f32> = grid.cells.iter().map(|c| c.vapor_pressure).collect();

    for y in 0..h { for x in 0..w {
        let idx = (y * w + x) as usize;
        let elev_m = elevation(x, y);
        let baseline_temp = bevy_poc::weather::baseline_temperature(0.3, se, elev_m);
        let water = is_water(x, y);
        let water_temp = baseline_temp;

        let mut nt = Vec::with_capacity(4);
        let mut nv = Vec::with_capacity(4);
        for (dx, dy) in &[(1i32,0),(-1,0),(0,1),(0,-1)] {
            let nx = x as i32 + dx; let ny = y as i32 + dy;
            if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                let ni = (ny as u32 * w + nx as u32) as usize;
                nt.push((*dx as f32, *dy as f32, old_temps[ni]));
                nv.push((*dx as f32, *dy as f32, old_vapors[ni]));
            }
        }

        bevy_poc::weather::tick_weather_cell(
            &mut grid.cells[idx], baseline_temp,
            &nt, &nv, water, water_temp, lift(x, y), 0.0, 0.0,
        );
    }}
}

// ── 网格平面 + 颜色更新 ───────────────────────────────

fn grid_plane_mesh() -> Mesh {
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut p = vec![]; let mut c = vec![]; let mut idx = vec![];
    let n = GRID;
    for y in 0..=n { for x in 0..=n {
        p.push([x as f32*CELL, 0., y as f32*CELL]);
        c.push([0.,0.,0.,1.]);
    }}
    let stride = n + 1;
    for y in 0..n { for x in 0..n {
        let a = y*stride + x; let b = a+1; let d = a+stride; let e = d+1;
        idx.extend_from_slice(&[a,b,e,a,e,d]);
    }}
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, p);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
    m.insert_indices(Indices::U32(idx)); m
}

fn update_grid_colors(
    grid: Res<WeatherGrid>, mode: Res<VisMode>,
    q: Query<&Mesh3d, With<GridPlane>>, mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(h) = q.get_single() else { return };
    let Some(m) = meshes.get_mut(h) else { return };
    let Some(VertexAttributeValues::Float32x3(pos)) = m.attribute(Mesh::ATTRIBUTE_POSITION) else { return };
    let mut colors = Vec::with_capacity(pos.len());
    for p in pos {
        let gx = (p[0] / CELL) as u32; let gy = (p[2] / CELL) as u32;
        let idx = ((gy * grid.width + gx) as usize).min(grid.cells.len()-1);
        let cell = &grid.cells[idx];
        let c = match *mode {
            VisMode::Temperature => {
                let t = ((cell.temperature - 270.) / 40.).clamp(0., 1.);
                Srgba::new(t, 0.1, 1.-t, 0.85)
            }
            VisMode::Humidity => {
                let es = bevy_poc::weather::saturation_vapor_pressure(cell.temperature, false);
                let h = (cell.vapor_pressure / es).clamp(0., 1.5);
                Srgba::new(0.1, 0.3 + h*0.4, 0.8, 0.85)
            }
            VisMode::Cloud => {
                Srgba::new(cell.cloud_cover, cell.cloud_cover, cell.cloud_cover*0.3+0.2, 0.85)
            }
            VisMode::Precipitation => {
                let r = (cell.precipitation * 20.).min(1.);
                Srgba::new(0.1+r*0.9, 0.1, 0.5+r*0.5, 0.85)
            }
        };
        colors.push([c.red, c.green, c.blue, c.alpha]);
    }
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
}

fn mode_switch(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<VisMode>) {
    if keys.just_pressed(KeyCode::Digit1) { *mode = VisMode::Temperature; }
    if keys.just_pressed(KeyCode::Digit2) { *mode = VisMode::Humidity; }
    if keys.just_pressed(KeyCode::Digit3) { *mode = VisMode::Cloud; }
    if keys.just_pressed(KeyCode::Digit4) { *mode = VisMode::Precipitation; }
}

// ── 线框棋盘 ──────────────────────────────────────────

fn grid_lines_mesh() -> Mesh {
    let mut v=vec![]; let mut i=vec![]; let n=GRID as usize;
    for zi in 0..=n { let z=zi as f32*CELL; let a=v.len() as u32; v.push([0.,0.,z]); v.push([WS,0.,z]); i.push(a); i.push(a+1); }
    for xi in 0..=n { let x=xi as f32*CELL; let a=v.len() as u32; v.push([x,0.,0.]); v.push([x,0.,WS]); i.push(a); i.push(a+1); }
    let mut m=Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, v); m.insert_indices(Indices::U32(i)); m
}

// ── 启动 ──────────────────────────────────────────────

fn setup(mut c:Commands, mut meshes:ResMut<Assets<Mesh>>, mut mats:ResMut<Assets<StandardMaterial>>){
    // 天气热力图平面
    c.spawn((Mesh3d(meshes.add(grid_plane_mesh())),
        MeshMaterial3d(mats.add(StandardMaterial{unlit:true,cull_mode:None,alpha_mode:AlphaMode::Blend,..default()})),
        GridPlane));
    // 线框
    c.spawn((Mesh3d(meshes.add(grid_lines_mesh())),
        MeshMaterial3d(mats.add(StandardMaterial{base_color:Color::srgb(0.3,0.3,0.3),unlit:true,..default()})),
        Transform::from_xyz(0.,0.1,0.)));
    c.spawn((Camera3d::default(),Projection::Perspective(PerspectiveProjection{fov:50_f32.to_radians(),..default()})));
    c.insert_resource(OC{yaw:-2.3,pitch:0.55,radius:WS*0.8,focus:Vec3::new(WH,0.,WH)});
    c.insert_resource(DayCycle{tick:800.});
    c.insert_resource(WeatherGrid{cells:vec![WeatherCell::default();4100],width:GRID,height:GRID});
}

// ── 相机（同 step2） ──────────────────────────────────

fn orbit_camera(
    mut oc: ResMut<OC>, mut qc: Query<&mut Transform, With<Camera3d>>,
    btn: Res<ButtonInput<MouseButton>>, mut scr: EventReader<MouseWheel>,
    mut motion: EventReader<bevy::input::mouse::MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>, wq: Query<&Window>, mut day: ResMut<DayCycle>,
) {
    let Ok(mut t) = qc.get_single_mut() else { return };
    let Ok(_w) = wq.get_single() else { return };
    let mut dx=0.0f32; let mut dy=0.0f32;
    for ev in motion.read() { dx+=ev.delta.x; dy+=ev.delta.y; }
    for ev in scr.read() { oc.radius=(oc.radius - ev.y*oc.radius*0.1).clamp(300.,SKY_R*0.85); }
    if btn.pressed(MouseButton::Left) { oc.yaw-=dx*0.003; oc.pitch=(oc.pitch+dy*0.003).clamp(-1.5,1.5); }
    if btn.pressed(MouseButton::Right) {
        let fwd=(oc.focus - spherical(oc.yaw,oc.pitch,oc.radius)).normalize();
        let r=Vec3::new(fwd.x,0.,fwd.z).normalize_or_zero().cross(Vec3::Y); let u=r.cross(fwd); let sp=oc.radius*0.0003;
        oc.focus+=r*(-dx*sp)+u*(-dy*sp);
    }
    if keys.pressed(KeyCode::KeyA){oc.yaw-=0.03;} if keys.pressed(KeyCode::KeyD){oc.yaw+=0.03;}
    if keys.pressed(KeyCode::KeyW){oc.pitch=(oc.pitch+0.03).min(1.5);}
    if keys.pressed(KeyCode::KeyS){oc.pitch=(oc.pitch-0.03).max(-1.5);}
    if keys.just_pressed(KeyCode::KeyR){oc.yaw=-2.3;oc.pitch=0.55;oc.radius=WS*0.8;oc.focus=Vec3::new(WH,0.,WH);}
    if keys.pressed(KeyCode::ArrowRight){day.tick=(day.tick+4.)%DAY_TICKS;}
    if keys.pressed(KeyCode::ArrowLeft){day.tick=(day.tick-4.+DAY_TICKS)%DAY_TICKS;}
    let mut cp=oc.focus+spherical(oc.yaw,oc.pitch,oc.radius);
    let sc=Vec3::new(WH,0.,WH); let to=cp-sc;
    if to.length()>SKY_R*0.92{cp=sc+to.normalize_or_zero()*SKY_R*0.92;oc.radius=(cp-oc.focus).length().max(300.);}
    t.translation=cp; t.look_at(oc.focus,Vec3::Y);
}
fn spherical(yaw:f32,pitch:f32,r:f32)->Vec3{Vec3::new(r*pitch.cos()*yaw.sin(),r*pitch.sin(),r*pitch.cos()*yaw.cos())}
