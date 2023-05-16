// Demonstrates spawning child shapes using with_children on ShapePainter

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin::default())
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(Msaa::Off)
        .add_startup_system(setup)
        .add_system(draw_gallery)
        .run();
}

#[derive(Component)]
struct Tree;

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Immediate mode shapes don't need to be parented to an entity but we do so here to demonstrate how
    commands.spawn((
        Tree,
        SpatialBundle::from_transform(Transform::from_xyz(0.0, -5.0, 0.0)),
    ));
}

fn draw_tree(time: f32, painter: &mut ShapePainter, depth: u32) {
    if depth == 0 {
        return;
    }

    let line_a = Vec3::Y + Vec3::X * 0.5;
    painter.rotate_z(time.sin() * 0.2);
    painter
        .line(Vec3::ZERO, line_a)
        .with_children(|child_painter| {
            child_painter.translate(line_a);

            draw_tree(time, child_painter, depth - 1);
        });

    let line_b = Vec3::Y + Vec3::NEG_X * 0.5;
    painter.rotate_z(-time.sin() * 0.4);
    painter
        .line(Vec3::ZERO, line_b)
        .with_children(|child_painter| {
            child_painter.translate(line_b);

            draw_tree(time, child_painter, depth - 1);
        });
}

fn draw_gallery(
    time: Res<Time>,
    mut painter: ShapePainter,
    mut tree: Query<&mut Transform, With<Tree>>,
) {
    let mut tree = tree.single_mut();
    tree.rotation = Quat::from_rotation_z(time.elapsed_seconds().sin() / 4.0);

    // Position our painter relative to our tree entity
    painter.transform = *tree;
    painter.color = Color::SEA_GREEN;
    painter
        .line(Vec3::ZERO, Vec3::Y)
        .with_children(|child_painter| {
            child_painter.translate(Vec3::Y);
            draw_tree(time.elapsed_seconds(), child_painter, 10);
        });
    painter.reset();
}
