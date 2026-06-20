//! Step 5: 物理大气散射天空 + 棋盘 + 日月星辰
//! cargo run --bin step5_atmosphere
//!
//! 关键变化：用 Bevy 0.18 内置 Atmosphere 替代手写 CPU 天空球

use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::MouseWheel;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::post_process::bloom::Bloom;
use bevy::pbr::{Atmosphere, ScatteringMedium};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, PrimitiveTopology, ShaderType};
use bevy::shader::ShaderRef;
use bevy::window::WindowResolution;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// ── 世界 ──────────────────────────────────────────────
const GRID: u32 = 64; const CELL: f32 = 158.0;
const WS: f32 = GRID as f32 * CELL; const WH: f32 = WS * 0.5;
const DAY_TICKS: f32 = 2100.0;
const TICK_TO_ANGLE: f32 = TAU / DAY_TICKS;

// ── 日月轨道 ──────────────────────────────────────────
const ORBIT_R: f32 = WS * 0.65;
const SUN_R: f32 = 540.0;
const MOON_R: f32 = 300.0;
const HORIZON_CUTOFF: f32 = 0.04;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window { title: "Step 5 — 物理大气".into(), resolution: WindowResolution::new(1280,720), ..default() }),
                ..default()
            }),
            MaterialPlugin::<SunMaterial>::default(),
            MaterialPlugin::<MoonMaterial>::default(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, sun_move, moon_move, star_fade, sun_light_update))
        .run();
}

// ── 自定义材质（日月） ─────────────────────────────

#[derive(ShaderType, Debug, Clone)]
struct SunUniforms { color_center: LinearRgba, color_edge: LinearRgba, emissive_intensity: f32 }
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SunMaterial { #[uniform(0)] uniforms: SunUniforms }
impl Material for SunMaterial { fn fragment_shader() -> ShaderRef { "shaders/sun_material.wgsl".into() } }

#[derive(ShaderType, Debug, Clone)]
struct MoonUniforms { base_color: LinearRgba, crater_color: LinearRgba, emissive_intensity: f32 }
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct MoonMaterial { #[uniform(0)] uniforms: MoonUniforms }
impl Material for MoonMaterial { fn fragment_shader() -> ShaderRef { "shaders/moon_material.wgsl".into() } }

// ── 资源 ─────────────────────────────────────────────

#[derive(Resource)] struct OC { yaw:f32, pitch:f32, radius:f32, focus:Vec3 }
#[derive(Resource)] struct DayCycle { tick: f32 }
#[derive(Component)] struct Sun;
#[derive(Component)] struct Moon;
#[derive(Component)] struct StarField;

// ── 相机 ────────────────────────────────────────────

fn orbit_camera(
    mut oc: ResMut<OC>, mut qc: Query<&mut Transform, With<Camera3d>>,
    btn: Res<ButtonInput<MouseButton>>, mut scr: MessageReader<MouseWheel>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>, wq: Query<&Window>, mut day: ResMut<DayCycle>,
) {
    let Ok(mut t) = qc.single_mut() else { return };
    let Ok(_w) = wq.single() else { return };
    let mut dx=0.0f32; let mut dy=0.0f32;
    for ev in motion.read() { dx+=ev.delta.x; dy+=ev.delta.y; }
    for ev in scr.read() { oc.radius=(oc.radius - ev.y*oc.radius*0.1).clamp(300.,15000.*0.85); }
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
    if keys.just_pressed(KeyCode::KeyR){oc.yaw=-2.3;oc.pitch=0.55;oc.radius=WS*0.8;oc.focus=Vec3::new(WH,0.,WH);}
    if keys.pressed(KeyCode::ArrowRight){day.tick=(day.tick+4.)%DAY_TICKS;}
    if keys.pressed(KeyCode::ArrowLeft){day.tick=(day.tick-4.+DAY_TICKS)%DAY_TICKS;}
    let mut cp=oc.focus+spherical(oc.yaw,oc.pitch,oc.radius);
    let sc=Vec3::new(WH,0.,WH); let to=cp-sc;
    if to.length()>15000.*0.92{cp=sc+to.normalize_or_zero()*15000.*0.92;oc.radius=(cp-oc.focus).length().max(300.);}
    t.translation=cp; t.look_at(oc.focus,Vec3::Y);
}
fn spherical(yaw:f32,pitch:f32,r:f32)->Vec3{Vec3::new(r*pitch.cos()*yaw.sin(),r*pitch.sin(),r*pitch.cos()*yaw.cos())}

// ── 日月轨道 ─────────────────────────────────────────

fn sun_pos(t:f32)->Vec3{let a=t*TICK_TO_ANGLE;Vec3::new(WH+ORBIT_R*a.cos(),ORBIT_R*a.sin().max(-ORBIT_R*0.3),WH)}
fn moon_pos(t:f32)->Vec3{let a=t*TICK_TO_ANGLE+PI;Vec3::new(WH+ORBIT_R*a.cos(),ORBIT_R*a.sin().max(-ORBIT_R*0.3),WH)}
fn sun_elev(t:f32)->f32{(t*TICK_TO_ANGLE).sin()}
fn fade(elev:f32)->f32{((elev-HORIZON_CUTOFF)/HORIZON_CUTOFF*2.).clamp(0.,1.)}

fn sun_move(day:Res<DayCycle>,mut q:Query<&mut Transform,With<Sun>>){
    let sf=fade(sun_elev(day.tick));
    if let Ok(mut t)=q.single_mut(){t.translation=sun_pos(day.tick);t.scale=Vec3::splat(sf);}
}
fn moon_move(day:Res<DayCycle>,mut q:Query<&mut Transform,With<Moon>>){
    let mf=fade(((day.tick*TICK_TO_ANGLE)+PI).sin());
    if let Ok(mut t)=q.single_mut(){t.translation=moon_pos(day.tick);t.scale=Vec3::splat(mf);}
}
fn star_fade(day:Res<DayCycle>,mut q:Query<&mut Visibility,With<StarField>>){
    if let Ok(mut v)=q.single_mut(){*v=if sun_elev(day.tick)<0.1{Visibility::Visible}else{Visibility::Hidden};}
}

// ── 光源 ────────────────────────────────────────────

fn sun_light_update(day:Res<DayCycle>,mut q:Query<(&mut DirectionalLight,&mut Transform)>,
                    mut amb:ResMut<GlobalAmbientLight>){
    let se=sun_elev(day.tick); let sp=sun_pos(day.tick);
    if let Ok((mut l,mut t))=q.single_mut(){
        l.color=if se>0.{Color::srgb(1.,0.7+se*0.3,0.35+se*0.45)}else{Color::srgb(0.3,0.35,0.65)};
        l.illuminance=if se>0.{1500.+se*6500.}else{150.};
        t.look_to((Vec3::new(WH,0.,WH)-sp).normalize(),Vec3::Y);
    }
    amb.brightness=if se>0.{150.+se*400.}else{60.};
}

// ── 辅助 ────────────────────────────────────────────

fn lowpoly_sphere(r:f32,_sub:u32)->Mesh{Sphere::new(r).mesh().build()}
fn hash(x:f32,y:f32,z:f32)->f32{let h=(x*12.9898+y*78.233+z*45.164).sin()*43758.5453;h-h.floor()}
fn star_mesh()->Mesh{
    let mut v=vec![]; let mut idx=vec![];
    for i in 0..600{
        let r=SKY_R;let phi=hash(i as f32,0.,0.)*TAU;
        let theta=(hash(0.,i as f32,0.)*0.7+0.25)*FRAC_PI_2;
        let x=WH+r*theta.sin()*phi.cos();let y=r*theta.cos();let z=WH+r*theta.sin()*phi.sin();
        let a=v.len()as u32;v.push([x,y,z]);idx.push(a);
    }
    let mut m=Mesh::new(PrimitiveTopology::PointList,RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,v);m.insert_indices(Indices::U32(idx));m
}
fn grid_mesh()->Mesh{
    let mut v=vec![];let mut i=vec![];let n=GRID as usize;
    for zi in 0..=n{let z=zi as f32*CELL;let a=v.len()as u32;v.push([0.,0.,z]);v.push([WS,0.,z]);i.push(a);i.push(a+1);}
    for xi in 0..=n{let x=xi as f32*CELL;let a=v.len()as u32;v.push([x,0.,0.]);v.push([x,0.,WS]);i.push(a);i.push(a+1);}
    let mut m=Mesh::new(PrimitiveTopology::LineList,RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,v);m.insert_indices(Indices::U32(i));m
}

// ── 启动 ────────────────────────────────────────────

const SKY_R: f32 = 15000.0;

fn setup(
    mut c:Commands,mut meshes:ResMut<Assets<Mesh>>,mut mats:ResMut<Assets<StandardMaterial>>,
    mut sun_mats:ResMut<Assets<SunMaterial>>,mut moon_mats:ResMut<Assets<MoonMaterial>>,
    mut scattering_media:ResMut<Assets<ScatteringMedium>>,
){
    // ★ 相机 + 物理大气
    let medium = scattering_media.add(ScatteringMedium::default());
    c.spawn((
        Camera3d::default(),
        Atmosphere::earthlike(medium),
        Projection::Perspective(PerspectiveProjection{fov:50_f32.to_radians(),..default()}),
    ));

    // 线框棋盘
    c.spawn((Mesh3d(meshes.add(grid_mesh())),MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(0.85,0.85,0.85),unlit:true,..default()})),Transform::default()));

    // 太阳
    c.spawn((Mesh3d(meshes.add(lowpoly_sphere(SUN_R,5))),MeshMaterial3d(sun_mats.add(SunMaterial{
        uniforms:SunUniforms{color_center:LinearRgba::new(1.,0.9,0.1,1.),color_edge:LinearRgba::new(0.9,0.25,0.,1.),
        emissive_intensity:8.0,}})),Sun));

    // 月亮
    c.spawn((Mesh3d(meshes.add(lowpoly_sphere(MOON_R,5))),MeshMaterial3d(moon_mats.add(MoonMaterial{
        uniforms:MoonUniforms{base_color:LinearRgba::new(0.82,0.82,0.86,1.),crater_color:LinearRgba::new(0.55,0.55,0.6,1.),
        emissive_intensity:3.0,}})),Moon));

    // 星星
    c.spawn((Mesh3d(meshes.add(star_mesh())),MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(1.,1.,1.),unlit:true,..default()})),StarField));

    // 方向光
    c.spawn((DirectionalLight{color:Color::srgb(1.,0.9,0.7),illuminance:8000.,shadows_enabled:false,..default()},Transform::default()));
    c.insert_resource(GlobalAmbientLight{color:Color::srgb(0.35,0.4,0.55),brightness:500.,affects_lightmapped_meshes:false});
    c.insert_resource(OC{yaw:-2.3,pitch:0.55,radius:WS*0.8,focus:Vec3::new(WH,0.,WH)});
    c.insert_resource(DayCycle{tick:800.});
}
