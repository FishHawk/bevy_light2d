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
@group(0) @binding(0)
var<uniform> view: View;

struct Light {
    light_position: vec3<f32>,
    light_color: vec4<f32>,
    outer_angle: f32,
    inner_radius_mult: f32,
    inner_angle_mult: f32,
    is_full_angle: f32,
}
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
    // output.positionCS = TransformObjectToHClip(input.positionOS);
    // output.uv = input.texcoord;

    // float4 worldSpacePos;
    // worldSpacePos.xyz = TransformObjectToWorld(input.positionOS);
    // worldSpacePos.w = 1;

    // float4 lightSpacePos = mul(_LightInvMatrix, worldSpacePos);
    // float4 lightSpaceNoRotPos = mul(_LightNoRotInvMatrix, worldSpacePos);
    // float halfTexelOffset = 0.5 * _LightLookup_TexelSize.x;
    // output.lookupUV = 0.5 * (lightSpacePos.xy + 1) + halfTexelOffset;

    // TRANSFER_NORMALS_LIGHTING(output, worldSpacePos)
    // TRANSFER_SHADOWS(output)

@group(2) @binding(0)
var light_lookup_texture: texture_2d<f32>;
@group(2) @binding(1)
var light_lookup_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // r = distance, g = angle, b = x direction, a = y direction
    let lookup = textureSample(light_lookup_texture, light_lookup_sampler, in.uv);

    // let distance = dot(in.uv, in.uv);
    let distance = lookup.r;
    let radius_attenuation = saturate(light.inner_radius_mult * distance);

    // let cos_angle = dot(vec2<f32>(0.0, 1.0) ,normalize(in.uv));
    // let angle = acos(cos_angle) / 3.14159;
    let angle = lookup.g;
    // let angle_attenuation = saturate((light.outer_angle - angle + light.is_full_angle) * light.inner_angle_mult);
    let angle_attenuation = saturate((light.outer_angle - angle) * light.inner_angle_mult);

    let attenuation = radius_attenuation * angle_attenuation;

    // half2 mappedUV;
    // mappedUV.x = attenuation;
    // mappedUV.y = _FalloffIntensity;
    // attenuation = SAMPLE_TEXTURE2D(_FalloffLookup, sampler_FalloffLookup, mappedUV).r;

    var light_color = light.light_color;
    light_color.a = attenuation;

    return vec4<f32>(attenuation, 0.0,0.0,1.0);
    // return light_color;
}

// r = distance, g = angle, b = x direction, a = y direction

//                 half2 mappedUV;
//                 mappedUV.x = attenuation;
//                 mappedUV.y = _FalloffIntensity;
//                 attenuation = SAMPLE_TEXTURE2D(_FalloffLookup, sampler_FalloffLookup, mappedUV).r;

//                 half4 lightColor = _LightColor;

// #if USE_ADDITIVE_BLENDING
//                 lightColor *= attenuation;
// #else
//                 lightColor.a = attenuation;
// #endif

//                 APPLY_NORMALS_LIGHTING(input, lightColor);
//                 APPLY_SHADOWS(input, lightColor, _ShadowIntensity);

//                 return lightColor * _InverseHDREmulationScale;