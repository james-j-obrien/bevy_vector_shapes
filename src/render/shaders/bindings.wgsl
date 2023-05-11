#define_import_path bevy_vector_shapes::bindings

const PI: f32 = 3.14159265359;
const TAU: f32 = 6.28318530718;

struct ColorGrading {
    exposure: f32,
    gamma: f32,
    pre_saturation: f32,
    post_saturation: f32,
}

struct View {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    world_position: vec3<f32>,
    // viewport(x_origin, y_origin, width, height)
    viewport: vec4<f32>,
    grading: ColorGrading,
};

@group(0) @binding(0)
var<uniform> view: View;

#ifdef TEXTURED
#ifdef FRAGMENT

@group(1) @binding(0)
var image: texture_2d<f32>;

@group(1) @binding(1)
var image_sampler: sampler;

#endif
#endif