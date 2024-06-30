// Demonstrates shapes respecting render layers
// Adapted directly from https://github.com/bevyengine/bevy/blob/main/examples/3d/render_to_texture.rs

use std::f32::consts::PI;

use bevy::{
    color::palettes::css::*,
    prelude::*,
    render::{camera::RenderTarget, texture::ImageSampler, view::RenderLayers},
};
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin::default())
        .insert_resource(ClearColor(DIM_GRAY.into()))
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_cube, draw_shapes))
        .run();
}

// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let image_handle =
        Canvas::create_image(images.as_mut(), 512, 512, ImageSampler::Default, false);

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = RenderLayers::layer(1);

    // Light
    // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::WHITE),
                // render before the "main pass" camera
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        first_pass_layer,
    ));

    let cube_size = 4.0;
    let cube_handle = meshes.add(Mesh::from(Cuboid::new(cube_size, cube_size, cube_size)));

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // Main pass cube, with material containing the rendered first pass texture.
    commands.spawn((
        PbrBundle {
            mesh: cube_handle,
            material: material_handle,
            transform: Transform::from_xyz(0.0, 0.0, 1.5)
                .with_rotation(Quat::from_rotation_x(-PI / 5.0)),
            ..default()
        },
        MainPassCube,
    ));

    // The main pass camera.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn draw_shapes(time: Res<Time>, mut painter: ShapePainter) {
    painter.reset();
    painter.render_layers = Some(RenderLayers::layer(1));
    painter.hollow = true;
    painter.transform.scale = Vec3::ONE * 3.0;

    let meter_fill = (time.elapsed_seconds().sin() + 1.0) / 2.0;
    let meter_size = PI * 1.5;

    let start_angle = -meter_size / 2.0;
    let end_angle = -meter_size / 2.0 + meter_fill * meter_size;

    painter.cap = Cap::Round;
    painter.thickness = 0.4;
    painter.set_color(CRIMSON * (1.0 / (0.5 + meter_fill)));
    painter.arc(1.3, start_angle, end_angle);

    painter.cap = Cap::None;
    painter.thickness = 0.2;
    painter.set_color(DARK_GRAY);
    painter.arc(1.6, start_angle, -start_angle);
    painter.arc(0.8, start_angle, -start_angle);

    let offset = Quat::from_rotation_z(start_angle) * Vec3::Y * 1.1;
    painter.translate(offset);
    painter.arc(0.5, start_angle + PI * 1.5, start_angle + 2.5 * PI);
    painter.translate(-offset);

    painter.translate(Quat::from_rotation_z(-start_angle) * Vec3::Y * 1.1);
    painter.arc(0.5, start_angle + PI, start_angle + 2.0 * PI);
}

fn rotate_cube(time: Res<Time>, mut query: Query<&mut Transform, With<MainPassCube>>) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_rotation_x(time.elapsed_seconds())
            * Quat::from_rotation_y(time.elapsed_seconds() / 2.0);
    }
}
