// Demonstrates using ShapeCommands to spawn entity backed shapes

use std::f32::consts::PI;

use bevy::{color::palettes::css::*, prelude::*};
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin::default())
        .insert_resource(ClearColor(DIM_GRAY.into()))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_circle)
        .run();
}

fn setup(mut commands: Commands, mut shapes: ShapeCommands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 0.0, 16.).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off,
    ));

    // The ShapeCommands API is identical to the ShapePainter API so can be used almost interchangeably
    shapes.circle(1.0).with_children(|parent| {
        for _ in 0..4 {
            parent.rotate_z(PI / 2.0);
            parent.line(Vec3::ZERO, Vec3::Y * 2.0);
        }
    });
}

fn rotate_circle(time: Res<Time>, mut circle: Query<&mut Transform, With<DiscComponent>>) {
    circle
        .iter_mut()
        .for_each(|mut tf| tf.rotation *= Quat::from_rotation_z(time.delta_secs()))
}
