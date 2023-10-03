// Demonstrates triangles

use core::f32::consts::PI;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use bevy::math::vec2;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the shape plugin
        .add_plugins(Shape2dPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, draw)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn(Camera2dBundle::default());
}

fn draw(mut painter: ShapePainter) {
    painter.hollow = false;
    painter.thickness = 10.0;
    painter.roundness = 20.0;

    // Regular 3-gon for comparison
    painter.color = Color::RED;
    painter.ngon(3.0, 100.0);

    // Triangle that happens to be regular as well
    painter.color = Color::ORANGE;
    painter.hollow = true;
    painter.triangle(
        100.0 * Vec2::from_angle(PI * -1. / 6.),
        100.0 * Vec2::from_angle(PI * 3. / 6.),
        100.0 * Vec2::from_angle(PI * 7. / 6.),
    );

    // non-regular triangles
    painter.color = Color::YELLOW;
    painter.hollow = true;
    painter.thickness = 10.0;
    painter.roundness = 0.0;

    let quad_points = [
        vec2(200., -100.),
        vec2(400., -110.),
        vec2(390., 70.),
        vec2(230., 105.),
    ];

    let delta = vec2(-10., 10.);

    painter.triangle(quad_points[0], quad_points[1], quad_points[2]);
    painter.triangle(quad_points[2] + delta, quad_points[3] + delta, quad_points[0] + delta);
}
