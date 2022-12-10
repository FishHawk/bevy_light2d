use bevy::{
    asset::HandleId,
    core_pipeline::{core_2d::Transparent2d, tonemapping::Tonemapping},
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem, SystemState,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_asset::RenderAssets,
        render_phase::{
            BatchedPhaseItem, DrawFunctions, EntityRenderCommand, RenderCommand,
            RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages,
            BufferVec, PipelineCache, ShaderType, SpecializedRenderPipeline,
            SpecializedRenderPipelines,
        },
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, ColorTargetState, ColorWrites, Extent3d, FragmentState,
            FrontFace, ImageCopyTexture, ImageDataLayout, MultisampleState, Origin3d, PolygonMode,
            PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor, SamplerBindingType,
            ShaderStages, TextureAspect, TextureDimension, TextureFormat, TextureSampleType,
            TextureViewDescriptor, TextureViewDimension, VertexBufferLayout, VertexFormat,
            VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{
            BevyDefault, DefaultImageSampler, GpuImage, ImageSampler, TextureFormatPixelInfo,
        },
        view::ViewUniform,
        view::{ExtractedView, ViewUniformOffset, ViewUniforms, VisibleEntities},
        Extract,
    },
    utils::{FloatOrd, HashMap},
};
use std::f32::consts::E;

use bytemuck::{Pod, Zeroable};

use super::Light2dOverlay;

pub const OVERLAY_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597138);

#[derive(Resource)]
pub struct Light2dOverlayPipeline {
    pub view_layout: BindGroupLayout,
    pub material_layout: BindGroupLayout,
}

impl FromWorld for Light2dOverlayPipeline {
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
            label: Some("light_overlay_view_layout"),
        });

        let material_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("light2d_meterial_layout"),
        });

        Self {
            view_layout,
            material_layout,
        }
    }
}

impl SpecializedRenderPipeline for Light2dOverlayPipeline {
    type Key = u32;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let formats = vec![
            VertexFormat::Float32x2, // uv
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: OVERLAY_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "vertex".into(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: OVERLAY_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: Some(vec![self.view_layout.clone(), self.material_layout.clone()]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("light2d_overlay_pipeline".into()),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct OverlayVertex {
    pub uv: [f32; 2],
}

#[derive(Resource)]
pub struct OverlayMeta {
    vertices: BufferVec<OverlayVertex>,
    view_bind_group: Option<BindGroup>,
}

impl Default for OverlayMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

const QUAD_INDICES: [usize; 6] = [0, 2, 3, 0, 1, 2];

const QUAD_UVS: [Vec2; 4] = [
    Vec2::new(0., 1.),
    Vec2::new(1., 1.),
    Vec2::new(1., 0.),
    Vec2::new(0., 0.),
];

#[derive(Component, Eq, PartialEq, Copy, Clone)]
pub struct OverlayBatch {
    image_handle_id: HandleId,
}

#[derive(Resource, Default)]
pub struct OverlayImageBindGroups {
    values: HashMap<Handle<Image>, BindGroup>,
}

#[allow(clippy::too_many_arguments)]
pub fn queue_light_overlay_bind_group(
    mut commands: Commands,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut overlay_meta: ResMut<OverlayMeta>,
    view_uniforms: Res<ViewUniforms>,
    overlay_pipeline: Res<Light2dOverlayPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<Light2dOverlayPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut image_bind_groups: ResMut<OverlayImageBindGroups>,
    gpu_images: Res<RenderAssets<Image>>,
    mut views: Query<(
        &mut RenderPhase<Transparent2d>,
        &mut VisibleEntities,
        &ExtractedView,
        Option<&Tonemapping>,
        &Children,
    )>,
    mut child_query: Query<&Light2dOverlay>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        let overlay_meta = &mut overlay_meta;

        overlay_meta.vertices.clear();

        overlay_meta.view_bind_group =
            Some(render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding,
                }],
                label: Some("overlay_view_bind_group"),
                layout: &overlay_pipeline.view_layout,
            }));

        let draw_overlay_function = draw_functions.read().get_id::<DrawOverlay>().unwrap();

        let image_bind_groups = &mut *image_bind_groups;

        let mut index = 0;
        for (mut transparent_phase, mut visible_entities, view, tonemapping, children) in &mut views
        {
            let pipeline = pipelines.specialize(&mut pipeline_cache, &overlay_pipeline, 0);
            for overlay in &mut child_query {
                // Set-up a new possible batch
                let image_handle_id = overlay.image.id();
                if let Some(gpu_image) = gpu_images.get(&Handle::weak(image_handle_id)) {
                    let current_batch_entity =
                        commands.spawn(OverlayBatch { image_handle_id }).id();
                    visible_entities.entities.push(current_batch_entity.clone());

                    image_bind_groups
                        .values
                        .entry(Handle::weak(image_handle_id))
                        .or_insert_with(|| {
                            render_device.create_bind_group(&BindGroupDescriptor {
                                entries: &[
                                    BindGroupEntry {
                                        binding: 0,
                                        resource: BindingResource::TextureView(
                                            &gpu_image.texture_view,
                                        ),
                                    },
                                    BindGroupEntry {
                                        binding: 1,
                                        resource: BindingResource::Sampler(&gpu_image.sampler),
                                    },
                                ],
                                label: Some("sprite_material_bind_group"),
                                layout: &overlay_pipeline.material_layout,
                            })
                        });

                    for i in QUAD_INDICES {
                        overlay_meta.vertices.push(OverlayVertex {
                            uv: QUAD_UVS[i].into(),
                        });
                    }
                    let sort_key = FloatOrd(100.0);
                    let item_start = index;
                    index += QUAD_INDICES.len() as u32;
                    let item_end = index;
                    transparent_phase.add(Transparent2d {
                        draw_function: draw_overlay_function,
                        pipeline,
                        entity: current_batch_entity,
                        sort_key,
                        batch_range: Some(item_start..item_end),
                    });
                }
            }
        }
        overlay_meta
            .vertices
            .write_buffer(&render_device, &render_queue);
    }
}

pub type DrawOverlay = (
    SetItemPipeline,
    SetOverlayViewBindGroup<0>,
    SetOverlayTextureBindGroup<1>,
    DrawOverlayBatch,
);

pub struct SetOverlayViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetOverlayViewBindGroup<I> {
    type Param = (SRes<OverlayMeta>, SQuery<Read<ViewUniformOffset>>);

    fn render<'w>(
        view: Entity,
        _item: Entity,
        (sprite_meta, view_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let view_uniform = view_query.get(view).unwrap();
        pass.set_bind_group(
            I,
            sprite_meta.into_inner().view_bind_group.as_ref().unwrap(),
            &[view_uniform.offset],
        );
        RenderCommandResult::Success
    }
}
pub struct SetOverlayTextureBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetOverlayTextureBindGroup<I> {
    type Param = (SRes<OverlayImageBindGroups>, SQuery<Read<OverlayBatch>>);

    fn render<'w>(
        _view: Entity,
        item: Entity,
        (image_bind_groups, query_batch): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_batch = query_batch.get(item).unwrap();
        let image_bind_groups = image_bind_groups.into_inner();

        pass.set_bind_group(
            I,
            image_bind_groups
                .values
                .get(&Handle::weak(sprite_batch.image_handle_id))
                .unwrap(),
            &[],
        );
        RenderCommandResult::Success
    }
}

pub struct DrawOverlayBatch;
impl<P: BatchedPhaseItem> RenderCommand<P> for DrawOverlayBatch {
    type Param = (SRes<OverlayMeta>, SQuery<Read<OverlayBatch>>);

    fn render<'w>(
        _view: Entity,
        item: &P,
        (sprite_meta, query_batch): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_meta = sprite_meta.into_inner();
        pass.set_vertex_buffer(0, sprite_meta.vertices.buffer().unwrap().slice(..));
        pass.draw(item.batch_range().as_ref().unwrap().clone(), 0..1);
        RenderCommandResult::Success
    }
}
