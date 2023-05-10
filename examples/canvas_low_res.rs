// Demonstrated shapes respecting render layers
// Adapted directly from https://github.com/bevyengine/bevy/blob/main/examples/3d/render_to_texture.rs

use bevy::{prelude::*, render::texture::ImageSampler};
use bevy_vector_shapes::prelude::*;

mod gallery_3d;
use gallery_3d::gallery;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Shape2dPlugin::new(ShapeConfig {
            disable_laa: true,
            ..ShapeConfig::default_2d()
        }))
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(Msaa::Off)
        .add_startup_system(setup)
        .add_system(draw_shapes)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut config = CanvasConfig::new(256, 256);
    config.sampler = ImageSampler::nearest();
    let (handle, _) = commands.spawn_canvas(images.as_mut(), config);

    let quad = meshes.add(Mesh::from(shape::Quad::new(Vec2::ONE * 2.0)));

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(handle),
        unlit: true,
        ..default()
    });

    // Main pass cube, with material containing the rendered first pass texture.
    commands.spawn((PbrBundle {
        mesh: quad,
        material: material_handle,
        ..default()
    },));

    // The main pass camera.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn draw_shapes(time: Res<Time>, mut painter: ShapePainter, canvas: Query<Entity, With<Canvas>>) {
    let canvas = canvas.single();
    painter.canvas = Some(canvas);
    painter.set_scale(Vec3::ONE * 12.0);

    gallery(painter, time.elapsed_seconds(), 0..15);
}
