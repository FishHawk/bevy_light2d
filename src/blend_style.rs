use bevy::render::render_resource::*;

pub struct BlendStyle {
    pub state: BlendState,
    pub render_texture_scale: f32,
}

pub const Multiply: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent::OVER,
};

pub const ADDITIVE: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent::OVER,
};

pub const SUBTRACT: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Subtract,
    },
    alpha: BlendComponent::OVER,
};
