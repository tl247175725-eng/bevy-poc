//! Step 3: 天空 + 棋盘 + 太阳/月亮/星星
//! cargo run --bin step3_skysun

use bevy::color::Srgba;
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// ── 自定义材质 ─────────────────────────────────────────

#[derive(ShaderType, Debug, Clone)]
struct SunUniforms {
    color_center: LinearRgba,
    color_edge: LinearRgba,
    emissive_intensity: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SunMaterial {
    #[uniform(0)] uniforms: SunUniforms,
}
impl Material for SunMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/sun_material.wgsl".into() }
}

#[derive(ShaderType, Debug, Clone)]
struct MoonUniforms {
    base_color: LinearRgba,
    crater_color: LinearRgba,
    emissive_intensity: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct MoonMaterial {
    #[uniform(0)] uniforms: MoonUniforms,
}
impl Material for MoonMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/moon_material.wgsl".into() }
}

// ── 世界 ──────────────────────────────────────────────
const GRID: u32 = 64; const CELL: f32 = 158.0;
const WS: f32 = GRID as f32 * CELL; const WH: f32 = WS * 0.5;
const LAYERS: u32 = 4; const LH: f32 = CELL;
const SKY_R: f32 = 15000.0;
const DAY_TICKS: f32 = 2100.0;
const TICK_TO_ANGLE: f32 = TAU / DAY_TICKS;

// ── 日月轨道 ──────────────────────────────────────────
const ORBIT_R: f32 = WS * 0.65; // 轨道半径，比棋盘大
const SUN_R: f32 = 540.0;
const MOON_R: f32 = 300.0;
const HORIZON_CUTOFF: f32 = 0.04; // sin(elev) < 此值 → 渐隐

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Step 3 — 日月星辰".into(), resolution: (1280.,720.).into(), ..default() }),
            ..default()
        }),
            MaterialPlugin::<SunMaterial>::default(),
            MaterialPlugin::<MoonMaterial>::default(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, sky_tick, sun_move, moon_move, star_fade, sun_light))
        .run();
}

// ── 资源/组件 ─────────────────────────────────────────

#[derive(Resource)] struct OC { yaw: f32, pitch: f32, radius: f32, focus: Vec3 }
#[derive(Resource)] struct DayCycle { tick: f32 }
#[derive(Component)] struct Sky;
#[derive(Component)] struct Sun;
#[derive(Component)] struct Moon;
#[derive(Component)] struct StarField;

// ── 相机（同 step2） ──────────────────────────────────

fn orbit_camera(
    mut oc: ResMut<OC>, mut qc: Query<&mut Transform, With<Camera3d>>,
    btn: Res<ButtonInput<MouseButton>>, mut scr: EventReader<MouseWheel>,
    mut motion: EventReader<bevy::input::mouse::MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>, wq: Query<&Window>, mut day: ResMut<DayCycle>,
) {
    let Ok(mut t) = qc.get_single_mut() else { return };
    let Ok(w) = wq.get_single() else { return };
    let Some(_cursor) = w.cursor_position() else { return };
    let mut dx=0.0f32; let mut dy=0.0f32;
    for ev in motion.read() { dx+=ev.delta.x; dy+=ev.delta.y; }
    for ev in scr.read() { oc.radius=(oc.radius - ev.y*oc.radius*0.1).clamp(300.,SKY_R*0.85); }
    if btn.pressed(MouseButton::Left) { oc.yaw-=dx*0.003; oc.pitch=(oc.pitch+dy*0.003).clamp(-1.5,1.5); }
    if btn.pressed(MouseButton::Right) {
        let fwd=(oc.focus - spherical(oc.yaw,oc.pitch,oc.radius)).normalize();
        let r=Vec3::new(fwd.x,0.,fwd.z).normalize_or_zero().cross(Vec3::Y);
        let u=r.cross(fwd); let sp=oc.radius*0.0003;
        oc.focus+=r*(-dx*sp)+u*(-dy*sp);
    }
    if keys.pressed(KeyCode::KeyA){oc.yaw-=0.03;} if keys.pressed(KeyCode::KeyD){oc.yaw+=0.03;}
    if keys.pressed(KeyCode::KeyW){oc.pitch=(oc.pitch+0.03).min(1.5);}
    if keys.pressed(KeyCode::KeyS){oc.pitch=(oc.pitch-0.03).max(-1.5);}
    let lk=[KeyCode::Digit1,KeyCode::Digit2,KeyCode::Digit3,KeyCode::Digit4];
    for i in 0..4{if keys.just_pressed(lk[i]){oc.focus=Vec3::new(WH,LH*(i+1)as f32*0.5,WH);oc.pitch=0.4;oc.radius=LH*(i+1)as f32*3.;}}
    if keys.just_pressed(KeyCode::KeyR){oc.yaw=-2.3;oc.pitch=0.55;oc.radius=WS*0.8;oc.focus=Vec3::new(WH,0.,WH);}
    if keys.pressed(KeyCode::ArrowRight){day.tick=(day.tick+4.)%DAY_TICKS;}
    if keys.pressed(KeyCode::ArrowLeft){day.tick=(day.tick-4.+DAY_TICKS)%DAY_TICKS;}
    let mut cp=oc.focus+spherical(oc.yaw,oc.pitch,oc.radius);
    let sc=Vec3::new(WH,0.,WH); let to=cp-sc;
    if to.length()>SKY_R*0.92{cp=sc+to.normalize_or_zero()*SKY_R*0.92;oc.radius=(cp-oc.focus).length().max(300.);}
    t.translation=cp; t.look_at(oc.focus,Vec3::Y);
}
fn spherical(yaw:f32,pitch:f32,r:f32)->Vec3{Vec3::new(r*pitch.cos()*yaw.sin(),r*pitch.sin(),r*pitch.cos()*yaw.cos())}

// ── 日月轨道 ──────────────────────────────────────────
fn sun_pos(tick:f32)->Vec3{let a=tick*TICK_TO_ANGLE;Vec3::new(WH+ORBIT_R*a.cos(),ORBIT_R*a.sin().max(-ORBIT_R*0.3),WH)}
fn moon_pos(tick:f32)->Vec3{let a=tick*TICK_TO_ANGLE+PI;Vec3::new(WH+ORBIT_R*a.cos(),ORBIT_R*a.sin().max(-ORBIT_R*0.3),WH)}
fn sun_elev(tick:f32)->f32{(tick*TICK_TO_ANGLE).sin()}
fn moon_elev(tick:f32)->f32{((tick*TICK_TO_ANGLE)+PI).sin()}

fn fade(elev:f32)->f32{((elev-HORIZON_CUTOFF)/HORIZON_CUTOFF*2.).clamp(0.,1.)}

// ── 低多边形球（Bevy 0.15 自带 Sphere） ────────────────
fn lowpoly_sphere(r:f32,_sub:u32)->Mesh{
    Sphere::new(r).mesh().build()
}

// ── 简单 hash 随机 ─────────────────────────────────────
fn hash(x:f32,y:f32,z:f32)->f32{let h=(x*12.9898+y*78.233+z*45.164).sin()*43758.5453;h-h.floor()}

// ── 星星 mesh ─────────────────────────────────────────
fn star_mesh()->Mesh{
    let mut v=vec![]; let mut idx=vec![];
    for i in 0..600{
        let r=SKY_R*0.98;
        // 用 hash 生成球面随机方向（上半球偏重）
        let phi=hash(i as f32,0.,0.)*TAU;
        let theta=(hash(0.,i as f32,0.)*0.7+0.25)*FRAC_PI_2;
        let x=r*theta.sin()*phi.cos();let y=r*theta.cos();let z=r*theta.sin()*phi.sin();
        let a=v.len()as u32;v.push([x,y,z]);idx.push(a);
    }
    let mut m=Mesh::new(PrimitiveTopology::PointList,RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,v);m.insert_indices(Indices::U32(idx));m
}

// ── 启动 ──────────────────────────────────────────────

fn setup(mut c:Commands,mut meshes:ResMut<Assets<Mesh>>,mut mats:ResMut<Assets<StandardMaterial>>,
         mut sun_mats:ResMut<Assets<SunMaterial>>,mut moon_mats:ResMut<Assets<MoonMaterial>>){
    // 天空球
    c.spawn((Mesh3d(meshes.add(sky_mesh())),MeshMaterial3d(mats.add(StandardMaterial{unlit:true,cull_mode:None,..default()})),
        Transform::from_xyz(WH,0.,WH),Sky));
    // 线框棋盘
    c.spawn((Mesh3d(meshes.add(grid_mesh())),MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(0.85,0.85,0.85),unlit:true,..default()})),Transform::default()));
    // 太阳：自定义 Flat Shading + 渐变材质
    c.spawn((Mesh3d(meshes.add(lowpoly_sphere(SUN_R,5))),MeshMaterial3d(sun_mats.add(SunMaterial{
        uniforms:SunUniforms{color_center:LinearRgba::new(1.,0.9,0.1,1.),color_edge:LinearRgba::new(0.9,0.25,0.,1.),
        emissive_intensity:8.0,}})),Sun));
    // 月亮：自定义坑洼 noise shader
    c.spawn((Mesh3d(meshes.add(lowpoly_sphere(MOON_R,5))),MeshMaterial3d(moon_mats.add(MoonMaterial{
        uniforms:MoonUniforms{base_color:LinearRgba::new(0.82,0.82,0.86,1.),crater_color:LinearRgba::new(0.55,0.55,0.6,1.),
        emissive_intensity:3.0,}})),Moon));
    // 星星
    c.spawn((Mesh3d(meshes.add(star_mesh())),MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(1.,1.,1.),unlit:true,..default()})),StarField));
    // 方向光
    c.spawn((DirectionalLight{color:Color::srgb(1.,0.9,0.7),illuminance:8000.,shadows_enabled:false,..default()},Transform::default()));
    c.insert_resource(AmbientLight{color:Color::srgb(0.35,0.4,0.55),brightness:500.});
    c.spawn((Camera3d::default(),Camera{hdr:true,..default()},
        BloomSettings::default(),Projection::Perspective(PerspectiveProjection{fov:50_f32.to_radians(),..default()})));
    c.insert_resource(OC{yaw:-2.3,pitch:0.55,radius:WS*0.8,focus:Vec3::new(WH,0.,WH)});
    c.insert_resource(DayCycle{tick:800.});
}

// ── 日月位置 + 渐显渐隐 ─────────────────────────────

fn sun_move(day:Res<DayCycle>,mut q:Query<&mut Transform,With<Sun>>){
    let sf=fade(sun_elev(day.tick));
    if let Ok(mut t)=q.get_single_mut(){t.translation=sun_pos(day.tick);t.scale=Vec3::splat(sf);}
}
fn moon_move(day:Res<DayCycle>,mut q:Query<&mut Transform,With<Moon>>){
    let mf=fade(moon_elev(day.tick)); let mp=moon_pos(day.tick);
    for mut t in q.iter_mut(){t.translation=mp;t.scale=Vec3::splat(mf);}
}
fn star_fade(day:Res<DayCycle>,mut q:Query<&mut Transform,With<StarField>>){
    let se=sun_elev(day.tick); let a=((0.15-se)/0.3).clamp(0.,1.);
    if let Ok(mut t)=q.get_single_mut(){t.scale=Vec3::splat(a);}
}
fn sun_light(day:Res<DayCycle>,mut q:Query<(&mut DirectionalLight,&mut Transform)>,mut amb:ResMut<AmbientLight>){
    let se=sun_elev(day.tick); let sp=sun_pos(day.tick);
    if let Ok((mut l,mut t))=q.get_single_mut(){
        let dir=(Vec3::new(WH,0.,WH)-sp).normalize();
        l.color=if se>0.{Color::srgb(1.,0.7+se*0.3,0.35+se*0.45)}else{Color::srgb(0.3,0.35,0.65)};
        l.illuminance=if se>0.{1500.+se*6500.}else{150.}; t.look_to(dir,Vec3::Y);
    }
    amb.brightness=if se>0.{150.+se*400.}else{60.};
}

// ── 天空: 6点颜色渐变 ─────────────────────────────────
fn sky_mesh()->Mesh{
    let(res,r)=(48u32,SKY_R);let mut m=Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default());
    let mut p=vec![];let mut n=vec![];let mut c=vec![];let mut idx=vec![];
    for lat in 0..=res{let t=lat as f32/res as f32*PI-FRAC_PI_2;
        for lon in 0..=res*2{let phi=lon as f32/(res*2)as f32*TAU;
            let x=r*t.cos()*phi.cos();let y=r*t.sin();let z=r*t.cos()*phi.sin();
            p.push([x,y,z]);let l=(x*x+y*y+z*z).sqrt();n.push([-x/l,-y/l,-z/l]);c.push([0.,0.,0.,1.]);}}
    let cc=res*2+1;for lat in 0..res{for lon in 0..res*2{let a=lat*cc+lon;idx.extend_from_slice(&[a,a+1,a+cc+1,a,a+cc+1,a+cc]);}}
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,p);m.insert_attribute(Mesh::ATTRIBUTE_NORMAL,n);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR,c);m.insert_indices(Indices::U32(idx));m
}

fn sky_tick(day:Res<DayCycle>,q:Query<&Mesh3d,With<Sky>>,mut meshes:ResMut<Assets<Mesh>>){
    let Ok(h)=q.get_single()else{return};let Some(m)=meshes.get_mut(h)else{return};
    let Some(VertexAttributeValues::Float32x3(pos))=m.attribute(Mesh::ATTRIBUTE_POSITION)else{return};
    let se=sun_elev(day.tick);let mp=moon_elev(day.tick);
    let mut colors=Vec::with_capacity(pos.len());
    for p in pos{
        let dir=Vec3::new(p[0]-WH,p[1],p[2]-WH).normalize();
        let h=dir.y.clamp(0.,1.); // 0=地平线,1=天顶
        // 6点颜色渐变表
        let sky=sky_color(se,mp,h);
        // 夜间加星星亮点（在天空球顶点色里模拟）
        let star_bright=((0.15-se)/0.3).clamp(0.,1.);
        let star=if star_bright>0.&&h>0.3{star_bright*0.4*(rand_frag(p[0],p[1],p[2]))}else{0.};
        colors.push([(sky.red+star).min(1.),(sky.green+star).min(1.),(sky.blue+star*0.7).min(1.),1.]);
    }
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR,colors);
}

fn rand_frag(x:f32,y:f32,z:f32)->f32{
    let h=(x*12.9898+y*78.233+z*45.164).sin()*43758.5453; (h-h.floor()).powi(3)
}

fn sky_color(sun_elev:f32,_moon_elev:f32,h:f32)->Srgba{
    // 6个关键时间点的天顶色(sky)和地平线色(horizon)
    let(sky,hor)=if sun_elev>0.5{// 正午
        (Srgba::new(0.25,0.45,0.95,1.),Srgba::new(0.7,0.8,1.0,1.))
    }else if sun_elev>0.15{// 上午/下午
        let t=(sun_elev-0.15)/0.35;
        (Srgba::new(0.2+t*0.05,0.38+t*0.07,0.85+t*0.1,1.),Srgba::new(0.75,0.75+t*0.05,0.85+t*0.15,1.))
    }else if sun_elev>0.0{// 日出/日落
        let t=sun_elev/0.15;
        (Srgba::new(0.15+t*0.05,0.2+t*0.18,0.5+t*0.35,1.),Srgba::new(0.9,0.5+t*0.25,0.3+t*0.55,1.))
    }else if sun_elev>-0.15{// 黄昏/黎明
        let t=(sun_elev+0.15)/0.15;
        (Srgba::new(0.03+t*0.12,0.03+t*0.17,0.1+t*0.4,1.),Srgba::new(0.4*t,0.2*t,0.8*t,1.))
    }else{// 深夜
        (Srgba::new(0.02,0.02,0.08,1.),Srgba::new(0.02,0.02,0.06,1.))
    };
    Srgba::new(
        sky.red*(1.-h)+hor.red*h,
        sky.green*(1.-h)+hor.green*h,
        sky.blue*(1.-h)+hor.blue*h,1.)
}

fn grid_mesh()->Mesh{
    let mut v=vec![];let mut i=vec![];let n=GRID as usize;
    for l in 0..=LAYERS{let y=l as f32*LH;
        for zi in 0..=n{let z=zi as f32*CELL;let a=v.len() as u32;v.push([0.,y,z]);v.push([WS,y,z]);i.push(a);i.push(a+1);}
        for xi in 0..=n{let x=xi as f32*CELL;let a=v.len() as u32;v.push([x,y,0.]);v.push([x,y,WS]);i.push(a);i.push(a+1);}
    }
    for xi in 0..=n{let x=xi as f32*CELL;
        for zi in 0..=n{let z=zi as f32*CELL;let a=v.len() as u32;v.push([x,0.,z]);v.push([x,LAYERS as f32*LH,z]);i.push(a);i.push(a+1);}
    }
    let mut m=Mesh::new(PrimitiveTopology::LineList,RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,v);m.insert_indices(Indices::U32(i));m
}
