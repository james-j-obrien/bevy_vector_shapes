// Demonstrates overriding the origin of a shape so that it is rendered in the correct order.

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the 3D shape plugin
        .add_plugins(ShapePlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, draw)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.5, 0.3, 2.0)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn draw(mut painter: ShapePainter) {
    // Render the background
    painter.color = Color::BLACK.with_alpha(0.9);
    painter.corner_radii = Vec4::splat(0.1);
    painter.rect(Vec2::new(2.0, 1.0));

    // Set the circle color
    painter.color = Color::WHITE;

    // Set the origin of the circles in front of the background
    // Without this, the left circle is blended in the wrong order
    painter.origin = Some(Vec3::Z * 0.01);

    // Render the left circle
    painter.set_translation(Vec3::X * -0.5);
    painter.circle(0.2);

    // Render the right circle
    painter.set_translation(Vec3::X * 0.5);
    painter.circle(0.2);
}
