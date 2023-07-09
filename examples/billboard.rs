// Demonstrates building across each type of shape
// NOTE: Lines billboard across their axis instead of directly to the camera

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

mod gallery_3d;
use gallery_3d::gallery;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin {
            base_config: ShapeConfig {
                alignment: Alignment::Billboard,
                ..ShapeConfig::default_3d()
            },
            ..default()
        })
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_gallery)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle::default());
}

fn draw_gallery(
    time: Res<Time>,
    painter: ShapePainter,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    cameras.for_each_mut(|mut tf| {
        *tf = Transform::from_translation(
            Quat::from_rotation_y(time.elapsed_seconds()) * Vec3::new(0., 2.5, 16.),
        )
        .looking_at(Vec3::Y * 2.5, Vec3::Y);
    });
    gallery(painter, time.elapsed_seconds(), 0..10);
}
