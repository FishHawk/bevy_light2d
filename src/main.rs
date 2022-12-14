use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use city::render::Light2dOverlay;
use city::{Light2dPlugin, PointLight2d, Shadow2d};

fn main() {
    const BACKGROUND_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Light2dPlugin)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };
    image.resize(size);

    let image_handle = images.add(image);

    let parent = commands.spawn(Camera2dBundle::default()).id();
    let child = commands
        .spawn(Light2dOverlay {
            image: image_handle.clone(),
            size: UVec2::new(size.width, size.height),
        })
        .id();
    commands.entity(parent).push_children(&[child]);

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
                    scale: Vec3::new(500.0, 500.0, 0.0),
                    ..default()
                },
                ..default()
            },
            PointLight2d {
                color,
                falloff_intensity: 0.5,
                inner_angle: 1.0,
                outer_angle: 1.0,
                inner_radius: 0.3,
            },
        ));
    }

    commands.spawn((
        SpatialBundle {
            transform: Transform {
                translation: Vec3::new(300.0, 300.0, 1.0),
                scale: Vec3::new(100.0, 100.0, 0.0),
                ..default()
            },
            ..default()
        },
        Shadow2d {
            closed: true,
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
            ],
        },
    ));
}

// use std::f32::consts::PI;

// use bevy::{
//     core_pipeline::clear_color::ClearColorConfig,
//     prelude::*,
//     render::{
//         camera::RenderTarget,
//         render_resource::{
//             Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
//         },
//         view::RenderLayers,
//     },
// };

// fn main() {
//     App::new()
//         .add_plugins(DefaultPlugins)
//         .add_startup_system(setup)
//         .add_system(cube_rotator_system)
//         .add_system(rotator_system)
//         .run();
// }

// // Marks the first pass cube (rendered to a texture.)
// #[derive(Component)]
// struct FirstPassCube;

// // Marks the main pass cube, to which the texture is applied.
// #[derive(Component)]
// struct MainPassCube;

// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut images: ResMut<Assets<Image>>,
// ) {
//     let size = Extent3d {
//         width: 512,
//         height: 512,
//         ..default()
//     };

//     // This is the texture that will be rendered to.
//     let mut image = Image {
//         texture_descriptor: TextureDescriptor {
//             label: None,
//             size,
//             dimension: TextureDimension::D2,
//             format: TextureFormat::Bgra8UnormSrgb,
//             mip_level_count: 1,
//             sample_count: 1,
//             usage: TextureUsages::TEXTURE_BINDING
//                 | TextureUsages::COPY_DST
//                 | TextureUsages::RENDER_ATTACHMENT,
//         },
//         ..default()
//     };

//     // fill image.data with zeroes
//     image.resize(size);

//     let image_handle = images.add(image);

//     let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 4.0 }));
//     let cube_material_handle = materials.add(StandardMaterial {
//         base_color: Color::rgb(0.8, 0.7, 0.6),
//         reflectance: 0.02,
//         unlit: false,
//         ..default()
//     });

//     // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
//     let first_pass_layer = RenderLayers::layer(1);

//     // The cube that will be rendered to the texture.
//     commands.spawn((
//         PbrBundle {
//             mesh: cube_handle,
//             material: cube_material_handle,
//             transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
//             ..default()
//         },
//         FirstPassCube,
//         first_pass_layer,
//     ));

//     // Light
//     // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
//     commands.spawn(PointLightBundle {
//         transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
//         ..default()
//     });

//     commands.spawn((
//         Camera3dBundle {
//             camera_3d: Camera3d {
//                 clear_color: ClearColorConfig::Custom(Color::WHITE),
//                 ..default()
//             },
//                 camera: Camera {
//                     // render before the "main pass" camera
//                     priority: -1,
//                     target: RenderTarget::Image(image_handle.clone()),
//                     ..default()
//                 },
//             transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
//                 .looking_at(Vec3::ZERO, Vec3::Y),
//             ..default()
//         },
//         first_pass_layer,
//     ));

//     let cube_size = 4.0;
//     let cube_handle = meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size)));

//     // This material has the texture that has been rendered.
//     let material_handle = materials.add(StandardMaterial {
//         base_color_texture: Some(image_handle),
//         reflectance: 0.02,
//         unlit: false,
//         ..default()
//     });

//     // Main pass cube, with material containing the rendered first pass texture.
//     commands.spawn((
//         PbrBundle {
//             mesh: cube_handle,
//             material: material_handle,
//             transform: Transform::from_xyz(0.0, 0.0, 1.5)
//                 .with_rotation(Quat::from_rotation_x(-PI / 5.0)),
//             ..default()
//         },
//         MainPassCube,
//     ));

//     // The main pass camera.
//     commands.spawn(Camera3dBundle {
//         transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
//         ..default()
//     });
// }

// /// Rotates the inner cube (first pass)
// fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<FirstPassCube>>) {
//     for mut transform in &mut query {
//         transform.rotate_x(1.5 * time.delta_seconds());
//         transform.rotate_z(1.3 * time.delta_seconds());
//     }
// }

// /// Rotates the outer cube (main pass)
// fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<MainPassCube>>) {
//     for mut transform in &mut query {
//         transform.rotate_x(1.0 * time.delta_seconds());
//         transform.rotate_y(0.7 * time.delta_seconds());
//     }
// }
