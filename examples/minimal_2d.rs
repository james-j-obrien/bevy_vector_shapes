// Demonstrates the minimal setup required to draw shapes with a 2D camera

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

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
    commands.spawn(Camera2d);
}

fn draw(mut painter: ShapePainter) {
    // Draw a circle
    painter.circle(100.0);
}
