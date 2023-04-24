// Demonstrates using a ShapePainter with immediate mode disabled
// Shapes are spawned in setup and are retained indefinitely

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

mod gallery_3d;
use gallery_3d::gallery;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin::retained())
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(Msaa::Off)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, painter: ShapePainter) {
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0., 0.0, 16.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));

    gallery(painter, 0.0, 0..15);
}
