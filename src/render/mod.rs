pub mod light;
pub mod overlay;
pub mod shadow;

use std::ops::Deref;

use bevy::{
    core_pipeline::{clear_color::ClearColorConfig, core_2d::Transparent2d},
    prelude::*,
    render::{
        camera::{CameraRenderGraph, ExtractedCamera, RenderTarget},
        render_phase::RenderPhase,
        view::{ExtractedView, VisibleEntities},
        Extract,
    },
};

pub use light::*;

#[derive(Component, Clone)]
pub struct Light2dOverlay {
    pub image: Handle<Image>,
    pub size: UVec2,
}

pub fn extract_cameras(
    mut commands: Commands,
    query: Extract<
        Query<
            (
                Entity,
                &Camera,
                &CameraRenderGraph,
                &GlobalTransform,
                &VisibleEntities,
                &Children,
            ),
            With<Camera2d>,
        >,
    >,
    child_query: Extract<Query<&Light2dOverlay>>,
) {
    for (parent, camera, camera_render_graph, transform, visible_entities, children) in query.iter()
    {
        if !camera.is_active {
            continue;
        }
        if let (Some((viewport_origin, _)), Some(viewport_size), Some(target_size)) = (
            camera.physical_viewport_rect(),
            camera.physical_viewport_size(),
            camera.physical_target_size(),
        ) {
            if target_size.x == 0 || target_size.y == 0 {
                continue;
            }

            for child in children.iter() {
                if let Ok(overlay) = child_query.get(child.clone()) {
                    commands.get_or_spawn(child.clone()).insert((
                        ExtractedCamera {
                            target: RenderTarget::Image(overlay.image.clone()),
                            viewport: camera.viewport.clone(),
                            physical_viewport_size: Some(viewport_size),
                            physical_target_size: Some(overlay.size),
                            render_graph: camera_render_graph.deref().clone(),
                            priority: camera.priority - 1,
                        },
                        ExtractedView {
                            projection: camera.projection_matrix(),
                            transform: *transform,
                            hdr: camera.hdr,
                            viewport: UVec4::new(
                                viewport_origin.x,
                                viewport_origin.y,
                                viewport_size.x,
                                viewport_size.y,
                            ),
                        },
                        RenderPhase::<Transparent2d>::default(),
                        Camera2d {
                            clear_color: ClearColorConfig::Custom(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                        },
                        overlay.clone(),
                    ));
                    commands
                        .get_or_spawn(parent)
                        .push_children(&[child.clone()]);
                }
            }
        }
    }
}
