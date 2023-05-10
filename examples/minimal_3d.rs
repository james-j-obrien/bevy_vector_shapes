// Demonstrates the minimal setup required to draw shapes with a 3D camera

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the 3D shape plugin
        .add_plugin(ShapePlugin::default())
        .add_startup_system(setup)
        .add_system(draw)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn(Camera3dBundle::default());
}

fn draw(mut painter: ShapePainter) {
    // Move the painter so it's not inside the camera
    painter.set_translation(Vec3::NEG_Z);
    // Draw a circle
    painter.circle(0.1);
}
