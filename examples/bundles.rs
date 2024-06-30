// Demonstrates use of the shape bundles directly rather than go through the ShapeCommands or ShapePainter abstractions

use bevy::{color::palettes::css::*, prelude::*};
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin::default())
        .insert_resource(ClearColor(DIM_GRAY.into()))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, update_shapes)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0., 0.0, 16.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));

    // Note: [`ShapeBundle`] does not include `RenderLayers` by default so the associated field
    // on [`ShapeConfig`] will be ignored, add the component manually or use [`ShapeCommands::rect`]
    // instead which will handle adding the `RenderLayers` component
    commands.spawn(
        ShapeBundle::rect(
            &ShapeConfig {
                color: MIDNIGHT_BLUE.into(),
                corner_radii: Vec4::splat(0.3),
                ..ShapeConfig::default_3d()
            },
            Vec2::splat(2.0),
        )
        .insert_3d(),
    );
}

fn update_shapes(time: Res<Time>, mut shapes: Query<&mut Transform, With<ShapeMaterial>>) {
    shapes.iter_mut().for_each(|mut tf| {
        tf.rotate_local_z(time.delta_seconds());
    })
}
