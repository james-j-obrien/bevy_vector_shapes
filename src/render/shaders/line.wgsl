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

    @location(7) start: vec3<f32>,
    @location(8) end: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) cap_ratio: f32
};

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // Vertex positions for a basic quad
    let vertex = get_quad_vertex(v);

    // Reconstruct our transformation matrix
    let matrix = mat4x4<f32>(
        v.matrix_0,
        v.matrix_1,
        v.matrix_2,
        v.matrix_3
    );

    // Vector from start -> end
    var line_vec = v.end - v.start;

    // Center of line in world space
    var center = line_vec / 2.0;

    // Line length in local space
    var line_length = length(line_vec);

    // Get our start and end in world space
    var world_start = (matrix * vec4<f32>(v.start, 1.0)).xyz;
    var world_end = (matrix * vec4<f32>(v.end, 1.0)).xyz;

    // The y basis is the normalized vector along the line
    var y_basis = normalize(world_start - world_end);

    // Choose which point we will work in reference to based on our y position
    var origin = select(world_end, world_start, vertex.y < 0.0);

    // Calculate the remainder of our basis vectors
    var basis_vectors = get_basis_vectors_from_up(matrix, origin, y_basis, v.flags);

    // Calculate thickness data
    var thickness_type = f_thickness_type(v.flags);
    var thickness_data = get_thickness_data(v.thickness, thickness_type, origin, basis_vectors[1]);

    let scale = vec3<f32>(length(matrix[0].xyz), length(matrix[1].xyz), length(matrix[2].xyz));

    // If our thickness in pixels is less than 1, clamp to 1 and reduce the alpha instead
    var out_color = v.color;
    if thickness_data.thickness_p * max(scale.x, scale.y) < 1.0 {
        out_color.a = out_color.a * thickness_data.thickness_p;
        thickness_data.thickness_p = 1.;
    }

    // Calculate thickness and radius in world units
    var thickness = thickness_data.thickness_p / thickness_data.pixels_per_u;
    var radius = thickness / 2.0;

    var cap_type = f_cap(v.flags);
    var cap_length = 0.0;

    // If we have caps increase the cap length to our radius
    if cap_type > 0u {
        cap_length = radius;
    }

    // If our caps are round store the ratio of the length of our caps to the entire length of the line
    if cap_type == 2u {
        out.cap_ratio = thickness / (line_length + thickness);
    }

    // Calculate the vertex position with scaling
    var local_pos = vertex.xy * vec2<f32>(radius, cap_length + line_length / 2.0) * scale.xy;

    // Scale our padding to world space and match direction of our vertex
    var aa_padding_u = AA_PADDING / thickness_data.pixels_per_u;
    var aa_padding = sign(vertex.xy) * aa_padding_u;

    // Pad our position and determine the ratio by which to scale uv such that uvs ignore the padding
    var padded_pos = local_pos + aa_padding;
    var uv_ratio = padded_pos / local_pos;

    // Caluclate the offset from our origin point
    var local_offset = vertex.xy * (vec2<f32>(radius, cap_length) * scale.xy + aa_padding_u);

    // Determine final world position by offsetting by the origin we chose and rotating by our basis vectors
    var world_pos = origin + local_offset.x * basis_vectors[0] + local_offset.y * basis_vectors[1];

    // Multiply the world space position by the view projection matrix to convert to our clip position
    out.clip_position = view.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = vertex.xy * uv_ratio;

    out.color = out_color;
    return out;
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) cap_ratio: f32
};

// Due to https://github.com/gfx-rs/naga/issues/1743 this cannot be compiled into the vertex shader on web
#ifdef FRAGMENT
@fragment
fn fragment(f: FragmentInput) -> @location(0) vec4<f32> {
    var in_shape = f.color.a;

    // If we have rounded caps mask them
    if f.cap_ratio > 0.0 {
        // Lines are symmetrical across both axis so we can mirror our point 
        //  onto the positive x and y axis by taking the absolute value
        var pos = abs(f.uv);

        // Our x value already represents the x distance to our line so we now must transform our y value
        // We want y = 0 to represent being within the body of the line, and 1 to be at the tip of our cap

        // Calculate the -y distance to the end of the quad in caps
        // The end of the quad is y = 1 so subtract to get the distance and then scale by cap length
        var to_end_cap = (pos.y - 1.) / f.cap_ratio;

        // We want y = 0 when the amount of caps until the end of the quad is 1 
        //  and y = 1 when the number of quads is 0 so take 1 + to_end_cap
        // If the total is negative we are within the line so clamp to > 0
        pos.y = max(0., 1. + to_end_cap);

        // We now have the shortest vector from our point to the line so take the distance
        var dist = length(pos);

        // Mask out corners
        in_shape = step_aa(dist, 1.);
    } else {
        // Simple rectangle sdf for no caps or square caps
        in_shape = step_aa(abs(f.uv.x), 1.) * step_aa(abs(f.uv.y), 1.0);
    }

    // Discard fragments no longer in the shape
    if in_shape < 0.0001 {
        discard;
    }

    return vec4<f32>(f.color.rgb, in_shape);
}
#endif