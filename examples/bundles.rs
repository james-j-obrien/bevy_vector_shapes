// Demonstrates use of the shape bundles directly rather than go through the ShapeCommands or ShapePainter abstractions

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin::default())
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(Msaa::Off)
        .add_startup_system(setup)
        .add_system(update_shapes)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0., 0.0, 16.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));

    commands.spawn(ShapeBundle::rect(
        &ShapeConfig {
            color: Color::MIDNIGHT_BLUE,
            corner_radii: Vec4::splat(0.3),
            ..default()
        },
        Vec2::splat(2.0),
    ));
}

fn update_shapes(time: Res<Time>, mut shapes: Query<&mut Transform, With<Shape>>) {
    shapes.for_each_mut(|mut tf| {
        tf.rotate_local_z(time.delta_seconds());
    })
}
