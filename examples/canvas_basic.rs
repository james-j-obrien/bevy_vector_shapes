// Demonstrates use of the canvas API by drawing the gallery from gallery_3d onto a canvas and drawing the canvas to the screen

use bevy::{color::palettes::css::*, prelude::*};
use bevy_vector_shapes::prelude::*;

mod gallery_3d;
use gallery_3d::gallery;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin::default())
        .insert_resource(ClearColor(DIM_GRAY.into()))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_shapes)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let config = CanvasConfig::new(1024, 1024);
    commands.spawn_canvas(images.as_mut(), config);

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        msaa: Msaa::Off,
        ..default()
    });
}

fn draw_shapes(time: Res<Time>, mut painter: ShapePainter, canvas: Query<(Entity, &Canvas)>) {
    let (canvas_e, canvas) = canvas.single();
    painter.image(canvas.image.clone(), Vec2::splat(20.));

    painter.set_canvas(canvas_e);
    painter.set_scale(Vec3::ONE * 48.0);

    gallery(painter, time.elapsed_secs(), 0..15);
}
