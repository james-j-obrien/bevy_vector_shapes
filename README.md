<div align="center">
<h1>
    Bevy Vector Shapes
</h1>

[![crates.io](https://img.shields.io/crates/v/bevy_vector_shapes)](https://crates.io/crates/bevy_vector_shapes)
[![docs.rs](https://docs.rs/bevy_vector_shapes/badge.svg)](https://docs.rs/bevy_vector_shapes)
[![CI](https://github.com/james-j-obrien/bevy_vector_shapes/workflows/Rust/badge.svg?branch=main)](https://github.com/james-j-obrien/bevy_vector_shapes/actions?query=workflow%3A%22Rust%22+branch%3Amain)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

<img src="assets/shapes_gallery_3d.gif" alt="Bevy Vector Shapes"/>
</div>

## What is Bevy Vector Shapes?
Bevy Vector Shapes is a library for easily and ergonomically creating instanced vector shapes in [Bevy Engine](https://bevyengine.org/).

## WARNING
Bevy Vector Shapes is in the very early stages of development. There may be issues and some documentation may be sparse.

## Features
- Variety of shape types: lines, rectangles, circles, arcs and regular polygons.
- Supports various bevy rendering features: 2D and 3D pipelines, transparency, alpha modes, render layers, bloom.
- Immediate and retained mode.
- Local anti-aliasing for smoother looking shapes.
- Optional billboarding for each shape type to ensure they are always facing the camera.
- Shapes of the same type and rendering configuration are fully instanced together.
- Compilation to wasm to run your projects in the browser.

## Usage
See the `minimal_2d` or `minimal_3d` examples for basic usage and the remaining examples for explorations of supported features.

```rust
use bevy::prelude::*;
// Import commonly used items
use bevy_vector_shapes::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the shape plugin, ShapePlugin for 3D cameras and Shape2dPlugin for 2D cameras
        .add_plugin(Shape2dPlugin::default())
        .add_startup_system(setup)
        .add_system(draw)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn(Camera2dBundle::default());
}

fn draw(mut painter: ShapePainter) {
    // Draw a circle
    painter.circle(100.0);
}
```

| bevy | bevy_vector_shapes |
| ---- | ------------------ |
| 0.10 | 0.3.1              |