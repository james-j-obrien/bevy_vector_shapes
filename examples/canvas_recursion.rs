// Demonstrates drawing a canvas image inside itself

use bevy::{color::palettes::css::*, prelude::*};
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin::default())
        .insert_resource(ClearColor(DIM_GRAY.into()))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_shapes)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let config = CanvasConfig::new(1024, 1024);
    commands.spawn_canvas(images.as_mut(), config);

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off,
    ));
}

fn draw_shapes(time: Res<Time>, mut painter: ShapePainter, canvas: Query<(Entity, &Canvas)>) {
    let (canvas_e, canvas) = canvas.single();
    painter.image(canvas.image.clone(), Vec2::splat(12.));

    painter.set_canvas(canvas_e);
    painter.hollow = true;
    painter.thickness = 16.0;
    painter.set_color(SEA_GREEN + Srgba::WHITE * 0.25);
    painter.rect(Vec2::splat(1024.0));

    painter.rotate_z(time.elapsed_secs().sin());
    painter.image(canvas.image.clone(), Vec2::splat(980.0));

    painter.reset();
}
