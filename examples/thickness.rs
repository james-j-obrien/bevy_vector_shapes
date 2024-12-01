// Demonstrates the various thickness types by drawing lines of each type and zoooming in and out

use bevy::{color::palettes::css::*, prelude::*};
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin::default())
        .insert_resource(ClearColor(DIM_GRAY.into()))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_gallery)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Msaa::Off));
}

fn draw_gallery(
    time: Res<Time>,
    mut painter: ShapePainter,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    painter.reset();
    cameras.iter_mut().for_each(|mut tf| {
        *tf = Transform::from_xyz(0., 0., 20. + 10.0 * time.elapsed_secs().sin())
            .looking_at(Vec3::ZERO, Vec3::Y);
    });

    let between_lines = 1.0;
    let between_sets = 3.0;
    let line_vec = Vec3::new(0.4, 2.0, 0.0);

    painter.set_color(MIDNIGHT_BLUE);
    painter.translate(Vec3::NEG_X * (between_lines * 3.0 + between_sets));
    painter.thickness_type = ThicknessType::Pixels;

    painter.thickness = 1.0;
    painter.line(-line_vec, line_vec);

    painter.thickness = 5.0;
    painter.translate(Vec3::X * between_lines);
    painter.line(-line_vec, line_vec);

    painter.thickness = 10.0;
    painter.translate(Vec3::X * between_lines);
    painter.line(-line_vec, line_vec);

    painter.set_color(CRIMSON);
    painter.translate(Vec3::X * between_sets);
    painter.thickness_type = ThicknessType::World;

    painter.thickness = 0.1;
    painter.line(-line_vec, line_vec);

    painter.thickness = 0.2;
    painter.translate(Vec3::X * between_lines);
    painter.line(-line_vec, line_vec);

    painter.thickness = 0.5;
    painter.translate(Vec3::X * between_lines);
    painter.line(-line_vec, line_vec);

    painter.set_color(SEA_GREEN);
    painter.translate(Vec3::X * between_sets);
    painter.thickness_type = ThicknessType::Screen;

    painter.thickness = 1.0;
    painter.line(-line_vec, line_vec);

    painter.thickness = 2.0;
    painter.translate(Vec3::X * between_lines);
    painter.line(-line_vec, line_vec);

    painter.thickness = 5.0;
    painter.translate(Vec3::X * between_lines);
    painter.line(-line_vec, line_vec);
}
