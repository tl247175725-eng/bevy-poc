//! Step 3: 天空 + 棋盘 + 太阳/月亮/星星
//! cargo run --bin step3_skysun

use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::MouseWheel;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::window::WindowResolution;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// ── 自定义材质 ─────────────────────────────────────────


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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Step 3 — 日月星辰".into(), resolution: WindowResolution::new(1280,720), ..default() }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, sky_tick, sun_move, moon_move, star_fade, sun_light, animate_sun))
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
    btn: Res<ButtonInput<MouseButton>>, mut scr: MessageReader<MouseWheel>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>, wq: Query<&Window>, mut day: ResMut<DayCycle>,
) {
    let Ok(mut t) = qc.single_mut() else { return };
    let Ok(w) = wq.single() else { return };
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
fn sun_dir(tick:f32)->Vec3{let a=tick*TICK_TO_ANGLE;Vec3::new(a.cos(),a.sin(),0.)}
fn moon_elev(tick:f32)->f32{((tick*TICK_TO_ANGLE)+PI).sin()}

fn fade(elev:f32)->f32{((elev-HORIZON_CUTOFF)/HORIZON_CUTOFF*2.).clamp(0.,1.)}

// ── CPU 平面着色球体（Flat Shading 无 shader） ──────────
fn flat_shaded_sphere(r:f32, sub:u32, center_color:[f32;4], edge_color:[f32;4]) -> Mesh {
    let Ok(base) = Sphere::new(r).mesh().ico(sub) else { return Sphere::new(r).mesh().build() };
    let Some(VertexAttributeValues::Float32x3(pos)) = base.attribute(Mesh::ATTRIBUTE_POSITION) else { return base.into() };
    let Some(indices) = base.indices() else { return base.into() };
    let Indices::U32(idx) = indices else { return base.into() };

    let mut new_pos = Vec::new();
    let mut new_nor = Vec::new();
    let mut new_col = Vec::new();
    let mut new_idx = Vec::new();

    // 每个三角形独立顶点 → 平面着色
    for t in idx.chunks(3) {
        if t.len() < 3 { continue; }
        let p0 = Vec3::from(pos[t[0] as usize]);
        let p1 = Vec3::from(pos[t[1] as usize]);
        let p2 = Vec3::from(pos[t[2] as usize]);
        let face_normal = (p1 - p0).cross(p2 - p0).normalize();
        // 面的中心方向（用于颜色渐变）
        let face_center = (p0 + p1 + p2) / 3.0;
        let facing = face_center.normalize();
        let gradient = (facing.y.abs() * 0.5 + 0.5).clamp(0.0, 1.0);
        let col = [
            center_color[0] * gradient + edge_color[0] * (1.0 - gradient),
            center_color[1] * gradient + edge_color[1] * (1.0 - gradient),
            center_color[2] * gradient + edge_color[2] * (1.0 - gradient),
            1.0,
        ];
        let base_idx = new_pos.len() as u32;
        for &vi in t {
            new_pos.push(pos[vi as usize]);
            new_nor.push([face_normal.x, face_normal.y, face_normal.z]);
            new_col.push(col);
        }
        new_idx.push(base_idx);
        new_idx.push(base_idx + 1);
        new_idx.push(base_idx + 2);
    }

    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_pos);
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_nor);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, new_col);
    m.insert_indices(Indices::U32(new_idx));
    m
}

// ── 3D Simplex 噪声（从 2D 扩展） ──────────────────────
fn hash3(p:Vec3)->f32{let h=(p.x*12.9898+p.y*78.233+p.z*45.164).sin()*43758.5453;h-h.floor()}
fn grad3(h:f32,x:f32,y:f32,z:f32)->f32{let h_=h*TAU;let(s,c)=(h_.sin(),h_.cos());let t=1.0-s;s*x*t+c*y*t+(1.0-t)*z}
fn simplex3d(p:Vec3)->f32{
    let f=1.0/3.0;let s=(p.x+p.y+p.z)*f;let i=(p.x+s).floor();let j=(p.y+s).floor();let k=(p.z+s).floor();
    let g=1.0/6.0;let t=(i+j+k)*g;let x0=p.x-i+t;let y0=p.y-j+t;let z0=p.z-k+t;
    let(i1,j1,k1,i2,j2,k2)=if x0>=y0{if y0>=z0{(1.,0.,0.,1.,1.,0.)}else if x0>=z0{(1.,0.,0.,1.,0.,1.)}else{(0.,0.,1.,1.,0.,1.)}}
        else{if y0<z0{(0.,0.,1.,0.,1.,1.)}else if x0<z0{(0.,1.,0.,0.,1.,1.)}else{(0.,1.,0.,1.,1.,0.)}};
    let x1=x0-i1+g;let y1=y0-j1+g;let z1=z0-k1+g;let x2=x0-i2+2.*g;let y2=y0-j2+2.*g;let z2=z0-k2+2.*g;
    let x3=x0-1.+3.*g;let y3=y0-1.+3.*g;let z3=z0-1.+3.*g;
    let n0=0.6-x0*x0-y0*y0-z0*z0;let n1=0.6-x1*x1-y1*y1-z1*z1;
    let n2=0.6-x2*x2-y2*y2-z2*z2;let n3=0.6-x3*x3-y3*y3-z3*z3;
    let mut v=0.;
    if n0>0.{let t_=n0*n0;v+=t_*t_*grad3(hash3(Vec3::new(i,j,k)),x0,y0,z0);}
    if n1>0.{let t_=n1*n1;v+=t_*t_*grad3(hash3(Vec3::new(i+i1,j+j1,k+k1)),x1,y1,z1);}
    if n2>0.{let t_=n2*n2;v+=t_*t_*grad3(hash3(Vec3::new(i+i2,j+j2,k+k2)),x2,y2,z2);}
    if n3>0.{let t_=n3*n3;v+=t_*t_*grad3(hash3(Vec3::new(i+1.,j+1.,k+1.)),x3,y3,z3);}
    v*32.
}

// ── 独立平面着色法线重算 ────────────────────────────────
fn recompute_flat_normals(mesh:&mut Mesh){
    let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return };
    let Some(indices) = mesh.indices() else { return };
    let Indices::U32(idx) = indices else { return };
    let mut normals = vec![[0.0f32,0.,0.];pos.len()];
    for t in idx.chunks(3){
        if t.len()<3{continue}
        let p0=Vec3::from(pos[t[0]as usize]);let p1=Vec3::from(pos[t[1]as usize]);let p2=Vec3::from(pos[t[2]as usize]);
        let n=(p1-p0).cross(p2-p0).normalize();
        normals[t[0]as usize]=[n.x,n.y,n.z];normals[t[1]as usize]=[n.x,n.y,n.z];normals[t[2]as usize]=[n.x,n.y,n.z];
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL,normals);
}

// ── 动态太阳组件 ────────────────────────────────────────
#[derive(Component)]
struct DynamicSun {
    original_positions: Vec<Vec3>,
    max_spike: f32,
}

// ── 简单 hash 随机 ─────────────────────────────────────
fn hash(x:f32,y:f32,z:f32)->f32{let h=(x*12.9898+y*78.233+z*45.164).sin()*43758.5453;h-h.floor()}

// ── 星星 mesh（以天空球心为原点） ───────────────────────
fn star_mesh()->Mesh{
    let mut v=vec![]; let mut idx=vec![];
    for i in 0..600{
        let r=SKY_R*0.98;
        let phi=hash(i as f32,0.,0.)*TAU;
        let theta=(hash(0.,i as f32,0.)*0.7+0.25)*FRAC_PI_2;
        let x=WH + r*theta.sin()*phi.cos();let y=r*theta.cos();let z=WH + r*theta.sin()*phi.sin();
        let a=v.len()as u32;v.push([x,y,z]);idx.push(a);
    }
    let mut m=Mesh::new(PrimitiveTopology::PointList,RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,v);m.insert_indices(Indices::U32(idx));m
}

// ── 启动 ──────────────────────────────────────────────

fn setup(mut c:Commands,mut meshes:ResMut<Assets<Mesh>>,mut mats:ResMut<Assets<StandardMaterial>>){
    // 天空球
    c.spawn((Mesh3d(meshes.add(sky_mesh())),MeshMaterial3d(mats.add(StandardMaterial{unlit:true,cull_mode:None,..default()})),
        Transform::from_xyz(WH,0.,WH),Sky));
    // 线框棋盘
    c.spawn((Mesh3d(meshes.add(grid_mesh())),MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(0.85,0.85,0.85),unlit:true,..default()})),Transform::default()));
    // 太阳：动态 CPU 顶点形变 + HDR 顶点色 + Bloom
    spawn_dynamic_sun(&mut c,&mut meshes,&mut mats);
    // 月亮：CPU 平面着色 + 灰白渐变
    c.spawn((Mesh3d(meshes.add(flat_shaded_sphere(MOON_R,3,
        [0.85,0.85,0.88,1.0], [0.55,0.55,0.60,1.0]))),
        MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(0.78,0.78,0.82),emissive:Color::srgb(0.15,0.15,0.2).into(),
        perceptual_roughness:0.9,..default()})),Moon));
    // 星星
    c.spawn((Mesh3d(meshes.add(star_mesh())),MeshMaterial3d(mats.add(StandardMaterial{
        base_color:Color::srgb(1.,1.,1.),unlit:true,..default()})),StarField));
    // 方向光
    c.spawn((DirectionalLight{color:Color::srgb(1.,0.9,0.7),illuminance:8000.,shadows_enabled:false,..default()},Transform::default()));
    c.insert_resource(GlobalAmbientLight{color:Color::srgb(0.35,0.4,0.55),brightness:500.,affects_lightmapped_meshes:false});
    c.spawn((Camera3d::default(),Bloom::default(),
        Projection::Perspective(PerspectiveProjection{fov:50_f32.to_radians(),..default()})));
    c.insert_resource(OC{yaw:-2.3,pitch:0.55,radius:WS*0.8,focus:Vec3::new(WH,0.,WH)});
    c.insert_resource(DayCycle{tick:800.});
}

// ── 日月位置 + 渐显渐隐 ─────────────────────────────

fn sun_move(day:Res<DayCycle>,mut q:Query<&mut Transform,With<Sun>>){
    let sf=fade(sun_elev(day.tick));
    if let Ok(mut t)=q.single_mut(){t.translation=sun_pos(day.tick);t.scale=Vec3::splat(sf);}
}
fn moon_move(day:Res<DayCycle>,mut q:Query<&mut Transform,With<Moon>>){
    let mf=fade(moon_elev(day.tick)); let mp=moon_pos(day.tick);
    for mut t in q.iter_mut(){t.translation=mp;t.scale=Vec3::splat(mf);}
}
fn star_fade(day:Res<DayCycle>,mut q:Query<&mut Visibility,With<StarField>>){
    let se=sun_elev(day.tick);
    // 太阳低于地平线→星星可见，高于一定角度→隐藏
    let visible = se < 0.1;
    if let Ok(mut v)=q.single_mut(){
        *v = if visible { Visibility::Visible } else { Visibility::Hidden };
    }
}
fn sun_light(day:Res<DayCycle>,mut q:Query<(&mut DirectionalLight,&mut Transform)>,mut amb:ResMut<GlobalAmbientLight>){
    let se=sun_elev(day.tick); let sp=sun_pos(day.tick);
    if let Ok((mut l,mut t))=q.single_mut(){
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

fn sky_tick(day:Res<DayCycle>,time:Res<Time>,q:Query<&Mesh3d,With<Sky>>,mut meshes:ResMut<Assets<Mesh>>){
    let Ok(h)=q.single()else{return};let Some(m)=meshes.get_mut(h)else{return};
    let Some(VertexAttributeValues::Float32x3(pos))=m.attribute(Mesh::ATTRIBUTE_POSITION)else{return};
    let se=sun_elev(day.tick); let sd=sun_dir(day.tick);
    let t=time.elapsed_secs();
    let mut colors=Vec::with_capacity(pos.len());
    for p in pos{
        let dir=Vec3::new(p[0]-WH,p[1],p[2]-WH).normalize();
        let sky=sky_shader(dir, se, sd);
        // 云层：Simplex2D噪声，仅在中高空(0.2~0.8)，随时间缓缓飘移
        let height_mask=smoothstep_f(0.2,0.4,dir.y)*(1.-smoothstep_f(0.7,0.85,dir.y));
        // 2D 噪声采样 (view_dir.xz 天然是连续球面坐标)
        let cloud_raw=cloud_fbm(Vec2::new(dir.x*3.+t*0.012, dir.z*3.+t*0.008), se);
        let cloud=cloud_raw*height_mask;
        let cloud_vis=smoothstep_f(-0.1,0.2,se); // 夜间消散
        let c=[(sky[0]*(1.-cloud)+cloud*0.95*cloud_vis).min(1.),
               (sky[1]*(1.-cloud)+cloud*0.85*cloud_vis).min(1.),
               (sky[2]*(1.-cloud)+cloud*0.7*cloud_vis).min(1.),1.];
        colors.push(c);
    }
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR,colors);
}

// ── Simplex 2D 梯度噪声（比hash噪声平滑，适合云层） ──────
fn hash2d(x:f32,y:f32)->f32{let h=(x*12.9898+y*78.233).sin()*43758.5453;h-h.floor()}
fn grad(h:f32,x:f32,y:f32)->f32{let a=h*6.283185;let(s,c)=(a.sin(),a.cos());s*x+c*y}
fn simplex2d(p:Vec2)->f32{
    let f=0.3660254;let s=(p.x+p.y)*f;let i=(p.x+s).floor();let j=(p.y+s).floor();
    let g=0.21132487;let t=(i+j)*g;let x0=p.x-i+t;let y0=p.y-j+t;
    let(i1,j1)=if x0>y0{(1.,0.)}else{(0.,1.)};
    let x1=x0-i1+g;let y1=y0-j1+g;let x2=x0-1.+2.*g;let y2=y0-1.+2.*g;
    let n0=0.5-x0*x0-y0*y0;let n1=0.5-x1*x1-y1*y1;let n2=0.5-x2*x2-y2*y2;
    let mut v=0.;
    if n0>0.{let t_=n0*n0;v+=t_*t_*grad(hash2d(i,j),x0,y0);}
    if n1>0.{let t_=n1*n1;v+=t_*t_*grad(hash2d(i+i1,j+j1),x1,y1);}
    if n2>0.{let t_=n2*n2;v+=t_*t_*grad(hash2d(i+1.,j+1.),x2,y2);}
    v*70.
}
fn fbm2d(p:Vec2)->f32{
    let mut v=0.;let mut a=0.5;let mut f=1.;
    for _ in 0..4{v+=a*simplex2d(p*f);f*=2.3;a*=0.45;}
    v
}
fn cloud_fbm(p:Vec2, sun_elev:f32)->f32{
    let n=fbm2d(p);
    let threshold=0.35-sun_elev*0.1;
    ((n-threshold)/(1.-threshold)).clamp(0.,1.)
}

/// 天空着色器——时间轴+空间轴双重插值
fn sky_shader(view_dir:Vec3, sun_elev:f32, sun_dir:Vec3)->[f32;3]{
    let view_h=view_dir.y.clamp(0.,1.); // 0=地平线,1=天顶
    let se=sun_elev.clamp(-1.,1.);

    // 三套预设颜色
    let day_zenith=[0.05,0.2,0.6];    let day_horizon=[0.5,0.7,0.9];
    let sunset_zenith=[0.1,0.08,0.25]; let sunset_horizon=[1.0,0.35,0.05];
    let night_zenith=[0.0,0.0,0.02];   let night_horizon=[0.01,0.04,0.08];

    // 权重：三套预设按太阳高度平滑混合
    let sunset_w=1.0-smoothstep_f(0.0,0.2,se.abs());
    let day_w=smoothstep_f(0.0,0.25,se);
    let night_w=smoothstep_f(0.0,0.2,-se);

    let zenith=[day_zenith[0]*day_w+sunset_zenith[0]*sunset_w+night_zenith[0]*night_w,
                day_zenith[1]*day_w+sunset_zenith[1]*sunset_w+night_zenith[1]*night_w,
                day_zenith[2]*day_w+sunset_zenith[2]*sunset_w+night_zenith[2]*night_w];
    let horizon=[day_horizon[0]*day_w+sunset_horizon[0]*sunset_w+night_horizon[0]*night_w,
                 day_horizon[1]*day_w+sunset_horizon[1]*sunset_w+night_horizon[1]*night_w,
                 day_horizon[2]*day_w+sunset_horizon[2]*sunset_w+night_horizon[2]*night_w];

    // 空间插值：天顶↔地平线 (非线性)
    let t=view_h.powf(0.65);
    let mut c=[zenith[0]*(1.-t)+horizon[0]*t,
               zenith[1]*(1.-t)+horizon[1]*t,
               zenith[2]*(1.-t)+horizon[2]*t];

    // 太阳光晕
    let sun_dot=view_dir.dot(sun_dir).max(0.);
    let glow=sun_dot.powf(20.)*sunset_w*0.3;
    c[0]=(c[0]+glow).min(1.);c[1]=(c[1]+glow*0.6).min(1.);c[2]=(c[2]+glow*0.2).min(1.);
    c
}
fn smoothstep_f(e0:f32,e1:f32,x:f32)->f32{let t=((x-e0)/(e1-e0)).clamp(0.,1.);t*t*(3.-2.*t)}

// ── 动态太阳 spawn ─────────────────────────────────────
fn spawn_dynamic_sun(commands:&mut Commands,meshes:&mut ResMut<Assets<Mesh>>,mats:&mut ResMut<Assets<StandardMaterial>>){
    let r=SUN_R;let sub=3;
    // 生成 icosphere 并提取原始顶点
    let Ok(base)=Sphere::new(r).mesh().ico(sub)else{return};
    let Some(VertexAttributeValues::Float32x3(base_pos))=base.attribute(Mesh::ATTRIBUTE_POSITION)else{return};
    let Some(indices)=base.indices()else{return};
    let Indices::U32(idx)=indices else {return};

    // 复制顶点实现平面着色
    let mut new_pos=vec![];let mut new_nor=vec![];let mut new_col=vec![];let mut new_idx=vec![];
    let mut orig=vec![];
    for t in idx.chunks(3){
        if t.len()<3{continue}
        let p0=Vec3::from(base_pos[t[0]as usize]);let p1=Vec3::from(base_pos[t[1]as usize]);let p2=Vec3::from(base_pos[t[2]as usize]);
        let n=(p1-p0).cross(p2-p0).normalize();
        let bi=new_pos.len()as u32;
        for &vi in t{
            new_pos.push(base_pos[vi as usize]);new_nor.push([n.x,n.y,n.z]);
            new_col.push([1.,0.8,0.2,1.]);orig.push(Vec3::from(base_pos[vi as usize]));
        }
        new_idx.extend_from_slice(&[bi,bi+1,bi+2]);
    }
    let mut m=Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION,new_pos);
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL,new_nor);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR,new_col);
    m.insert_indices(Indices::U32(new_idx));

    let mat=StandardMaterial{base_color:Color::WHITE,unlit:false,..default()};
    commands.spawn((Mesh3d(meshes.add(m)),MeshMaterial3d(mats.add(mat)),Sun,
        DynamicSun{original_positions:orig,max_spike:r*0.18}));
}

// ── 动态太阳动画（双层噪声：高频密度 + 低频错落） ──────
fn animate_sun(
    time:Res<Time>,day:Res<DayCycle>,
    q:Query<(&DynamicSun,&Mesh3d)>,
    mut meshes:ResMut<Assets<Mesh>>,
){
    let tf=time.elapsed_secs()*1.5;let ts=time.elapsed_secs()*0.5;
    let se=sun_elev(day.tick).max(0.1);
    for(sun,mesh_handle)in q.iter(){
        let Some(mesh)=meshes.get_mut(&mesh_handle.0)else{continue};
        let Some(VertexAttributeValues::Float32x3(cur_pos))=mesh.attribute(Mesh::ATTRIBUTE_POSITION)else{continue};
        let mut new_pos=Vec::with_capacity(cur_pos.len());
        let mut new_col=Vec::with_capacity(cur_pos.len());
        for orig in sun.original_positions.iter(){
            let dir=orig.normalize();
            // 高频层——决定尖刺密集度
            let fh=14.;let nh=((simplex3d(Vec3::new(dir.x*fh+tf,dir.y*fh,dir.z*fh-tf))+1.)*0.5).powf(12.);
            // 低频层——决定错落感（某些区域尖刺高，某些平坦）
            let fl=3.;let nl=(simplex3d(Vec3::new(dir.x*fl-ts,dir.y*fl+ts,dir.z*fl))+1.)*0.5;
            let spike=nh*(nl+0.1);
            let disp=spike*sun.max_spike*se;
            new_pos.push((*orig+dir*disp).to_array());
            // 色彩：深橙红基底(0.8,0.15,0)→尖刺顶端亮黄白(3.0,2.5,0.8)
            let tc=(disp/sun.max_spike).clamp(0.,1.);
            new_col.push([0.8+tc*2.2,0.15+tc*2.35,tc*0.8,1.]);
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION,new_pos);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR,new_col);
        recompute_flat_normals(mesh);
    }
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
