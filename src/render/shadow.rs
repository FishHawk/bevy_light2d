use bevy::{
    ecs::system::SystemState,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction,
            DepthBiasState, DepthStencilState, Extent3d, FragmentState, FrontFace,
            ImageCopyTexture, ImageDataLayout, MultisampleState, Origin3d, PolygonMode,
            PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor, SamplerBindingType,
            ShaderStages, ShaderType, SpecializedRenderPipeline, StencilState, TextureAspect,
            TextureDimension, TextureFormat, TextureSampleType, TextureViewDescriptor,
            TextureViewDimension, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{
            BevyDefault, DefaultImageSampler, GpuImage, ImageSampler, TextureFormatPixelInfo,
        },
        view::ViewUniform,
        Extract,
    },
};
use std::f32::consts::E;

use crate::Shadow2d;

pub const SHADOW_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597129);

#[derive(Resource)]
pub struct Shadow2dPipeline {
    pub view_layout: BindGroupLayout,
}

impl FromWorld for Shadow2dPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut system_state: SystemState<(
            Res<RenderDevice>,
            Res<DefaultImageSampler>,
            Res<RenderQueue>,
        )> = SystemState::new(world);
        let (render_device, default_sampler, render_queue) = system_state.get_mut(world);

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ViewUniform::min_size()),
                },
                count: None,
            }],
            label: Some("light_view_layout"),
        });

        Self { view_layout }
    }
}

impl SpecializedRenderPipeline for Shadow2dPipeline {
    type Key = u32;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let formats = vec![
            VertexFormat::Float32x3, // position
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: SHADOW_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "vertex".into(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: SHADOW_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: Some(vec![self.view_layout.clone()]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: StencilState {
                    front: todo!(),
                    back: todo!(),
                    read_mask: todo!(),
                    write_mask: todo!(),
                },
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("light_2d_pipeline".into()),
        }
    }
}

pub fn extract_shadows(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    light_query: Extract<Query<(Entity, &ComputedVisibility, &Shadow2d, &GlobalTransform)>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, visibility, shadow, transform) in light_query.iter() {
        if !visibility.is_visible() {
            continue;
        }
        values.push((entity, (*transform, shadow.clone())));
    }

    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}
