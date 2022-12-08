struct View {
    view_proj: mat4x4<f32>,
    light_position: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> view: View;

struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex(
    @location(0) vertex_position: vec3<f32>,
){
    // Transform the position.
    // If you are batching the shadow data, you can pre-apply the transform instead.
    let pos_xy = (view.view_proj * vec4<f32>(vertex_position.xy, 0.0, 1.0)).xy;

    // When a_vertex.z is 0, the vertex is on the near side of the shadow and is output as is.
    // When a_vertex.z is 1, the vertex is on the far side of the shadow as is projected to inifity.
    let pos_xyzw = vec4(pos_xy - vertex_position.z * view.light_position, 0.0, 1.0 - vertex_position.z);

    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = pos_xyzw;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0);
}