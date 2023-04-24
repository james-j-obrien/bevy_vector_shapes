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
- Supports various bevy rendering features: transparency, alpha modes, render layers, bloom.
- Immediate and retained mode.
- Local anti-aliasing for smoother looking shapes.
- Optional billboarding for each shape type to ensure they are always facing the camera.
- Shapes of the same type and rendering configuration are fully batched and instanced together.

## Usage
See the `minimal_2d` or `minimal_3d` examples for basic usage.

| bevy | bevy_vector_shapes |
| ---- | ------------------ |
| 0.10 | 0.1                |


## License

bevy_vector_shapes is free and open source! All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option. This means you can select the license you prefer! This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are very good reasons to include both.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.