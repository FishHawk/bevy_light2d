use bevy::prelude::*;
use city::{Light2DPlugin, PointLight2D};

fn main() {
    const BACKGROUND_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Light2DPlugin)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Sprites
    commands.spawn(SpriteBundle {
        texture: asset_server.load("temp/example.png"),
        ..default()
    });

    // Lights
    for (x, y, color) in [
        (0.0, 0.0, Color::rgba(1.0, 0.0, 0.0, 0.5)),
        // (200.0, 0.0, Color::rgba(1.0, 0.0, 0.0, 0.6)),
        // (400.0, 0.0, Color::rgba(1.0, 0.0, 0.0, 0.6)),
        //
    ] {
        commands.spawn((
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(x, y, 1.0),
                    scale: Vec3::new(1000.0, 1000.0, 0.0),
                    ..default()
                },
                ..default()
            },
            PointLight2D {
                color,
                falloff_intensity: 0.5,
                inner_angle: 1.0,
                outer_angle: 1.0,
                inner_radius: 0.3,
            },
        ));
    }
}
