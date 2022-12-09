use std::{f32::consts::E, sync::atomic::AtomicUsize};

use bevy::{
    core_pipeline::{
        clear_color::ClearColorConfig, core_2d::Transparent2d, tonemapping::Tonemapping,
    },
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem, SystemState,
    },
    prelude::*,
    render::{
        camera::{CameraRenderGraph, ExtractedCamera},
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_asset::RenderAssets,
        render_phase::{
            BatchedPhaseItem, DrawFunctions, EntityRenderCommand, RenderCommand,
            RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            BlendState, BufferBindingType, BufferUsages, BufferVec, ColorTargetState, ColorWrites,
            Extent3d, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout,
            MultisampleState, Origin3d, PipelineCache, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineDescriptor, SamplerBindingType, ShaderStages,
            ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines, TextureAspect,
            TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
            TextureViewDescriptor, TextureViewDimension, VertexBufferLayout, VertexFormat,
            VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{
            BevyDefault, DefaultImageSampler, GpuImage, ImageSampler, TextureCache,
            TextureFormatPixelInfo,
        },
        view::{
            ExtractedView, ExtractedWindows, ViewTarget, ViewUniform, ViewUniformOffset,
            ViewUniforms, VisibleEntities,
        },
        Extract,
    },
    utils::{FloatOrd, HashMap},
};
use bytemuck::{Pod, Zeroable};

use crate::{PointLight2d, LIGHT_SHADER_HANDLE, SHADOW_SHADER_HANDLE};

#[derive(Component, ShaderType, Clone)]
pub struct Light2dUniform {
    light_position: Vec3,
    light_color: Vec4,
    falloff_intensity: f32,
    outer_angle: f32,
    inner_radius_mult: f32,
    inner_angle_mult: f32,
    is_full_angle: f32,
}

#[derive(Resource)]
pub struct Light2dPipeline {
    view_layout: BindGroupLayout,
    light_layout: BindGroupLayout,
    falloff_lookup_layout: BindGroupLayout,
    falloff_lookup_gpu_image: GpuImage,
    point_light_lookup_layout: BindGroupLayout,
    point_light_lookup_gpu_image: GpuImage,
}

impl FromWorld for Light2dPipeline {
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
        let light_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(Light2dUniform::min_size()),
                },
                count: None,
            }],
            label: Some("light_light_layout"),
        });

        let falloff_lookup_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
                label: Some("light2d_falloff_lookup_texture_layout"),
            });

        let point_light_lookup_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
                label: Some("light2d_point_light_lookup_texture_layout"),
            });

        let point_light_lookup_gpu_image = create_gpu_image_from_image(
            create_point_light_lookup_image(),
            &render_device,
            &default_sampler,
            &render_queue,
        );

        let falloff_lookup_gpu_image = create_gpu_image_from_image(
            create_falloff_lookup_image(),
            &render_device,
            &default_sampler,
            &render_queue,
        );

        Self {
            view_layout,
            light_layout,
            falloff_lookup_layout,
            falloff_lookup_gpu_image,
            point_light_lookup_layout,
            point_light_lookup_gpu_image,
        }
    }
}

fn create_falloff_lookup_image() -> Image {
    const WIDTH: usize = 2048;
    const HEIGHT: usize = 128;
    let mut data = Vec::with_capacity(WIDTH * HEIGHT * 4);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let alpha: f32 = x as f32 / WIDTH as f32;
            let intensity: f32 = y as f32 / HEIGHT as f32;
            let falloff = alpha.powf(E.powf(1.5 - 3.0 * intensity));
            for u in falloff.to_bits().to_le_bytes() {
                data.push(u);
            }
        }
    }
    Image::new_fill(
        Extent3d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data[..],
        TextureFormat::R32Float,
    )
}

fn create_point_light_lookup_image() -> Image {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    let mut data = Vec::with_capacity(WIDTH * HEIGHT * 4 * 4);
    let center = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let pos = Vec2::new(x as f32, y as f32);
            let distance = Vec2::distance(pos, center);
            let red = if x == WIDTH - 1 || y == HEIGHT - 1 {
                0.0
            } else {
                (1.0 - (2.0 * distance / (WIDTH as f32))).clamp(0.0, 1.0)
            };

            let angle_cos = (pos - center).normalize().y;
            let angle_cos = if angle_cos.is_nan() { 1.0 } else { angle_cos };
            let angle = angle_cos.acos().abs() / std::f32::consts::PI;
            let green = (1.0 - angle).clamp(0.0, 1.0);

            let direction = (center - pos).normalize();
            let blue = direction.x;
            let alpha = direction.y;

            for f in vec![red, green, blue, alpha] {
                for u in f.to_bits().to_le_bytes() {
                    data.push(u);
                }
            }
        }
    }
    Image::new_fill(
        Extent3d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data[..],
        TextureFormat::Rgba32Float,
    )
}

fn create_gpu_image_from_image(
    image: Image,
    render_device: &RenderDevice,
    default_sampler: &DefaultImageSampler,
    render_queue: &RenderQueue,
) -> GpuImage {
    let texture = render_device.create_texture(&image.texture_descriptor);
    let sampler = match image.sampler_descriptor {
        ImageSampler::Default => (**default_sampler).clone(),
        ImageSampler::Descriptor(descriptor) => render_device.create_sampler(&descriptor),
    };
    let format_size = image.texture_descriptor.format.pixel_size();
    render_queue.write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &image.data,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(
                std::num::NonZeroU32::new(image.texture_descriptor.size.width * format_size as u32)
                    .unwrap(),
            ),
            rows_per_image: Some(
                std::num::NonZeroU32::new(
                    image.texture_descriptor.size.height * format_size as u32,
                )
                .unwrap(),
            ),
        },
        image.texture_descriptor.size,
    );
    let texture_view = texture.create_view(&TextureViewDescriptor::default());
    GpuImage {
        texture,
        texture_view,
        texture_format: image.texture_descriptor.format,
        sampler,
        size: Vec2::new(
            image.texture_descriptor.size.width as f32,
            image.texture_descriptor.size.height as f32,
        ),
    }
}

impl SpecializedRenderPipeline for Light2dPipeline {
    type Key = u32;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let formats = vec![
            VertexFormat::Float32x3, // position
            VertexFormat::Float32x2, // uv
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: LIGHT_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "vertex".into(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: LIGHT_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: Some(vec![
                self.view_layout.clone(),
                self.light_layout.clone(),
                self.falloff_lookup_layout.clone(),
                self.point_light_lookup_layout.clone(),
            ]),
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
            label: Some("light_2d_pipeline".into()),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ExtractedPointLight2d {
    pub transform: GlobalTransform,
}

pub fn extract_lights(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    light_query: Extract<Query<(Entity, &ComputedVisibility, &PointLight2d, &GlobalTransform)>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, visibility, light, transform) in light_query.iter() {
        if !visibility.is_visible() {
            continue;
        }
        values.push((
            entity,
            (
                Light2dUniform {
                    light_color: light.color.as_linear_rgba_f32().into(),
                    light_position: transform.translation(),
                    falloff_intensity: light.falloff_intensity,
                    outer_angle: light.outer_angle,
                    inner_radius_mult: 1.0 / (1.0 - light.inner_radius),
                    inner_angle_mult: 1.0 / (light.outer_angle - light.inner_angle),
                    is_full_angle: if light.inner_angle == 1.0 { 1.0 } else { 0.0 },
                },
                ExtractedPointLight2d {
                    transform: *transform,
                },
            ),
        ));
    }

    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

#[derive(Resource)]
pub struct Light2dBindGroup {
    pub value: BindGroup,
    pub falloff_lookup_bind_group: BindGroup,
    pub light_lookup_bind_group: BindGroup,
}

pub fn queue_light_bind_group(
    mut commands: Commands,
    light2d_pipeline: Res<Light2dPipeline>,
    render_device: Res<RenderDevice>,
    light_uniforms: Res<ComponentUniforms<Light2dUniform>>,
) {
    if let Some(binding) = light_uniforms.uniforms().binding() {
        commands.insert_resource(Light2dBindGroup {
            value: render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: binding,
                }],
                label: Some("light_bind_group"),
                layout: &light2d_pipeline.light_layout,
            }),
            falloff_lookup_bind_group: render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(
                            &light2d_pipeline.falloff_lookup_gpu_image.texture_view,
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(
                            &light2d_pipeline.falloff_lookup_gpu_image.sampler,
                        ),
                    },
                ],
                label: Some("light_lookup_bind_group"),
                layout: &light2d_pipeline.point_light_lookup_layout,
            }),
            light_lookup_bind_group: render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(
                            &light2d_pipeline.point_light_lookup_gpu_image.texture_view,
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(
                            &light2d_pipeline.point_light_lookup_gpu_image.sampler,
                        ),
                    },
                ],
                label: Some("light_lookup_bind_group"),
                layout: &light2d_pipeline.point_light_lookup_layout,
            }),
        });
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Light2dVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Resource)]
pub struct LightMeta {
    vertices: BufferVec<Light2dVertex>,
    view_bind_group: Option<BindGroup>,
}

impl Default for LightMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

const QUAD_INDICES: [usize; 6] = [0, 2, 3, 0, 1, 2];

const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
    Vec2::new(-0.5, -0.5),
    Vec2::new(0.5, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

const QUAD_UVS: [Vec2; 4] = [
    Vec2::new(0., 1.),
    Vec2::new(1., 1.),
    Vec2::new(1., 0.),
    Vec2::new(0., 0.),
];

#[allow(clippy::too_many_arguments)]
pub fn queue_lights(
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut light_meta: ResMut<LightMeta>,
    view_uniforms: Res<ViewUniforms>,

    light_pipeline: Res<Light2dPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<Light2dPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    light2d: Query<(&Light2dUniform, &ExtractedPointLight2d)>,
    mut views: Query<(
        &ExtractedView,
        &VisibleEntities,
        Option<&Tonemapping>,
        &mut RenderPhase<Transparent2d>,
    )>,
) {
    if light2d.is_empty() {
        return;
    }
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        let light_meta = &mut light_meta;
        light_meta.vertices.clear();
        light_meta.view_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: view_binding,
            }],
            label: Some("light_view_bind_group"),
            layout: &light_pipeline.view_layout,
        }));

        let draw_sprite_function = draw_functions.read().get_id::<DrawLight>().unwrap();
        let mut colored_index = 0;

        for (view, visible_entities, tonemapping, mut transparent_phase) in &mut views {
            let pipeline = pipelines.specialize(&mut pipeline_cache, &light_pipeline, 0);

            for visible_entity in &visible_entities.entities {
                if let Ok((light_uniform, extracted_light)) = light2d.get(*visible_entity) {
                    // Apply size and global transform
                    let positions = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
                        extracted_light
                            .transform
                            .transform_point(quad_pos.extend(0.))
                            .into()
                    });
                    let uvs = QUAD_UVS.map(|quad_uv| quad_uv.into());

                    // These items will be sorted by depth with other phase items
                    let sort_key = FloatOrd(extracted_light.transform.translation().z);

                    // Store the vertex data and add the item to the render phase
                    for i in QUAD_INDICES {
                        light_meta.vertices.push(Light2dVertex {
                            position: positions[i],
                            uv: uvs[i],
                        });
                    }
                    let item_start = colored_index;
                    colored_index += QUAD_INDICES.len() as u32;
                    let item_end = colored_index;

                    transparent_phase.add(Transparent2d {
                        draw_function: draw_sprite_function,
                        pipeline: pipeline,
                        entity: *visible_entity,
                        sort_key,
                        batch_range: Some(item_start..item_end),
                    });
                }
            }
        }
        light_meta
            .vertices
            .write_buffer(&render_device, &render_queue);
    }
}

pub type DrawLight = (
    SetItemPipeline,
    SetLightViewBindGroup<0>,
    SetSpriteTextureBindGroup<1>,
    SetFalloffLookupBindGroup<2>,
    SetLightLookupBindGroup<3>,
    DrawLightBatch,
);
pub struct SetLightViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetLightViewBindGroup<I> {
    type Param = (SRes<LightMeta>, SQuery<Read<ViewUniformOffset>>);

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

pub struct SetSpriteTextureBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetSpriteTextureBindGroup<I> {
    type Param = (
        SRes<Light2dBindGroup>,
        SQuery<Read<DynamicUniformIndex<Light2dUniform>>>,
    );

    fn render<'w>(
        _view: Entity,
        item: Entity,
        (light_bind_group, light_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let light_index = light_query.get(item).unwrap();
        pass.set_bind_group(
            I,
            &light_bind_group.into_inner().value,
            &[light_index.index()],
        );
        RenderCommandResult::Success
    }
}

pub struct SetLightLookupBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetLightLookupBindGroup<I> {
    type Param = SRes<Light2dBindGroup>;

    fn render<'w>(
        _view: Entity,
        item: Entity,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &bind_groups.into_inner().light_lookup_bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct SetFalloffLookupBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetFalloffLookupBindGroup<I> {
    type Param = SRes<Light2dBindGroup>;

    fn render<'w>(
        _view: Entity,
        item: Entity,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &bind_groups.into_inner().falloff_lookup_bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct DrawLightBatch;
impl<P: BatchedPhaseItem> RenderCommand<P> for DrawLightBatch {
    type Param = SRes<LightMeta>;

    fn render<'w>(
        _view: Entity,
        item: &P,
        light_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_meta = light_meta.into_inner();
        pass.set_vertex_buffer(0, sprite_meta.vertices.buffer().unwrap().slice(..));
        pass.draw(item.batch_range().as_ref().unwrap().clone(), 0..1);
        RenderCommandResult::Success
    }
}
