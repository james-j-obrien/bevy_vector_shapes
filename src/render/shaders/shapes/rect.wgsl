#import bevy_vector_shapes::core
#import bevy_vector_shapes::core::{view, image, image_sampler}

struct Vertex {
    @builtin(instance_index) index: u32,
    @location(0) pos: vec3<f32>
};

struct Shape {
    @location(0) matrix_0: vec4<f32>,
    @location(1) matrix_1: vec4<f32>,
    @location(2) matrix_2: vec4<f32>,
    @location(3) matrix_3: vec4<f32>,

    @location(4) color: vec4<f32>,  
    @location(5) thickness: f32,
    @location(6) flags: u32,

    @location(7) size: vec2<f32>,
    @location(8) corner_radii: vec4<f32>,
}

#ifdef PER_OBJECT_BUFFER_BATCH_SIZE
@group(1) @binding(0) var<uniform> shapes: array<Shape, #{PER_OBJECT_BUFFER_BATCH_SIZE}u>;
#else
@group(1) @binding(0) var<storage> shapes: array<Shape>;
#endif 

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) corner_radii: vec4<f32>,
    @location(4) thickness: f32,
#ifdef TEXTURED
    @location(5) texture_uv: vec2<f32>,
#endif
};

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let vertex = v.pos;
    let shape = shapes[v.index];
    let matrix = mat4x4<f32>(
        shape.matrix_0,
        shape.matrix_1,
        shape.matrix_2,
        shape.matrix_3
    );
    let shortest_side = min(shape.size.x, shape.size.y);
    let half_shortest_side = shortest_side / 2.0;

    out.size = shape.size / shortest_side;

#ifdef PIPELINE_2D
    let local_position = vertex.xy * shape.size / 2.0;
    let scale = max(core::get_scale(matrix), vec2<f32>(0.000001));
    let scaled_position = local_position * scale;
    let y_axis = matrix[1].xyz;
    let y_axis_length = length(y_axis);
    var y_direction = vec3<f32>(0.0, 1.0, 0.0);
    if y_axis_length > 0.000001 {
        y_direction = y_axis / y_axis_length;
    }
    let thickness_data = core::get_thickness_data(
        shape.thickness,
        core::f_thickness_type(shape.flags),
        matrix[3].xyz,
        y_direction,
    );
    let aa_padding = core::AA_PADDING / thickness_data.pixels_per_u;
    let padded_local_position = local_position + sign(local_position) * aa_padding / scale;
    let uv_ratio = (abs(scaled_position) + aa_padding) / max(abs(scaled_position), vec2<f32>(0.000001));

    out.clip_position = view.view_proj * matrix * vec4<f32>(padded_local_position, 0.0, 1.0);
    out.uv = vertex.xy * out.size * uv_ratio;
    out.thickness = core::calculate_thickness(thickness_data, half_shortest_side, shape.flags);
#endif

#ifdef PIPELINE_3D
    let local_position = vertex.xy * shape.size / 2.0;
    let origin = matrix[3].xyz;
    let alignment = core::f_alignment(shape.flags);

    var y_basis = normalize(matrix[1].xyz);
    var z_basis = normalize(matrix[2].xyz);
    if alignment == 1u {
        y_basis = normalize((view.view * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz);
        z_basis = normalize(view.world_position - origin);
    }

    let x_basis = normalize(cross(y_basis, z_basis));
    y_basis = cross(x_basis, z_basis);

    let scale = core::get_scale(matrix);
    let scaled_position = local_position * scale;
    let thickness_data = core::get_thickness_data(
        shape.thickness,
        core::f_thickness_type(shape.flags),
        origin,
        y_basis,
    );
    let aa_padding = core::AA_PADDING / thickness_data.pixels_per_u;
    let padded_position = scaled_position + sign(local_position) * aa_padding;
    let uv_ratio = padded_position / scaled_position;
    let world_position = origin + padded_position.x * x_basis + padded_position.y * y_basis;

    out.clip_position = view.view_proj * vec4<f32>(world_position, 1.0);
    out.uv = vertex.xy * out.size * uv_ratio;
    out.thickness = core::calculate_thickness(thickness_data, half_shortest_side, shape.flags);
#endif

    out.corner_radii = 2.0 * min(shape.corner_radii / shortest_side, vec4<f32>(0.5));
    out.color = shape.color;
#ifdef TEXTURED
    out.texture_uv = core::get_texture_uv(vertex.xy);
#endif
    return out;
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) corner_radii: vec4<f32>,
    @location(4) thickness: f32,
#ifdef TEXTURED
    @location(5) texture_uv: vec2<f32>,
#endif
};

// Given a position, and a size determine the distance between a point and the rectangle with those side lengths
fn rectSDF(position: vec2<f32>, size: vec2<f32>) -> f32 {
    // Rectangles are symmetrical across both axis so we can mirror our point 
    // into the positive x and y axis by taking the absolute value
    var pos = abs(position);

    // Calculate the vector from the corner of the rect to our point
    var to_corner = pos - size;

    // By clamping away negative values we now have the vector to the edge of the rect
    // from outside, however if we are inside the rect this is all 0s
    var outside_to_edge = max(vec2<f32>(0.), to_corner);

    // If the point is inside the rect then it is always below or to the left of our corner 
    // so take the largest negative value from our vector, this will be 0 outside the rect
    var inside_length = min(0., max(to_corner.x, to_corner.y));

    // Combining these two lengths gives us the length for all cases
    return length(outside_to_edge) + inside_length;
}

// Given a uv position get which quadrant that position is in
// Return an integer from 0 to 3
fn quadrant(in: vec2<f32>) -> i32 {
    var uv = vec2<i32>(sign(in));
    return -uv.y + (-uv.x * uv.y + 3) / 2;
}

// Due to https://github.com/gfx-rs/naga/issues/1743 this cannot be compiled into the vertex shader on web
#ifdef FRAGMENT
@fragment
fn fragment(f: FragmentInput) -> @location(0) vec4<f32> {
    let radius = f.corner_radii[quadrant(f.uv)];
    let signed_distance = rectSDF(f.uv, f.size - radius) - radius;
    let alpha = f.color.a
        * core::step_aa(-f.thickness, signed_distance)
        * core::step_aa(signed_distance, 0.0);

    var color = core::color_output(vec4<f32>(f.color.rgb, alpha));

#ifdef TEXTURED
    color = color * textureSample(image, image_sampler, f.texture_uv);
#endif

    return color;
}
#endif
