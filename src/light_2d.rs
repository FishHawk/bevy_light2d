use bevy::{
    prelude::{Color, Component, Vec2},
    reflect::{Reflect, TypeUuid},
};

#[derive(Component, Debug, Clone, Reflect)]
#[repr(C)]
pub struct PointLight2d {
    pub color: Color,
    pub falloff_intensity: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
    pub inner_radius: f32,
}

impl Default for PointLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            falloff_intensity: 1.0,
            inner_angle: 1.0,
            outer_angle: 1.0,
            inner_radius: 1.0,
        }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[repr(C)]
// #[derive(Debug, TypeUuid, Clone)]
// #[uuid = "b7e962fa-c102-4369-ac86-ea026a9aa1b3"]
pub struct Shadow2d {
    pub closed: bool,
    // cull_mode
    pub points: Vec<Vec2>,
}
