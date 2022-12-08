struct View {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    world_position: vec3<f32>,
    // viewport(x_origin, y_origin, width, height)
    viewport: vec4<f32>,
};
struct Light {
    light_position: vec3<f32>,
    light_color: vec4<f32>,
    falloff_intensity: f32,
    outer_angle: f32,
    inner_radius_mult: f32,
    inner_angle_mult: f32,
    is_full_angle: f32,
}

@group(0) @binding(0)
var<uniform> view: View;

@group(1) @binding(0)
var<uniform> light: Light;

struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex(
    @location(0) vertex_position: vec3<f32>,
    @location(1) vertex_uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}

@group(2) @binding(0)
var falloff_lookup_texture: texture_2d<f32>;
@group(2) @binding(1)
var falloff_lookup_sampler: sampler;

@group(3) @binding(0)
var light_lookup_texture: texture_2d<f32>;
@group(3) @binding(1)
var light_lookup_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // r = distance, g = angle, b = x direction, a = y direction
    let lookup = textureSample(light_lookup_texture, light_lookup_sampler, in.uv);

    let distance = lookup.r;
    let radius_attenuation = saturate(light.inner_radius_mult * distance);

    let angle = lookup.g;
    let angle_attenuation = saturate((light.outer_angle - angle) * light.inner_angle_mult);
    let angle_attenuation = saturate( angle * light.inner_angle_mult);

    let attenuation = radius_attenuation * angle_attenuation;

    let attenuation = textureSample(
        falloff_lookup_texture,
        falloff_lookup_sampler,
        vec2<f32>(attenuation, light.falloff_intensity)
    ).r;

    var light_color = light.light_color;
    light_color.a *= attenuation;

    // #if USE_ADDITIVE_BLENDING
    // lightColor *= attenuation;
    // #else
    // lightColor.a = attenuation;
    // #endif

    // APPLY_NORMALS_LIGHTING(input, lightColor);
    // APPLY_SHADOWS(input, lightColor, _ShadowIntensity);

    return light_color;
}
