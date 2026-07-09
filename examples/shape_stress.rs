// Stress test for measuring the CPU and GPU load.
//
// Env vars:
// - MODE=immediate|retained            which authoring API to stress (default: immediate)
// - COUNT=<n>                          total number of shapes (default: 50000)
// - TYPES=disc,rect,line,ngon,tri      which shape types to draw (default: all five)
// - INTERLEAVE=0|1                     1 alternates types per shape (worst-case batching),
//                                      0 lays them out in contiguous blocks (default: 0)
// - MATERIALS=<n>                      number of unique materials via render-layer variants (default: 1)
// - DIM=2d|3d                          which pipeline to stress (default: 2d)
// - VSYNC=0|1                          0 disables vsync for unthrottled sampling (default: 1)
// - EXIT_AFTER=<secs>                  automatically exit after this many seconds, for scripted runs
//
//   COUNT=20000 INTERLEAVE=1 cargo run --release --example shape_stress

use std::str::FromStr;

use bevy::{
    camera::visibility::RenderLayers,
    color::palettes::css::*,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::diagnostic::RenderDiagnosticsPlugin,
    window::PresentMode,
};
use bevy_vector_shapes::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShapeType {
    Disc,
    Rect,
    Line,
    Ngon,
    Tri,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Immediate,
    Retained,
}

#[derive(Resource, Clone)]
struct StressConfig {
    mode: Mode,
    count: usize,
    types: Vec<ShapeType>,
    interleave: bool,
    materials: usize,
    is_3d: bool,
    exit_after: Option<f32>,
}

fn env_or<T: FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

impl StressConfig {
    fn from_env() -> Self {
        let types = std::env::var("TYPES")
            .map(|v| {
                v.split(',')
                    .filter_map(|t| match t.trim() {
                        "disc" => Some(ShapeType::Disc),
                        "rect" => Some(ShapeType::Rect),
                        "line" => Some(ShapeType::Line),
                        "ngon" => Some(ShapeType::Ngon),
                        "tri" => Some(ShapeType::Tri),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .ok()
            .filter(|t: &Vec<ShapeType>| !t.is_empty())
            .unwrap_or_else(|| {
                vec![
                    ShapeType::Disc,
                    ShapeType::Rect,
                    ShapeType::Line,
                    ShapeType::Ngon,
                    ShapeType::Tri,
                ]
            });

        Self {
            mode: if env_or("MODE", "immediate".to_string()) == "retained" {
                Mode::Retained
            } else {
                Mode::Immediate
            },
            count: env_or("COUNT", 50_000),
            types,
            interleave: env_or("INTERLEAVE", 0u8) != 0,
            materials: env_or("MATERIALS", 1usize).max(1),
            is_3d: env_or("DIM", "2d".to_string()) == "3d",
            exit_after: std::env::var("EXIT_AFTER")
                .ok()
                .and_then(|v| v.parse().ok()),
        }
    }

    fn shape_type(&self, index: usize) -> ShapeType {
        let n = self.types.len();
        if self.interleave {
            self.types[index % n]
        } else {
            self.types[(index * n / self.count).min(n - 1)]
        }
    }

    // Grid position and shape size for the given index.
    fn layout(&self, index: usize) -> (Vec3, f32) {
        let axis = (self.count as f32).sqrt().ceil().max(1.0);
        let col = (index as f32) % axis;
        let row = (index as f32) / axis;
        if self.is_3d {
            let spacing = 2.0;
            (Vec3::new(col * spacing, 0.0, row.floor() * spacing), 0.8)
        } else {
            let spacing = 700.0 / axis;
            // Spread z so the transparent phase does real sorting work, as it would
            // in an application that layers its shapes.
            let z = index as f32 * 0.001;
            (
                Vec3::new(
                    (col - axis / 2.0) * spacing,
                    (row.floor() - axis / 2.0) * spacing,
                    z,
                ),
                spacing * 0.4,
            )
        }
    }

    fn color(&self, index: usize) -> Color {
        const COLORS: [Srgba; 5] = [SEA_GREEN, CORNFLOWER_BLUE, ORANGE, MEDIUM_PURPLE, CRIMSON];
        COLORS[index % COLORS.len()].into()
    }

    fn render_layers(&self, index: usize) -> Option<RenderLayers> {
        (self.materials > 1).then(|| RenderLayers::default().with(1 + index % self.materials))
    }
}

fn main() {
    let present_mode = if env_or("VSYNC", 1u8) == 0 {
        PresentMode::AutoNoVsync
    } else {
        PresentMode::AutoVsync
    };
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ShapePlugin::default())
        .insert_resource(ClearColor(DIM_GRAY.into()))
        .insert_resource(StressConfig::from_env())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(RenderDiagnosticsPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(Startup, (setup, spawn_retained))
        .add_systems(Update, (draw_immediate, exit_after))
        .run();
}

fn setup(config: Res<StressConfig>, mut commands: Commands) {
    info!(
        "shape_stress: mode={} count={} types={} interleave={} materials={} dim={}",
        if config.mode == Mode::Retained {
            "retained"
        } else {
            "immediate"
        },
        config.count,
        config.types.len(),
        config.interleave,
        config.materials,
        if config.is_3d { "3d" } else { "2d" },
    );

    if config.is_3d {
        let axis = (config.count as f32).sqrt().ceil();
        let center = Vec3::new(axis, 0.0, axis);
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(-20.0, 20.0, -20.0).looking_at(center, Vec3::Y),
            Msaa::Off,
        ));
    } else {
        commands.spawn((Camera2d, Msaa::Off));
    }
}

fn exit_after(config: Res<StressConfig>, time: Res<Time>, mut exit: MessageWriter<AppExit>) {
    if let Some(secs) = config.exit_after {
        if time.elapsed_secs() > secs {
            exit.write(AppExit::Success);
        }
    }
}

fn configure(shape_config: &mut ShapeConfig, config: &StressConfig) {
    if config.is_3d {
        shape_config.set_3d();
        shape_config.alignment = Alignment::Billboard;
    } else {
        shape_config.set_2d();
    }
}

fn spawn_retained(config: Res<StressConfig>, mut shapes: ShapeCommands) {
    if config.mode != Mode::Retained {
        return;
    }
    configure(&mut shapes, &config);
    for i in 0..config.count {
        let (pos, size) = config.layout(i);
        shapes.transform = Transform::from_translation(pos);
        shapes.set_color(config.color(i));
        shapes.render_layers = config.render_layers(i);
        shapes.thickness = size * 0.15;
        match config.shape_type(i) {
            ShapeType::Disc => {
                shapes.circle(size);
            }
            ShapeType::Rect => {
                shapes.rect(Vec2::splat(size * 2.0));
            }
            ShapeType::Line => {
                shapes.line(Vec3::new(-size, -size, 0.0), Vec3::new(size, size, 0.0));
            }
            ShapeType::Ngon => {
                shapes.ngon(6.0, size);
            }
            ShapeType::Tri => {
                shapes.triangle(
                    Vec2::new(0.0, size),
                    Vec2::new(-size, -size),
                    Vec2::new(size, -size),
                );
            }
        }
    }
}

fn draw_immediate(config: Res<StressConfig>, mut painter: ShapePainter) {
    if config.mode != Mode::Immediate {
        return;
    }
    configure(&mut painter, &config);
    for i in 0..config.count {
        let (pos, size) = config.layout(i);
        painter.transform = Transform::from_translation(pos);
        painter.set_color(config.color(i));
        painter.render_layers = config.render_layers(i);
        painter.thickness = size * 0.15;
        match config.shape_type(i) {
            ShapeType::Disc => {
                painter.circle(size);
            }
            ShapeType::Rect => {
                painter.rect(Vec2::splat(size * 2.0));
            }
            ShapeType::Line => {
                painter.line(Vec3::new(-size, -size, 0.0), Vec3::new(size, size, 0.0));
            }
            ShapeType::Ngon => {
                painter.ngon(6.0, size);
            }
            ShapeType::Tri => {
                painter.triangle(
                    Vec2::new(0.0, size),
                    Vec2::new(-size, -size),
                    Vec2::new(size, -size),
                );
            }
        }
    }
}
