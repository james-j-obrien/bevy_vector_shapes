// Demonstrates the minimal setup required to draw shapes with a 2D camera

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the shape plugin
        .add_plugin(Shape2dPlugin::default())
        .add_startup_system(setup)
        .add_system(draw)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn(Camera2dBundle::default());
}

fn draw(mut painter: ShapePainter) {
    painter.clear();
    // Draw a circle
    painter.circle(50.0);

    painter.color = Color::RED;
    painter.translate(Vec3::X * 100.0);
    painter.circle(50.0);
}
