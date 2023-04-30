#import bevy_vector_shapes::core

struct Vertex {
    @builtin(vertex_index) index: u32,
    @location(0) matrix_0: vec4<f32>,
    @location(1) matrix_1: vec4<f32>,
    @location(2) matrix_2: vec4<f32>,
    @location(3) matrix_3: vec4<f32>,

    @location(4) color: vec4<f32>,  
    @location(5) thickness: f32,
    @location(6) flags: u32,

    @location(7) size: vec2<f32>,
    @location(8) corner_radii: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) corner_radii: vec4<f32>,
    @location(4) thickness: f32,
};

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // Vertex positions for a basic quad
    var vertexes: array<vec3<f32>, 6u> = array<vec3<f32>, 6u>(
        vec3<f32>(-1.0, 1.0, 0.0),
        vec3<f32>(1.0, 1.0, 0.0),
        vec3<f32>(1.0, -1.0, 0.0),
        vec3<f32>(1.0, -1.0, 0.0),
        vec3<f32>(-1.0, -1.0, 0.0),
        vec3<f32>(-1.0, 1.0, 0.0),
    );
    let vertex = vertexes[v.index % 6u];

    // Reconstruct our transformation matrix
    let matrix = mat4x4<f32>(
        v.matrix_0,
        v.matrix_1,
        v.matrix_2,
        v.matrix_3
    );
    // Shortest of the two side lengths for the rectangle
    var shortest_side = min(v.size.x, v.size.y);

    var vertex_data = get_vertex_data(matrix, vertex.xy * v.size / 2.0, v.thickness, v.flags);
    out.clip_position = vertex_data.clip_pos;

    // Our vertex outputs should all be in uv space so scale our uv space such that the shortest side is of length 1
    out.size = v.size / shortest_side;
    out.uv = vertex.xy * out.size * vertex_data.uv_ratio;
    out.thickness = calculate_thickness(vertex_data.thickness_data, shortest_side / 2.0, v.flags);

    // Our corner radii cannot be more than half the shortest side so cap them
    out.corner_radii = 2.0 * min(v.corner_radii / shortest_side, vec4<f32>(0.5));

    out.color = v.color;
    return out;
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) corner_radii: vec4<f32>,
    @location(4) thickness: f32,
};

// Given a position, and a size determine the distance between a point and the rectangle with those side lengths
fn rectSDF(pos: vec2<f32>, size: vec2<f32>) -> f32 {
    // Rectangles are symmetrical across both axis so we can mirror our point 
    // into the positive x and y axis by taking the absolute value
    var pos = abs(pos);

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
fn quadrant(uv: vec2<f32>) -> i32 {
    var uv = vec2<i32>(sign(uv));
    return -uv.y + (-uv.x * uv.y + 3) / 2;
}

// Due to https://github.com/gfx-rs/naga/issues/1743 this cannot be compiled into the vertex shader on web
#ifdef FRAGMENT
@fragment
fn fragment(f: FragmentInput) -> @location(0) vec4<f32> {
    // Mask representing whether this fragment falls within the shape
    var in_shape = f.color.a;

    // Use quadrant to determine which corner radii to use
    var quadrant = quadrant(f.uv);
    var radii = f.corner_radii[quadrant];

    // Calculate our positions distance from the rectangle
    var dist = rectSDF(f.uv, f.size - radii) - radii;
    
    // Cut off points outside the shape or within the hollow area
    in_shape *= step_aa(-f.thickness, dist) * step_aa(dist, 0.);

    // Discard fragments no longer in the shape
    if in_shape < 0.0001 {
        discard;
    }

    return vec4<f32>(f.color.rgb, in_shape);
}
#endif