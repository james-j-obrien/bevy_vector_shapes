// Demonstrates each shape type with a selection of options and possible use cases

#![allow(dead_code)]
use std::{
    f32::consts::{PI, TAU},
    ops::Range,
};

use bevy::{camera::visibility::RenderLayers, color::palettes::css::*, prelude::*};
use bevy_vector_shapes::prelude::*;

pub trait Pastel {
    fn pastel(&self) -> Srgba;
}

impl Pastel for Srgba {
    fn pastel(&self) -> Srgba {
        let mut out = *self + Srgba::WHITE * 0.25;
        out.set_alpha(1.0);
        out
    }
}

pub fn gallery(mut painter: ShapePainter, seconds: f32, entries: Range<i32>) {
    let seconds = seconds % (2.0 * TAU);
    let start_pos = painter.transform;

    for i in entries {
        let (x, y) = ((i % 5 - 2) as f32, (i / 5 - 1) as f32);
        let diag_vec = Vec3::X + Vec3::Y;
        painter.transform = start_pos;
        painter.translate((Vec3::X * x - Vec3::Y * y) * 4.0);

        match i {
            // Line examples
            0 => {
                painter.thickness = 0.5;
                painter.set_color(MIDNIGHT_BLUE.pastel());
                painter.cap = Cap::None;
                painter.line(Vec3::new(-1.0, 1.0, 0.0), Vec3::new(1.0, 1.0, 0.0));

                painter.set_color(MIDNIGHT_BLUE.pastel() * 1.2);
                painter.cap = Cap::Square;
                painter.line(Vec3::new(-1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

                painter.set_color(MIDNIGHT_BLUE.pastel() * 1.6);
                painter.cap = Cap::Round;
                painter.line(Vec3::new(-1.0, -1.0, 0.0), Vec3::new(1.0, -1.0, 0.0));
            }
            5 => {
                painter.thickness = 0.5;
                painter.set_color(MIDNIGHT_BLUE.pastel() * 1.4);
                painter.cap = Cap::Round;
                let rotation = Quat::from_rotation_z(seconds * 2.5);
                let radius = ((seconds * 3.0).sin() + 0.5) / 2.0 * 1.5;
                painter.line(
                    rotation * Vec3::new(-1.0, 0.0, 0.0) * radius,
                    rotation * Vec3::new(1.0, 0.0, 0.0) * radius,
                );
            }
            10 => {
                painter.thickness = 0.4;
                painter.set_color(MIDNIGHT_BLUE.pastel());
                painter.cap = Cap::Round;
                painter.line((Vec3::X + Vec3::Y) * 0.2, (Vec3::NEG_X + Vec3::NEG_Y) * 0.2);
                painter.line((Vec3::X + Vec3::NEG_Y) * 0.2, (Vec3::NEG_X + Vec3::Y) * 0.2);

                painter.translate(Vec3::Z * 0.001);
                painter.set_color(WHITE);
                painter.thickness = 0.2;
                painter.cap = Cap::Square;
                let radius = 0.5 + ((seconds * 2.0).sin() + 1.0) / 4.0;
                let offset = Quat::from_rotation_z(PI / 4.0);

                for i in 0..4 {
                    let rotation = Quat::from_rotation_z(-seconds + PI / 2.0 * i as f32);
                    painter.line(
                        rotation * Vec3::X * radius,
                        rotation * Vec3::X * radius + rotation * offset * Vec3::X * 0.4,
                    );
                    painter.line(
                        rotation * Vec3::X * radius,
                        rotation * Vec3::X * radius + rotation * offset.inverse() * Vec3::X * 0.4,
                    );
                }
            }
            // Rect examples
            1 => {
                painter.set_color(SEA_GREEN.pastel() * 0.7);
                painter.corner_radii = Vec4::ZERO;
                painter.translate(Vec3::Y);
                painter.rect(Vec2::new(2.0, 0.7));

                painter.set_color(SEA_GREEN.pastel());
                painter.translate(-Vec3::Y);
                painter.corner_radii = Vec4::splat(0.2);
                painter.rect(Vec2::new(2.0, 0.7));

                painter.set_color(SEA_GREEN.pastel() * 1.2);
                painter.corner_radii = Vec4::new(0.0, 0.2, 0.35, 0.0);
                painter.translate(-Vec3::Y);
                painter.rect(Vec2::new(2.0, 0.7));
            }
            6 => {
                let bar_fill = 0.1 + 0.9 * (seconds.sin() + 1.0) / 2.0;
                let square_fill = ((seconds * 3.).sin() + 1.0) / 2.0;

                painter.hollow = true;
                painter.set_color(SEA_GREEN.pastel());
                painter.thickness = 0.2 + 1.3 * square_fill;
                painter.corner_radii = Vec4::splat(1.0);
                painter.rotate_z(bar_fill * TAU);
                painter.rect(Vec2::splat(2.5));
            }
            11 => {
                let bar_fill = 0.2 + 0.8 * (seconds.sin() + 1.0) / 2.0;
                let bar_border = 0.2;
                let bar_size = Vec2::new(1.0, 3.0);

                painter.hollow = true;
                painter.thickness = bar_border;
                painter.set_color(WHITE);
                painter.corner_radii = Vec4::new(0.5, 0.5, 0.5, 0.8);
                painter.rect(bar_size + bar_border * 3.0);

                painter.set_color(SEA_GREEN.pastel() * (1.0 / (0.8 + bar_fill * 0.6)));
                painter.hollow = false;
                painter.corner_radii -= 0.5 * Vec4::splat(bar_border * 3.0);
                painter.translate(-Vec3::Y * (1.0 - bar_fill) * bar_size.y / 2.0);
                painter.rect(Vec2::new(bar_size.x, bar_fill * bar_size.y));
            }
            // Circle examples
            2 => {
                painter.hollow = false;
                painter.set_color(ORANGE.pastel() * 0.7);
                painter.translate(diag_vec * 0.8);
                painter.circle(0.8);

                painter.hollow = true;
                painter.thickness = 0.4;
                painter.set_color(ORANGE.pastel() * 1.4);
                painter.translate(-diag_vec * 1.6);
                painter.circle(0.8);
            }
            7 => {
                let circle_fill = ((seconds * 2.).sin() + 1.0) / 2.0;

                painter.hollow = true;
                painter.set_color(ORANGE.pastel());
                painter.thickness = 0.5;
                painter.circle(1.5 * circle_fill);
            }
            12 => {
                fn draw_bubble(
                    painter: &mut ShapePainter,
                    seconds: f32,
                    position: Vec3,
                    scale: f32,
                ) {
                    let seconds = seconds % PI;
                    let circle_size = (seconds).powf(2.8) / 40. * scale;

                    painter.thickness = f32::powf(2.5, 2.8) / 40.0 * scale - circle_size;
                    painter.hollow = true;
                    painter.set_color(ORANGE.pastel() + WHITE * circle_size);
                    painter.translate(position + Vec3::Y * circle_size * 2.0 * scale);
                    painter.circle(circle_size);
                }
                painter.translate(Vec3::NEG_Y * 0.6);
                let start_pos = painter.transform;
                draw_bubble(&mut painter, seconds, Vec3::X + Vec3::NEG_Y * 0.6, 1.1);
                painter.transform = start_pos;
                draw_bubble(&mut painter, seconds + 0.5, Vec3::NEG_Y, 1.7);
                painter.transform = start_pos;
                draw_bubble(
                    &mut painter,
                    seconds + PI / 3.0,
                    Vec3::NEG_X + Vec3::NEG_Y,
                    1.3,
                );
                painter.transform = start_pos;
                draw_bubble(
                    &mut painter,
                    seconds + PI / 2.0,
                    Vec3::NEG_X * 0.5 + Vec3::NEG_Y * 1.2,
                    1.9,
                );
                painter.transform = start_pos;
                draw_bubble(
                    &mut painter,
                    seconds + PI / 1.2,
                    Vec3::X * 0.7 + Vec3::NEG_Y * 1.4,
                    1.4,
                );
            }
            // Arc examples
            3 => {
                painter.hollow = false;
                painter.cap = Cap::None;
                painter.set_color(CRIMSON.pastel() * 0.7);
                painter.translate(diag_vec * 0.8);
                painter.arc(0.8, 0.0, TAU * (2. / 3.));

                painter.hollow = true;
                painter.cap = Cap::Round;
                painter.thickness = 0.4;
                painter.set_color(CRIMSON.pastel() * 1.4);
                painter.translate(-diag_vec * 1.6);
                painter.arc(0.8, 0.0, TAU * (2. / 3.));
            }
            8 => {
                let start_angle = seconds * 3.0;
                let end_angle = start_angle + PI * (seconds.sin() + 1.) * 0.75 + 0.5 * PI;

                painter.thickness = 0.5;
                painter.hollow = true;
                painter.set_color(CRIMSON.pastel());
                painter.cap = Cap::None;
                painter.arc(1.5, start_angle, end_angle);
            }
            13 => {
                let meter_fill = ((seconds).sin() + 1.0) / 2.0;
                let meter_size = PI * 1.5;

                let start_angle = -meter_size / 2.0;
                let end_angle = -meter_size / 2.0 + meter_fill * meter_size;

                painter.hollow = true;
                painter.cap = Cap::Round;
                painter.thickness = 0.4;
                painter.set_color(CRIMSON.pastel() * (1.0 / (0.8 + meter_fill * 0.6)));
                painter.arc(1.3, start_angle, end_angle);

                painter.cap = Cap::None;
                painter.thickness = 0.2;
                painter.set_color(WHITE);
                painter.arc(1.6, start_angle, -start_angle);
                painter.arc(0.8, start_angle, -start_angle);

                let offset = Quat::from_rotation_z(start_angle) * Vec3::Y * 1.1;
                painter.translate(offset);
                painter.arc(0.5, start_angle + PI * 1.5, start_angle + 2.5 * PI);
                painter.translate(-offset);

                painter.translate(Quat::from_rotation_z(-start_angle) * Vec3::Y * 1.1);
                painter.arc(0.5, start_angle + PI, start_angle + 2.0 * PI);
            }
            // Polygon examples
            4 => {
                painter.thickness = 0.4;
                painter.hollow = true;
                painter.set_color(PURPLE.pastel() * 0.6);
                painter.translate(diag_vec);
                painter.ngon(4., 0.8);

                painter.roundness = 0.1;
                painter.hollow = false;
                painter.set_color(PURPLE.pastel() * 0.8);
                painter.translate(-Vec3::X * 2.0);
                painter.ngon(3., 0.8);

                painter.hollow = true;
                painter.set_color(PURPLE.pastel());
                painter.translate(-Vec3::Y * 2.0);
                painter.ngon(5., 0.8);

                painter.roundness = 0.0;
                painter.hollow = false;
                painter.set_color(PURPLE.pastel() * 1.2);
                painter.translate(Vec3::X * 2.0);
                painter.ngon(6., 0.8);
            }
            9 => {
                painter.hollow = true;
                painter.thickness = 0.5;
                painter.set_color(PURPLE.pastel());
                painter.roundness = 0.5;
                painter.ngon(3. + (seconds.sin() + 1.) * 3., 1.5);
            }
            14 => {
                const HEX_RADIUS: f32 = 0.35;
                const BOUNDS: f32 = 1.8;

                let down_vec = Quat::from_rotation_z(-PI * 7.0 / 6.0) * Vec3::Y * HEX_RADIUS * 2.0;
                let right_vec = Quat::from_rotation_z(-PI * 5.0 / 6.0) * Vec3::Y * HEX_RADIUS * 2.0;

                fn draw_gon(painter: &mut ShapePainter, origin: Vec3, sides: f32, radius: f32) {
                    let dist = ((painter.transform.translation - origin) / painter.transform.scale)
                        .length();
                    if dist <= BOUNDS {
                        let ratio = 1.0 - f32::max(dist, 0.5) / BOUNDS;
                        painter.set_color(PURPLE.pastel());
                        painter.color.set_alpha(ratio);
                        painter.ngon(sides, radius * f32::powf(ratio, 0.2) * 0.8);
                    }
                }

                fn draw_position(painter: &mut ShapePainter, pos: Vec3, right: Vec3) {
                    let origin_tf = painter.transform;

                    painter.translate(pos);
                    draw_gon(painter, origin_tf.translation, 6.0, HEX_RADIUS);

                    let tri_radius = 2.0 * HEX_RADIUS / (2.0 * f32::sqrt(3.0));
                    let normal = right.normalize().cross(Vec3::Z) * tri_radius;

                    painter.translate(right * 0.5 - normal);
                    draw_gon(painter, origin_tf.translation, 3.0, tri_radius);

                    painter.translate(normal * 2.0);
                    painter.rotate_z(PI);
                    draw_gon(painter, origin_tf.translation, 3.0, tri_radius);

                    painter.transform = origin_tf;
                }

                painter.hollow = false;
                painter.roundness = 0.0;

                let offset = down_vec * (seconds % PI) / PI
                    + right_vec * (seconds % (PI / 3.0)) / (PI / 3.0);
                for x in -6..6 {
                    for y in -6..6 {
                        draw_position(
                            &mut painter,
                            down_vec * y as f32 + right_vec * x as f32 + offset,
                            right_vec,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

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
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off,
        RenderLayers::default(),
    ));
}

fn draw_gallery(time: Res<Time>, painter: ShapePainter) {
    gallery(painter, time.elapsed_secs(), 0..15);
}
