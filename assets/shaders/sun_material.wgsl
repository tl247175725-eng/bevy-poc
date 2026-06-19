// Low-Poly 太阳材质 — Flat Shading + 颜色渐变 + 自发光

#import bevy_pbr::forward_io::VertexOutput;
#import bevy_pbr::mesh_view_bindings::view;

struct SunMaterial {
    color_center: vec4<f32>,
    color_edge: vec4<f32>,
    emissive_intensity: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: SunMaterial;

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    // 1. Flat Shading — 用屏幕空间偏导数计算真正面法线
    let dx = dpdx(in.world_position.xyz);
    let dy = dpdy(in.world_position.xyz);
    let flat_normal = normalize(abs(cross(dx, dy)));

    // 2. 视线方向
    let V = normalize(view.world_position.xyz - in.world_position.xyz);

    // 3. Fresnel 渐变：正对=亮黄，边缘=橙红
    let NdotV = abs(dot(flat_normal, V));
    let gradient_factor = smoothstep(0.1, 0.9, NdotV);

    let base_color = mix(
        material.color_edge.rgb,
        material.color_center.rgb,
        gradient_factor,
    );

    let final_color = base_color * material.emissive_intensity;

    return vec4<f32>(final_color, 1.0);
}
