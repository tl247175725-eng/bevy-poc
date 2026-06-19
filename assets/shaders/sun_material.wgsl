// Low-Poly 太阳 Flat Shading + 颜色渐变 + 自发光

#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct SunUniforms {
    color_center: vec4<f32>,
    color_edge: vec4<f32>,
    emissive_intensity: f32,
};

@group(2) @binding(0) var<uniform> material: SunUniforms;

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let dx = dpdx(in.world_position.xyz);
    let dy = dpdy(in.world_position.xyz);
    let flat_normal = normalize(cross(dx, dy));

    let V = normalize(view.world_position.xyz - in.world_position.xyz);
    let NdotV = abs(dot(flat_normal, V));
    let gradient_factor = smoothstep(0.15, 0.85, NdotV);

    let base_color = mix(
        material.color_edge.rgb,
        material.color_center.rgb,
        gradient_factor,
    );

    let final_color = base_color * material.emissive_intensity;
    return vec4<f32>(final_color, 1.0);
}
