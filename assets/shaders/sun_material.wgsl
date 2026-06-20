@group(2) @binding(0) var<uniform> material_color: vec4<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return material_color;
}
