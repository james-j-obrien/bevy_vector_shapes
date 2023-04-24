// Demonstrates rendering the same gallery as gallery_3d but with a 2d camera

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_vector_shapes::prelude::*;

mod gallery_3d;
use gallery_3d::gallery;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Shape2dPlugin::default())
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(Msaa::Off)
        .add_startup_system(setup)
        .add_system(draw_gallery)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::Z),
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 5.2 * 4.5,
                min_height: 3.2 * 4.5,
            },
            ..default()
        },
        ..default()
    });
}

fn draw_gallery(time: Res<Time>, painter: ShapePainter) {
    gallery(painter, time.elapsed_seconds(), 0..15);
}
