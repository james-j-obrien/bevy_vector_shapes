// Demonstrates rendering the same gallery as gallery_3d but with a 2d camera

use bevy::{color::palettes::css::*, prelude::*};
use bevy_vector_shapes::prelude::*;

mod gallery_3d;
use gallery_3d::gallery;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Shape2dPlugin::default())
        .insert_resource(ClearColor(DIM_GRAY.into()))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_gallery)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off));
}

fn draw_gallery(time: Res<Time>, mut painter: ShapePainter) {
    painter.scale(Vec3::ONE * 34.0);
    gallery(painter, time.elapsed_secs(), 0..15);
}
