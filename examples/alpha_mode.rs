// Demonstrates the supported alpha modes that shapes respect

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_vector_shapes::prelude::*;

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

fn draw_circles(painter: &mut ShapePainter, radius: f32) {
    painter.translate(-(Vec3::X + Vec3::NEG_Y) * f32::sqrt(radius) * 0.5);
    painter.color = Color::rgba(1.0, 0.0, 0.0, 0.5);
    painter.circle(radius);

    painter.rotate_z(-TAU / 3.0);
    painter.translate(Vec3::Y * radius * 1.2 + Vec3::Z * 0.0001);
    painter.color = Color::rgba(0.0, 1.0, 0.0, 0.5);
    painter.circle(radius);

    painter.rotate_z(-TAU / 3.0);
    painter.translate(Vec3::Y * radius * 1.2 + Vec3::Z * 0.0001);
    painter.color = Color::rgba(0.0, 0.0, 1.0, 0.5);
    painter.circle(radius);
}

fn draw_gallery(mut painter: ShapePainter) {
    let radius = 2.0;

    painter.clear();
    painter.translate(Vec3::X * radius * -4.0);
    painter.alpha_mode = AlphaMode::Add;
    draw_circles(&mut painter, radius);

    painter.clear();
    painter.alpha_mode = AlphaMode::Multiply;
    draw_circles(&mut painter, radius);

    painter.clear();
    painter.translate(Vec3::X * radius * 4.0);
    painter.alpha_mode = AlphaMode::Blend;
    draw_circles(&mut painter, radius);
}
