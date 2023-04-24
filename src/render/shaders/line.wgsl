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
    var vertexes: array<vec3<f32>, 6u> = array<vec3<f32>, 6u>(
        vec3<f32>(-1.0, 1.0, 0.0),
        vec3<f32>(1.0, 1.0, 0.0),
        vec3<f32>(1.0, -1.0, 0.0),
        vec3<f32>(1.0, -1.0, 0.0),
        vec3<f32>(-1.0, -1.0, 0.0),
        vec3<f32>(-1.0, 1.0, 0.0),
    );
    let vertex = vertexes[v.index];

    let matrix = mat4x4<f32>(
        v.matrix_0,
        v.matrix_1,
        v.matrix_2,
        v.matrix_3
    );

    let a = (matrix * vec4<f32>(v.start, 1.0)).xyz;
    let b = (matrix * vec4<f32>(v.end, 1.0)).xyz;

    // Vector from A -> B
    var line_vec = b - a;
    // Line length in world space
    var line_length = length(line_vec);

    // In order to determine the rotated position for the vertex of our quad
    //  we must calculate each of the basis vectors

    // The y basis is the normalized vector along the line
    var y_basis = -normalize(line_vec);

    // Choose which point we will work in reference to based on our y position
    var origin = select(b, a, vertex.y < 0.0);

    // Calculate the remainder of our basis vectors
    var basis_vectors = get_basis_vectors_from_up(matrix, origin, y_basis, v.flags);

    // Calculate thickness data
    var thickness_type = f_thickness_type(v.flags);
    var thickness_data = get_thickness_data(v.thickness, thickness_type, origin, basis_vectors[1]);

    // If our thickness in pixels is less than 1, clamp to 1 and reduce the alpha instead
    var out_color = v.color;
    if thickness_data.thickness_p < 1.0 {
        out_color.a = out_color.a * thickness_data.thickness_p;
        thickness_data.thickness_p = 1.;
    }

    // Calculate thickness and radius in world units
    var thickness_u = thickness_data.thickness_p / thickness_data.pixels_per_u;
    var radius_u = thickness_u / 2.;

    // Calculate XY sacle
    var scale = get_scale(matrix);

    var cap_type = f_cap(v.flags);
    var cap_length = 0.0;

    // If we have caps increase the cap length to our radius
    if cap_type > 0u {
        cap_length = radius_u;
    }

    // If our caps are round store the ratio of the length of our caps to the entire length of the line
    if cap_type == 2u {
        out.cap_ratio = thickness_u / (line_length + thickness_u);
    }


    // Calculate the local positioning of the vertex by multiplying by our basis vectors
    var local_pos = vertex.x * basis_vectors[0] * radius_u + cap_length * vertex.y * basis_vectors[1];

    // Scale our padding to world space and match direction of our vertex
    var aa_padding_u = AA_PADDING / thickness_data.pixels_per_u;
    var aa_padding = vertex.xy * aa_padding_u;

    // Rotate our padding into 3 dimensions by multiplying by basis vectors
    var world_aa_padding = aa_padding.x * basis_vectors[0] + aa_padding.y * basis_vectors[1];

    // In order to scale our padding into uv space we need to know the 2d local space coordinate of our vertex
    var local_pos_xy = vertex.xy * vec2<f32>(radius_u, line_length / 2.0 + cap_length);

    // Pad our position and determine the ratio by which to scale uv such that uvs ignore the padding
    var padded_pos = local_pos + world_aa_padding;
    var uv_ratio = (local_pos_xy + aa_padding) / local_pos_xy;

    // Determine final world position by offsetting by the origin we chose
    var world_pos = origin + padded_pos;

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