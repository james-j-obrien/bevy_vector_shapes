// Demonstrated shapes respecting render layers
// Adapted directly from https://github.com/bevyengine/bevy/blob/main/examples/3d/render_to_texture.rs

use bevy::{color::palettes::css::*, prelude::*};
use bevy_vector_shapes::prelude::*;

mod gallery_3d;
use gallery_3d::gallery;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin::default())
        .insert_resource(ClearColor(DIM_GRAY.into()))
        .add_systems(Startup, setup)
        .add_systems(Update, (draw_shapes, draw_canvas))
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut config = CanvasConfig::new(256, 256);
    config.clear_color = ClearColorConfig::Custom((WHITE * 0.5).into());
    let (_, _entity) = commands.spawn_canvas(images.as_mut(), config);

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off,
    ));
}

fn draw_canvas(time: Res<Time>, mut painter: ShapePainter, canvas: Single<Entity, With<Canvas>>) {
    painter.rotate_z(time.elapsed_secs().sin());
    painter.set_canvas(*canvas);
    painter.set_color(WHITE * 2.0);
    painter.translate(Vec3::NEG_Y * 12.0 * 16.0);
    painter.thickness = 16.0;

    for _ in 0..12 {
        painter.translate(Vec3::Y * 32.0);
        painter.line(Vec3::NEG_X * 256.0, Vec3::X * 256.0);
    }
    painter.reset();
}

fn draw_shapes(time: Res<Time>, mut painter: ShapePainter, canvas: Single<&Canvas>) {
    painter.texture = Some(canvas.image.clone());
    painter.translate(Vec3::NEG_Y * 2.0);

    gallery(painter, time.elapsed_secs(), 0..10);
}
