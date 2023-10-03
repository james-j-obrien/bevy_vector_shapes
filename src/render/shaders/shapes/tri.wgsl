#import bevy_vector_shapes::core as core
#import bevy_vector_shapes::core view, image, image_sampler
#import bevy_vector_shapes::constants PI, TAU

struct Vertex {
    @builtin(vertex_index) index: u32,
    @location(0) matrix_0: vec4<f32>,
    @location(1) matrix_1: vec4<f32>,
    @location(2) matrix_2: vec4<f32>,
    @location(3) matrix_3: vec4<f32>,

    @location(4) color: vec4<f32>,  
    @location(5) thickness: f32,
    @location(6) flags: u32,
  
    @location(7) vertex_0: vec2<f32>,
    @location(8) vertex_1: vec2<f32>,
    @location(9) vertex_2: vec2<f32>,
    @location(10) roundness: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) thickness: f32,

    @location(3) vertex_0: vec2<f32>,
    @location(4) vertex_1: vec2<f32>,
    @location(5) vertex_2: vec2<f32>,
    @location(6) roundness: f32,
#ifdef TEXTURED
    @location(7) texture_uv: vec2<f32>,
#endif
};

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // Vertex positions for a basic quad
    let vertex = core::get_quad_vertex(v.index);

    // Reconstruct our transformation matrix
    let matrix = mat4x4<f32>(
        v.matrix_0,
        v.matrix_1,
        v.matrix_2,
        v.matrix_3
    );

    // Scale so triangle is completely in clipping range
    var scale = max(
        max(
            max(abs(v.vertex_0.x), abs(v.vertex_0.y)),
            max(abs(v.vertex_1.x), abs(v.vertex_1.y)),
        ),
        max(abs(v.vertex_2.x), abs(v.vertex_2.y)),
    );

    // Calculate vertex data shared between most shapes
    var vertex_data = core::get_vertex_data(matrix, vertex.xy * scale, v.thickness, v.flags);

    out.clip_position = vertex_data.clip_pos;
    out.uv = vertex_data.local_pos / (scale * vertex_data.scale) * vertex_data.uv_ratio;
    out.thickness = core::calculate_thickness(vertex_data.thickness_data, scale, v.flags);
    out.roundness = min(v.roundness / scale, 1.0);

    out.vertex_0 = (1. - 2. * out.roundness) * v.vertex_0 / scale;
    out.vertex_1 = (1. - 2. * out.roundness) * v.vertex_1 / scale;
    out.vertex_2 = (1. - 2. * out.roundness) * v.vertex_2 / scale;

    out.color = v.color;
#ifdef TEXTURED
    out.texture_uv = core::get_texture_uv(vertex.xy);
#endif
    return out;
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) thickness: f32,

    @location(3) vertex_0: vec2<f32>,
    @location(4) vertex_1: vec2<f32>,
    @location(5) vertex_2: vec2<f32>,
    @location(6) roundness: f32,
#ifdef TEXTURED
    @location(7) texture_uv: vec2<f32>,
#endif
};

fn cross2d(a: vec2<f32>, b: vec2<f32>) -> f32 {
    // For two vertices A, B
    // The cross product (pos - A) x (B - A) is equivalent to
    // ||pos - A|| * ||B - A|| * sin(theta)
    // with theta being the inscribed angle between the edges (A,pos) and (A,B).
    // sin(theta) is the signed distance of pos to the edge (A,B)
    // See: https://en.wikipedia.org/wiki/Cross_product
    return (a.x * b.y) - (a.y * b.x);
}

fn triangleSDF(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> f32 {
    // Heavily inspired by https://iquilezles.org/articles/distfunctions2d/

    var ab = b - a; var bc = c - b; var ca = a - c;
    var ap = p - a; var bp = p - b; var cp = p - c;

    // pos projected to the edges and clipped to stay inside the triangle.
    // One of these is the closest point on the triangle boundary
    var pq_ab = ap - ab * clamp(dot(ap, ab) / dot(ab, ab), 0.0, 1.0);
    var pq_bc = bp - bc * clamp(dot(bp, bc) / dot(bc, bc), 0.0, 1.0);
    var pq_ca = cp - ca * clamp(dot(cp, ca) / dot(ca, ca), 0.0, 1.0);

    // which way around is our triangle?
    var s = sign(cross2d(ab, ca));

    // These are not actual 2d points but rather pairs of
    // a) squared distance to the nearest pq_* point
    // and b) 2d cross product to tell us whether we're inside or outside the triangle
    var d_ab = vec2<f32>(dot(pq_ab, pq_ab), s*cross2d(ap, ab));
    var d_bc = vec2<f32>(dot(pq_bc, pq_bc), s*cross2d(bp, bc));
    var d_ca = vec2<f32>(dot(pq_ca, pq_ca), s*cross2d(cp, ca));

    var d = min(min(d_ab, d_bc), d_ca);

    return -sqrt(d.x) * sign(d.y);
}

// Due to https://github.com/gfx-rs/naga/issues/1743 this cannot be compiled into the vertex shader on web
#ifdef FRAGMENT
@fragment
fn fragment(f: FragmentInput) -> @location(0) vec4<f32> {
    // Mask representing whether this fragment falls within the shape
    var in_shape = f.color.a;

    // Calculate our positions distance from the polygon
    var dist = triangleSDF(f.uv, f.vertex_0, f.vertex_1, f.vertex_2) - f.roundness;

    // Cut off points outside the shape or within the hollow area
    in_shape *= core::step_aa(-f.thickness, dist) * core::step_aa(dist, 0.);

    var color = core::color_output(vec4<f32>(f.color.rgb, in_shape));
#ifdef TEXTURED
    color = color * textureSample(image, image_sampler, f.texture_uv);
#endif

    // Discard fragments no longer in the shape
    if in_shape < 0.0001 {
        discard;
    }

    return color;
}
#endif
