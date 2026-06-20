//! Step 3: 天空 + 棋盘 + 太阳(实体组合) + 月亮 + 星星
//! cargo run --bin step3_skysun

use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::MouseWheel;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::window::WindowResolution;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// ── 世界 ──────────────────────────────────────────────
const GRID: u32 = 64; const CELL: f32 = 158.0;
const WS: f32 = GRID as f32 * CELL; const WH: f32 = WS * 0.5;
const SKY_R: f32 = 15000.0;
const DAY_TICKS: f32 = 2100.0;
const TICK_TO_ANGLE: f32 = TAU / DAY_TICKS;

// ── 日月轨道 ──────────────────────────────────────────
const ORBIT_R: f32 = WS * 0.65;
const SUN_R: f32 = 540.0;
const MOON_R: f32 = 300.0;
const HORIZON_CUTOFF: f32 = 0.04;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Step 3 — 日月星辰".into(), resolution: WindowResolution::new(1280,720), ..default() }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, sky_tick, sun_move, moon_move, star_fade, sun_light, sun_spin, spike_breathe))
        .run();
}

// ── 资源/组件 ─────────────────────────────────────────

#[derive(Resource)] struct OC { yaw: f32, pitch: f32, radius: f32, focus: Vec3 }
#[derive(Resource)] struct DayCycle { tick: f32 }
#[derive(Component)] struct Sky;
#[derive(Component)] struct Sun;
#[derive(Component)] struct Moon;
#[derive(Component)] struct StarField;
#[derive(Component)] struct SunSpike { phase: f32, base_scale: f32 }
#[derive(Component)] struct SunBase;

// ── 相机 ──────────────────────────────────────────────

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
    for i in 0..4{if keys.just_pressed(lk[i]){oc.focus=Vec3::new(WH,0.,WH);oc.pitch=0.4;oc.radius=WS*0.5;}}
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
fn fade(elev:f32)->f32{((elev-HORIZON_CUTOFF)/HORIZON_CUTOFF*2.).clamp(0.,1.)}

fn sun_move(day:Res<DayCycle>,mut q:Query<&mut Transform,With<Sun>>){
    let sf=fade(sun_elev(day.tick)); let sp=sun_pos(day.tick);
    for mut t in q.iter_mut(){t.translation=sp;t.scale=Vec3::splat(sf);}
}
fn moon_move(day:Res<DayCycle>,mut q:Query<&mut Transform,With<Moon>>){
    let mf=fade(((day.tick*TICK_TO_ANGLE)+PI).sin()); let mp=moon_pos(day.tick);
    for mut t in q.iter_mut(){t.translation=mp;t.scale=Vec3::splat(mf);}
}
fn star_fade(day:Res<DayCycle>,mut q:Query<&mut Visibility,With<StarField>>){
    if let Ok(mut v)=q.single_mut(){*v=if sun_elev(day.tick)<0.1{Visibility::Visible}else{Visibility::Hidden};}
}
fn sun_light(day:Res<DayCycle>,mut q:Query<(&mut DirectionalLight,&mut Transform)>,mut amb:ResMut<GlobalAmbientLight>){
    let se=sun_elev(day.tick); let sp=sun_pos(day.tick);
    if let Ok((mut l,mut t))=q.single_mut(){
        l.color=if se>0.{Color::srgb(1.,0.7+se*0.3,0.35+se*0.45)}else{Color::srgb(0.3,0.35,0.65)};
        l.illuminance=if se>0.{1500.+se*6500.}else{150.};
        t.look_to((Vec3::new(WH,0.,WH)-sp).normalize(),Vec3::Y);
    }
    amb.brightness=if se>0.{150.+se*400.}else{60.};
}

// ── 太阳实体组合 ──────────────────────────────────────

fn cone_mesh(h:f32,r:f32)->Mesh{
    let mut pos=vec![];let mut nor=vec![];let mut idx=vec![];
    let n=4; // 四面锥
    pos.push([0.,-h*0.1,0.]); // 底面中心
    for i in 0..=n{let a=i as f32/n as f32*TAU;pos.push([r*a.cos(),-h*0.1,r*a.sin()]);}
    pos.push([0.,h,0.]); // 顶点
    let ai=pos.len()as u32-1;
    for i in 1..=n{idx.extend_from_slice(&[ai,(i-1)as u32,i as u32]);}
    for _ in 0..pos.len(){nor.push([0.,1.,0.]);}
    let mut m=Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,pos);m.insert_attribute(Mesh::ATTRIBUTE_NORMAL,nor);
    m.insert_indices(Indices::U32(idx));m
}

fn spawn_sun(commands:&mut Commands,meshes:&mut ResMut<Assets<Mesh>>,mats:&mut ResMut<Assets<StandardMaterial>>){
    let r=SUN_R;
    // icosphere 简单构造
    let Ok(base)=Sphere::new(r).mesh().ico(4)else{return};
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default());
    // 复制顶点做 flat shading
    let Some(VertexAttributeValues::Float32x3(bp))=base.attribute(Mesh::ATTRIBUTE_POSITION)else{return};
    let Some(indices)=base.indices()else{return};
    let Indices::U32(bi)=indices else {return};
    let mut p=vec![];let mut n=vec![];let mut idx=vec![];
    for t in bi.chunks(3){
        if t.len()<3{continue}
        let p0=Vec3::from(bp[t[0]as usize]);let p1=Vec3::from(bp[t[1]as usize]);let p2=Vec3::from(bp[t[2]as usize]);
        let fn_=(p1-p0).cross(p2-p0).normalize();
        let bi_=p.len()as u32;
        for &vi in t{p.push(bp[vi as usize]);n.push([fn_.x,fn_.y,fn_.z]);}
        idx.extend_from_slice(&[bi_,bi_+1,bi_+2]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION,p);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL,n);
    mesh.insert_indices(Indices::U32(idx));

    // 太阳基体: 暗橙红 + unlit
    let sun_id = commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(mats.add(StandardMaterial{base_color:Color::srgb(2.,0.4,0.),unlit:true,..default()})),
        Sun, SunBase,
    )).id();

    // 60个尖刺子实体，随机分布球面
    for i in 0..60{
        let phi=i as f32*2.399; // 黄金角分布
        let theta=(phi*0.5).sin().acos(); // 球面均匀
        let dir=Vec3::new(theta.sin()*phi.cos(),theta.cos(),theta.sin()*phi.sin()).normalize();
        let spike_h=r*0.08+((i*37+13)as f32).sin().abs()*r*0.25;
        let spike_r=r*0.015+((i*53+7)as f32).sin().abs()*r*0.03;
        let spike_id = commands.spawn((
            Mesh3d(meshes.add(cone_mesh(spike_h,spike_r))),
            MeshMaterial3d(mats.add(StandardMaterial{base_color:Color::srgb(1.,0.85,0.4),
                emissive:Color::srgb(8.,6.,1.).into(),unlit:true,..default()})),
            Transform::from_translation(r*dir).looking_at(Vec3::ZERO,Vec3::Y),
            SunSpike{phase:i as f32*0.5+((i*31)as f32).sin(),base_scale:0.6+((i*17)as f32).sin().abs()*0.4},
        )).id();
        commands.entity(sun_id).add_child(spike_id);
    }
}

// ── 太阳自转 ──────────────────────────────────────────
fn sun_spin(time:Res<Time>,mut q:Query<&mut Transform,With<SunBase>>){
    for mut tr in q.iter_mut(){
        tr.rotate_local_y(time.delta_secs()*0.03);
    }
}

// ── 尖刺呼吸（sine缩放，模拟耀斑消长） ────────────────
fn spike_breathe(time:Res<Time>,mut q:Query<(&SunSpike,&mut Transform)>){
    let t=time.elapsed_secs();
    for(spike,mut tr)in q.iter_mut(){
        let breath=(t*0.6+spike.phase).sin()*0.3+0.85;
        let s=spike.base_scale*breath;
        tr.scale=Vec3::new(s,s,s*1.5); // Z轴拉伸=尖刺的"长度"
    }
}

// ── 低多边形球 ────────────────────────────────────────
fn lowpoly_sphere(r:f32,_sub:u32)->Mesh{Sphere::new(r).mesh().build()}

// ── 星星 ──────────────────────────────────────────────
fn hash(x:f32,y:f32,z:f32)->f32{let h=(x*12.9898+y*78.233+z*45.164).sin()*43758.5453;h-h.floor()}
fn star_mesh()->Mesh{
    let mut v=vec![]; let mut idx=vec![];
    for i in 0..600{
        let r=SKY_R*0.98;
        let phi=hash(i as f32,0.,0.)*TAU;
        let theta=(hash(0.,i as f32,0.)*0.7+0.25)*FRAC_PI_2;
        let x=WH+r*theta.sin()*phi.cos();let y=r*theta.cos();let z=WH+r*theta.sin()*phi.sin();
        let a=v.len()as u32;v.push([x,y,z]);idx.push(a);
    }
    let mut m=Mesh::new(PrimitiveTopology::PointList,RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,v);m.insert_indices(Indices::U32(idx));m
}

// ── 天空 ──────────────────────────────────────────────

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

fn sky_tick(day:Res<DayCycle>,time:Res<Time>,q:Query<&Mesh3d,With<Sky>>,mut meshes:ResMut<Assets<Mesh>>){
    let Ok(h)=q.single()else{return};let Some(m)=meshes.get_mut(h)else{return};
    let Some(VertexAttributeValues::Float32x3(pos))=m.attribute(Mesh::ATTRIBUTE_POSITION)else{return};
    let se=sun_elev(day.tick); let sd=Vec3::new((day.tick*TICK_TO_ANGLE).cos(),(day.tick*TICK_TO_ANGLE).sin(),0.);
    let t=time.elapsed_secs();
    let mut colors=Vec::with_capacity(pos.len());
    for p in pos{
        let dir=Vec3::new(p[0]-WH,p[1],p[2]-WH).normalize();
        let sky=sky_shader(dir, se, sd);
        // 云层（可选，保持兼容）
        let cloud=0.0f32;
        colors.push([(sky[0]+cloud).min(1.),(sky[1]+cloud*0.85).min(1.),(sky[2]+cloud*0.7).min(1.),1.]);
    }
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR,colors);
}

fn sky_shader(view_dir:Vec3, sun_elev:f32, sun_dir:Vec3)->[f32;3]{
    let view_h=view_dir.y.clamp(0.,1.);let se=sun_elev.clamp(-1.,1.);
    // 三套预设
    let day_zenith=[0.05,0.2,0.6];let day_horizon=[0.5,0.7,0.9];
    let sunset_zenith=[0.1,0.08,0.25];let sunset_horizon=[1.0,0.35,0.05];
    let night_zenith=[0.0,0.0,0.02];let night_horizon=[0.01,0.04,0.08];
    let sunset_w=1.0-smoothstep_f(0.0,0.2,se.abs());
    let day_w=smoothstep_f(0.0,0.25,se);let night_w=smoothstep_f(0.0,0.2,-se);
    let zenith=[day_zenith[0]*day_w+sunset_zenith[0]*sunset_w+night_zenith[0]*night_w,
                day_zenith[1]*day_w+sunset_zenith[1]*sunset_w+night_zenith[1]*night_w,
                day_zenith[2]*day_w+sunset_zenith[2]*sunset_w+night_zenith[2]*night_w];
    let horizon=[day_horizon[0]*day_w+sunset_horizon[0]*sunset_w+night_horizon[0]*night_w,
                 day_horizon[1]*day_w+sunset_horizon[1]*sunset_w+night_horizon[1]*night_w,
                 day_horizon[2]*day_w+sunset_horizon[2]*sunset_w+night_horizon[2]*night_w];
    let t=view_h.powf(0.65);
    let mut c=[zenith[0]*(1.-t)+horizon[0]*t,zenith[1]*(1.-t)+horizon[1]*t,zenith[2]*(1.-t)+horizon[2]*t];
    let sun_dot=view_dir.dot(sun_dir).max(0.);let glow=sun_dot.powf(20.)*sunset_w*0.3;
    c[0]=(c[0]+glow).min(1.);c[1]=(c[1]+glow*0.6).min(1.);c[2]=(c[2]+glow*0.2).min(1.);
    c
}
fn smoothstep_f(e0:f32,e1:f32,x:f32)->f32{let t=((x-e0)/(e1-e0)).clamp(0.,1.);t*t*(3.-2.*t)}

// ── 线框棋盘 ──────────────────────────────────────────

fn grid_mesh()->Mesh{
    let mut v=vec![];let mut i=vec![];let n=GRID as usize;
    for zi in 0..=n{let z=zi as f32*CELL;let a=v.len()as u32;v.push([0.,0.,z]);v.push([WS,0.,z]);i.push(a);i.push(a+1);}
    for xi in 0..=n{let x=xi as f32*CELL;let a=v.len()as u32;v.push([x,0.,0.]);v.push([x,0.,WS]);i.push(a);i.push(a+1);}
    let mut m=Mesh::new(PrimitiveTopology::LineList,RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,v);m.insert_indices(Indices::U32(i));m
}

// ── 启动 ──────────────────────────────────────────────

fn setup(mut c:Commands,mut meshes:ResMut<Assets<Mesh>>,mut mats:ResMut<Assets<StandardMaterial>>){
    // 天空球
    c.spawn((Mesh3d(meshes.add(sky_mesh())),MeshMaterial3d(mats.add(StandardMaterial{unlit:true,cull_mode:None,..default()})),
        Transform::from_xyz(WH,0.,WH),Sky));
    // 线框棋盘
    c.spawn((Mesh3d(meshes.add(grid_mesh())),MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(0.85,0.85,0.85),unlit:true,..default()})),Transform::default()));
    // 太阳：实体组合（基体+60尖刺子实体）
    spawn_sun(&mut c,&mut meshes,&mut mats);
    // 月亮
    c.spawn((Mesh3d(meshes.add(lowpoly_sphere(MOON_R,5))),MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(0.78,0.78,0.82),emissive:Color::srgb(0.15,0.15,0.2).into(),
        perceptual_roughness:0.9,..default()})),Moon));
    // 星星
    c.spawn((Mesh3d(meshes.add(star_mesh())),MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(1.,1.,1.),unlit:true,..default()})),StarField));
    // 方向光 + 全局光照 + 相机 + Bloom
    c.spawn((DirectionalLight{color:Color::srgb(1.,0.9,0.7),illuminance:8000.,shadows_enabled:false,..default()},Transform::default()));
    c.insert_resource(GlobalAmbientLight{color:Color::srgb(0.35,0.4,0.55),brightness:500.,affects_lightmapped_meshes:false});
    c.spawn((Camera3d::default(),Bloom::default(),Projection::Perspective(PerspectiveProjection{fov:50_f32.to_radians(),..default()})));
    c.insert_resource(OC{yaw:-2.3,pitch:0.55,radius:WS*0.8,focus:Vec3::new(WH,0.,WH)});
    c.insert_resource(DayCycle{tick:800.});
}
