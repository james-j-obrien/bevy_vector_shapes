// Demonstrated shapes respecting render layers
// Adapted directly from https://github.com/bevyengine/bevy/blob/main/examples/3d/render_to_texture.rs

use std::f32::consts::PI;

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Shape2dPlugin::default())
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_startup_system(setup)
        .add_system(draw_shapes)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut config = CanvasConfig::new(512, 512);
    config.clear_color = ClearColorConfig::Custom(Color::WHITE);
    let (handle, _) = commands.spawn_canvas(images.as_mut(), config);

    let quad = meshes.add(Mesh::from(shape::Quad::new(Vec2::ONE)));

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(handle),
        unlit: true,
        ..default()
    });

    // Main pass cube, with material containing the rendered first pass texture.
    commands.spawn((PbrBundle {
        mesh: quad,
        material: material_handle,
        ..default()
    },));

    // The main pass camera.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn draw_shapes(time: Res<Time>, mut painter: ShapePainter, canvas: Query<Entity, With<Canvas>>) {
    let canvas = canvas.single();
    painter.reset();
    painter.canvas = Some(canvas);

    painter.transform.scale = Vec3::ONE * 128.0;
    painter.hollow = true;

    let meter_fill = (time.elapsed_seconds().sin() + 1.0) / 2.0;
    let meter_size = PI * 1.5;

    let start_angle = -meter_size / 2.0;
    let end_angle = -meter_size / 2.0 + meter_fill * meter_size;

    painter.cap = Cap::Round;
    painter.thickness = 0.4;
    painter.color = Color::CRIMSON * (1.0 / (0.5 + meter_fill));
    painter.arc(1.3, start_angle, end_angle);

    painter.cap = Cap::None;
    painter.thickness = 0.2;
    painter.color = Color::DARK_GRAY;
    painter.arc(1.6, start_angle, -start_angle);
    painter.arc(0.8, start_angle, -start_angle);

    let offset = Quat::from_rotation_z(start_angle) * Vec3::Y * 1.1;
    painter.translate(offset);
    painter.arc(0.5, start_angle + PI * 1.5, start_angle + 2.5 * PI);
    painter.translate(-offset);

    painter.translate(Quat::from_rotation_z(-start_angle) * Vec3::Y * 1.1);
    painter.arc(0.5, start_angle + PI, start_angle + 2.0 * PI);
}
