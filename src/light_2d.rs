use bevy::{
    prelude::{Color, Component, Vec2},
    reflect::Reflect,
};

#[derive(Component, Debug, Clone, Reflect)]
#[repr(C)]
pub struct PointLight2D {
    pub color: Color,
    pub falloff_intensity: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
    pub inner_radius: f32,
}

impl Default for PointLight2D {
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
