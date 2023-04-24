// A stress test drawing a large number of shapes

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

const SHAPES_PER_AXIS: u32 = 100;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin::default())
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_startup_system(setup)
        .add_system(draw_spheres)
        .run();
}

fn setup(mut commands: Commands) {
    let shapes = SHAPES_PER_AXIS as f32;
    let center = Vec3::new(shapes, 0.0, shapes);
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 20.0, 0.0).looking_at(center, Vec3::Y),
        ..default()
    });
}

fn draw_health_bar(painter: &mut ShapePainter, hp: f32) {
    painter.translate(Vec3::Y * 0.7);
    painter.corner_radii = Vec4::splat(0.3);

    painter.color = Color::GREEN * hp + Color::RED * (1. - hp);
    painter.rect(Vec2::new(0.2 + 0.8 * hp, 0.2));

    painter.thickness = 0.02;
    painter.hollow = true;
    painter.color = Color::WHITE;
    painter.rect(Vec2::new(1.06, 0.26));
}

fn draw_spheres(time: Res<Time>, mut painter: ShapePainter) {
    for x in 0..SHAPES_PER_AXIS {
        for y in 0..SHAPES_PER_AXIS {
            let (x, y) = (x as f32, y as f32);
            let offset = time.elapsed_seconds() + x + 100. * y;
            let position = Vec3::new(x * 2.0, offset.sin(), y * 2.0);

            painter.hollow = false;
            painter.color = Color::DARK_GRAY;
            painter.alignment = Alignment::Billboard;
            painter.transform.translation = position;
            painter.circle(0.5);

            let hp = (offset.sin() + 1.) / 2.0;
            draw_health_bar(&mut painter, hp);
        }
    }
}
