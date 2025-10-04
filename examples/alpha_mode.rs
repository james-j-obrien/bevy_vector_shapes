// Demonstrates the supported alpha modes that shapes respect

use std::f32::consts::TAU;

use bevy::camera::ScalingMode;
use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

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
    commands.spawn((
        Camera2d,
        Transform::from_translation(Vec3::Z),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 5.2 * 4.5,
                min_height: 3.2 * 4.5,
            },
            ..OrthographicProjection::default_2d()
        }),
        Msaa::Off,
    ));
}

fn draw_circles(painter: &mut ShapePainter, radius: f32) {
    painter.translate(-(Vec3::X + Vec3::NEG_Y) * f32::sqrt(radius) * 0.5);
    painter.color = Color::srgba(1.0, 0.0, 0.0, 0.5);
    painter.circle(radius);

    painter.rotate_z(-TAU / 3.0);
    painter.translate(Vec3::Y * radius * 1.2 + Vec3::Z * 0.0001);
    painter.color = Color::srgba(0.0, 1.0, 0.0, 0.5);
    painter.circle(radius);

    painter.rotate_z(-TAU / 3.0);
    painter.translate(Vec3::Y * radius * 1.2 + Vec3::Z * 0.0001);
    painter.color = Color::srgba(0.0, 0.0, 1.0, 0.5);
    painter.circle(radius);
}

fn draw_gallery(mut painter: ShapePainter) {
    let radius = 2.0;

    painter.reset();
    painter.translate(Vec3::X * radius * -4.0);
    painter.alpha_mode = ShapeAlphaMode::Add;
    draw_circles(&mut painter, radius);

    painter.reset();
    painter.alpha_mode = ShapeAlphaMode::Multiply;
    draw_circles(&mut painter, radius);

    painter.reset();
    painter.translate(Vec3::X * radius * 4.0);
    painter.alpha_mode = ShapeAlphaMode::Blend;
    draw_circles(&mut painter, radius);
}
