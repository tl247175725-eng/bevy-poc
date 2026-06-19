// Low-Poly 月亮 Flat Shading + 3D 噪声坑洼 + 边缘微光

#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct MoonUniforms {
    base_color: vec4<f32>,
    crater_color: vec4<f32>,
    emissive_intensity: f32,
};

@group(2) @binding(0) var<uniform> material: MoonUniforms;

fn hash3(p: vec3<f32>) -> f32 {
    let h = dot(p, vec3<f32>(127.1, 311.7, 74.7));
    return fract(sin(h) * 43758.5453);
}

fn noise3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(mix(hash3(i+vec3<f32>(0.,0.,0.)), hash3(i+vec3<f32>(1.,0.,0.)), u.x),
            mix(hash3(i+vec3<f32>(0.,1.,0.)), hash3(i+vec3<f32>(1.,1.,0.)), u.x), u.y),
        mix(mix(hash3(i+vec3<f32>(0.,0.,1.)), hash3(i+vec3<f32>(1.,0.,1.)), u.x),
            mix(hash3(i+vec3<f32>(0.,1.,1.)), hash3(i+vec3<f32>(1.,1.,1.)), u.x), u.y),
        u.z,
    );
}

fn fbm(p: vec3<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    for (var i = 0u; i < 4u; i++) {
        val += amp * noise3(p * freq);
        freq *= 2.0;
        amp *= 0.5;
    }
    return val;
}

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let dx = dpdx(in.world_position.xyz);
    let dy = dpdy(in.world_position.xyz);
    let flat_normal = normalize(cross(dx, dy));

    let V = normalize(view.world_position.xyz - in.world_position.xyz);
    let NdotV = abs(dot(flat_normal, V));

    let craters = fbm(in.world_position.xyz * 0.04);
    let base = mix(material.crater_color.rgb, material.base_color.rgb, smoothstep(0.3, 0.7, craters));
    let rim = pow(1.0 - NdotV, 3.0) * 0.3;
    let final_color = (base + rim) * material.emissive_intensity;

    return vec4<f32>(final_color, 1.0);
}
