mod light_2d;
mod render;

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::UniformComponentPlugin, render_phase::AddRenderCommand,
        render_resource::SpecializedRenderPipelines, RenderApp, RenderStage,
    },
};

pub use light_2d::*;

use render::{DrawLight, Light2dPipeline, Light2dUniform, LightMeta};

#[derive(Default)]
pub struct Light2dPlugin;

pub const LIGHT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597128);
pub const SHADOW_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597129);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum LightSystem {
    ExtractLights,
}

impl Plugin for Light2dPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        let light_shader = Shader::from_wgsl(include_str!("render/light.wgsl"));
        shaders.set_untracked(LIGHT_SHADER_HANDLE, light_shader);
        let shadow_shader = Shader::from_wgsl(include_str!("render/shadow.wgsl"));
        shaders.set_untracked(SHADOW_SHADER_HANDLE, shadow_shader);

        app.register_type::<PointLight2d>()
            .add_plugin(UniformComponentPlugin::<Light2dUniform>::default());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<Light2dPipeline>()
                .init_resource::<SpecializedRenderPipelines<Light2dPipeline>>()
                .init_resource::<LightMeta>()
                .add_render_command::<Transparent2d, DrawLight>()
                .add_system_to_stage(
                    RenderStage::Extract,
                    render::extract_lights.label(LightSystem::ExtractLights),
                )
                .add_system_to_stage(RenderStage::Extract, render::extract_cameras)
                .add_system_to_stage(RenderStage::Queue, render::queue_light_bind_group)
                .add_system_to_stage(RenderStage::Queue, render::queue_lights);
        };
    }
}
