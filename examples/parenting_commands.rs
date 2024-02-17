// Demonstrates spawning shapes as children of non-shape entities and spawning non-shape entities as children of shapes.
//
// Alternatively see the `bundles` example to spawn shapes as bundles and bypass ShapeCommands entirely.
use std::f32::consts::PI;

use bevy::math::primitives::Cuboid;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin::default())
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_targets)
        .run();
}

#[derive(Component)]
struct Target;

fn setup(mut commands: Commands, mut shapes: ShapeCommands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0.0, 16.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // When spawning shapes as children of non-shape entities you can use `with_shape_children`
    // This requires passing in a ShapeConfig, you can construct one yourself or
    // take an existing one from a ShapePainter or ShapeCommands with .config()
    commands
        .spawn((Target, SpatialBundle::default()))
        .with_shape_children(shapes.config(), |child_builder| {
            for _ in 0..4 {
                child_builder.rotate_z(PI / 2.0);
                child_builder.line(Vec3::Y, Vec3::Y * 2.0);
            }
        });

    let cube_handle = meshes.add(Mesh::from(Cuboid::new(0.2, 0.2, 0.2)));

    // When spawning non-shapes as children of shapes you can use `with_children` like normal
    shapes
        .circle(0.2)
        .insert(Target)
        .with_children(|child_builder| {
            for i in 0..4 {
                let transform = Transform::from_translation(
                    Quat::from_rotation_z(PI / 2.0 * i as f32 + PI / 4.0)
                        * Vec3::new(0.0, 1.0, 0.0),
                );
                child_builder.spawn(PbrBundle {
                    mesh: cube_handle.clone(),
                    transform,
                    ..default()
                });
            }
        });
}

fn rotate_targets(time: Res<Time>, mut target: Query<&mut Transform, With<Target>>) {
    target
        .iter_mut()
        .for_each(|mut tf| tf.rotation *= Quat::from_rotation_z(time.delta_seconds()))
}
