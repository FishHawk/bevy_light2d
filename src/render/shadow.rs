use bevy::{
    ecs::system::SystemState,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, ColorTargetState, ColorWrites, Extent3d, FragmentState,
            FrontFace, ImageCopyTexture, ImageDataLayout, MultisampleState, Origin3d, PolygonMode,
            PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor, SamplerBindingType,
            ShaderStages, ShaderType, SpecializedRenderPipeline, TextureAspect, TextureDimension,
            TextureFormat, TextureSampleType, TextureViewDescriptor, TextureViewDimension,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{
            BevyDefault, DefaultImageSampler, GpuImage, ImageSampler, TextureFormatPixelInfo,
        },
        view::ViewUniform,
    },
};
use std::f32::consts::E;

pub const SHADOW_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597129);
