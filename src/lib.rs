mod light_2d;
pub mod render;

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{
        extract_component::UniformComponentPlugin, render_phase::AddRenderCommand,
        render_resource::SpecializedRenderPipelines, RenderApp, RenderStage,
    },
};

pub use light_2d::*;

use render::{
    light::{Light2dPipeline, Light2dUniform, LIGHT_SHADER_HANDLE},
    overlay::{
        DrawOverlay, Light2dOverlayPipeline, OverlayImageBindGroups, OverlayMeta,
        OVERLAY_SHADER_HANDLE, queue_light_overlay_bind_group,
    },
};
use render::{DrawLight, LightMeta};

#[derive(Default)]
pub struct Light2dPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum LightSystem {
    ExtractLights,
}

impl Plugin for Light2dPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        let light_shader = Shader::from_wgsl(include_str!("render/light.wgsl"));
        shaders.set_untracked(LIGHT_SHADER_HANDLE, light_shader);
        let overlay_shader = Shader::from_wgsl(include_str!("render/overlay.wgsl"));
        shaders.set_untracked(OVERLAY_SHADER_HANDLE, overlay_shader);
        // let shadow_shader = Shader::from_wgsl(include_str!("render/shadow.wgsl"));
        // shaders.set_untracked(SHADOW_SHADER_HANDLE, shadow_shader);

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
                .add_system_to_stage(RenderStage::Queue, render::queue_light_bind_group)
                .add_system_to_stage(RenderStage::Queue, render::queue_lights)
                //
                .init_resource::<OverlayImageBindGroups>()
                .init_resource::<Light2dOverlayPipeline>()
                .init_resource::<SpecializedRenderPipelines<Light2dOverlayPipeline>>()
                .init_resource::<OverlayMeta>()
                .add_render_command::<Transparent2d, DrawOverlay>()
                .add_system_to_stage(RenderStage::Extract, render::extract_cameras)
                .add_system_to_stage(RenderStage::Queue, queue_light_overlay_bind_group);
        };
    }
}
