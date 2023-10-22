// Demonstrates triangles

use bevy::math::{vec2, vec3};
use bevy::prelude::shape::Icosphere;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use core::f32::consts::PI;

#[derive(Resource)]
struct Sphere(pub Mesh);

fn main() {
    let mesh = Mesh::try_from(Icosphere {
        radius: 100.0,
        subdivisions: 4,
    })
    .expect("Failed to generate mesh.");

    App::new()
        .add_plugins(DefaultPlugins)
        // Add the shape plugin
        .add_plugins(Shape2dPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, draw)
        .insert_resource(Sphere(mesh))
        .insert_resource(Msaa::Off)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn(Camera2dBundle::default());
}

fn draw(mut painter: ShapePainter, sphere: Res<Sphere>) {
    let mesh = &sphere.0;
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
    let mut iter = positions.as_float3().unwrap().iter();
    painter.hollow = false;
    painter.thickness = 10.0;
    painter.color = Color::ORANGE;

    // painter.roundness = 20.0;
    // while let (Some(a), Some(b), Some(c)) = (iter.next(), iter.next(), iter.next()) {
    //     let a = Vec3::from(*a);
    //     let b = Vec3::from(*b);
    //     let c = Vec3::from(*c);
    //     painter.triangle(a, b, c);
    // }

    // Regular 3-gon for comparison
    // painter.color = Color::RED;
    // painter.ngon(3.0, 100.0);

    // Triangle that happens to be regular as well
    // painter.color = Color::ORANGE;
    // painter.triangle(
    //     100.0 * Vec2::from_angle(PI * -1. / 6.).extend(0.0), // + Vec2::splat(100.0),
    //     100.0 * Vec2::from_angle(PI * 3. / 6.).extend(0.0),  // + Vec2::splat(100.0),
    //     100.0 * Vec2::from_angle(PI * 7. / 6.).extend(0.0),  // + Vec2::splat(100.0),
    // );

    painter.color = Color::BLUE;
    painter.triangle(vec2(-200.0, 0.0), vec2(-200.0, 100.0), vec2(-300.0, 100.0));
    painter.color = Color::ALICE_BLUE;
    painter.triangle(vec2(-200.0, 0.0), vec2(-200.0, 100.0), vec2(-400.0, 0.0));

    // // non-regular triangles
    // painter.color = Color::YELLOW;
    // // painter.hollow = false;
    // painter.thickness = 10.0;
    // // painter.roundness = 0.0;

    // let quad_points = [
    //     vec2(200., -100.),
    //     vec2(400., -110.),
    //     vec2(390., 70.),
    //     vec2(230., 105.),
    // ];

    // let delta = vec2(-10., 10.);

    // painter.triangle(quad_points[0], quad_points[1], quad_points[2]);
    // painter.triangle(
    //     quad_points[2] + delta,
    //     quad_points[3] + delta,
    //     quad_points[0] + delta,
    // );
}
