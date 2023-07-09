// Demonstrated use of a canvas to render low resolutions shapes on a small canvas

use bevy::{prelude::*, render::texture::ImageSampler};
use bevy_vector_shapes::prelude::*;

mod gallery_3d;
use gallery_3d::gallery;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin::new(ShapeConfig {
            disable_laa: true,
            ..ShapeConfig::default_3d()
        }))
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_shapes)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut config = CanvasConfig::new(256, 256);
    config.sampler = ImageSampler::nearest();
    commands.spawn_canvas(images.as_mut(), config);

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn draw_shapes(time: Res<Time>, mut painter: ShapePainter, canvas: Query<(Entity, &Canvas)>) {
    let (canvas_e, canvas) = canvas.single();
    painter.image(canvas.image.clone(), Vec2::splat(20.));

    painter.set_canvas(canvas_e);
    painter.set_scale(Vec3::ONE * 12.0);

    gallery(painter, time.elapsed_seconds(), 0..15);
}
