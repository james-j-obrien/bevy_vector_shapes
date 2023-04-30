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
  
    @location(7) radius: f32,
    @location(8) start_angle: f32, 
    @location(9) end_angle: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) thickness: f32,
    @location(3) angle: f32,
    @location(4) delta: f32,
    @location(5) cap: u32
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

    let matrix = mat4x4<f32>(
        v.matrix_0,
        v.matrix_1,
        v.matrix_2,
        v.matrix_3
    );

    var vertex_data = get_vertex_data(matrix, vertex.xy * v.radius, v.thickness, v.flags);

    // Multiply the world space position by the view projection matrix to convert to our clip position
    out.clip_position = vertex_data.clip_pos;
    out.uv = vertex.xy * vertex_data.uv_ratio;
    out.thickness = calculate_thickness(vertex_data.thickness_data, v.radius, v.flags);

    // Extract cap type from flags
    out.cap = f_cap(v.flags);

    // Setup angles for the fragment shader if we are an arc
    var arc = f_arc(v.flags);
    if arc > 0u {
        // Transform our angles such that 0 points towards y up
        var delta = (v.end_angle - v.start_angle) / 2.0;
        out.angle = (v.start_angle - PI / 2.0 + delta);
        out.delta = delta;

        // Rotate our uv space such that y up is towards the center of our arc
        out.uv = rotate_vec_a(out.uv, -out.angle);
    } else {
        out.angle = 0.0;
        out.delta = PI;
    }

    out.color = v.color;
    return out;
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) thickness: f32,
    @location(3) angle: f32,
    @location(4) delta: f32,
    @location(5) cap: u32
};

// Due to https://github.com/gfx-rs/naga/issues/1743 this cannot be compiled into the vertex shader on web
#ifdef FRAGMENT
@fragment
fn fragment(f: FragmentInput) -> @location(0) vec4<f32> {
    // Mask representing whether this fragment falls within the shape
    var in_shape = f.color.a;

    // Cut off points outside the shape or within the hollow area
    var dist = length(f.uv) - 1.;
    in_shape *= step_aa(-f.thickness, dist) * step_aa(dist, 0.);

    // Cut off points outside the allowed range of angles
    var angle = atan2(f.uv.y, f.uv.x);
    in_shape *= step_aa_pd(-f.delta, angle, abs(angle)) * step_aa_pd(angle, f.delta, abs(angle));

    // Handle rounded caps
    if f.cap == 2u {
        // Take the delta in the direction towards our point
        var nearest_angle = sign(angle) * f.delta;

        // With that delta find the point at the end of the arc
        // Use thickness to offset from the radius
        var end_point = vec2<f32>(cos(nearest_angle), sin(nearest_angle)) * (1.0 - f.thickness / 2.0);

        // Mask in points near the end point based on our thickness
        var dist = length(end_point - f.uv);

        var mask = step_aa(dist, f.thickness / 2.0);
        in_shape = max(in_shape, mask);
    }

    // Discard fragments no longer in the shape
    if in_shape < 0.0001 {
        discard;
    }

    return color_output(vec4<f32>(f.color.rgb, in_shape));
}
#endif