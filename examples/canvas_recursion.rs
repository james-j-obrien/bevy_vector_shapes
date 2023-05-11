// Demonstrates drawing a canvas image inside itself

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin::default())
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_startup_system(setup)
        .add_system(draw_shapes)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let config = CanvasConfig::new(1024, 1024);
    commands.spawn_canvas(images.as_mut(), config);

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn draw_shapes(time: Res<Time>, mut painter: ShapePainter, canvas: Query<(Entity, &Canvas)>) {
    let (canvas_e, canvas) = canvas.single();
    painter.image(canvas.image.clone(), Vec2::splat(12.));

    painter.set_canvas(canvas_e);
    painter.hollow = true;
    painter.thickness = 16.0;
    painter.color = Color::SEA_GREEN;
    painter.rect(Vec2::splat(1024.0));

    painter.rotate_z(time.elapsed_seconds().sin());
    painter.image(canvas.image.clone(), Vec2::splat(980.0));

    painter.reset();
}
