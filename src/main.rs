use bevy::prelude::*;
use city::{Light2DPlugin, PointLight2D};

fn main() {
    const BACKGROUND_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 400.0,
                height: 400.0,
                ..default()
            },
            ..default()
        }))
        .add_plugin(Light2DPlugin)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Lights
    for (x, y, color) in [
        (0.0, 0.0, Color::YELLOW),
        (-200.0, -200.0, Color::BLUE),
        (200.0, -200.0, Color::RED),
        //
    ] {
        commands.spawn((
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(x, y, 0.0),
                    scale: Vec3::new(200.0, 200.0, 0.0),
                    ..default()
                },
                ..default()
            },
            PointLight2D {
                color,
                falloff_intensity: Vec2::new(1.0, 1.0),
                inner_angle: 0.0,
                outer_angle: 0.5,
                inner_radius: 0.5,
            },
        ));
    }
}
