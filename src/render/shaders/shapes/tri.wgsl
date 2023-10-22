#import bevy_vector_shapes::core as core
#import bevy_vector_shapes::core view, image, image_sampler
#import bevy_vector_shapes::constants PI, TAU

struct Vertex {
    @builtin(instance_index) index: u32,
    @builtin(vertex_index) vertex_index: u32,
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
  
    @location(7) v_0: vec2<f32>,
    @location(8) v_1: vec2<f32>,
    @location(9) v_2: vec2<f32>,
    @location(10) roundness: f32,
};

#ifdef PER_OBJECT_BUFFER_BATCH_SIZE
@group(1) @binding(0) var<uniform> shapes: array<Shape, #{PER_OBJECT_BUFFER_BATCH_SIZE}u>;
#else
@group(1) @binding(0) var<storage> shapes: array<Shape>;
#endif 


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) thickness: f32,

    @location(3) v_0: vec2<f32>,
    @location(4) v_1: vec2<f32>,
    @location(5) v_2: vec2<f32>,
    @location(6) roundness: f32,
#ifdef TEXTURED
    @location(7) texture_uv: vec2<f32>,
#endif
};

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // Vertex positions for a basic quad
    let shape = shapes[v.index];
    var vertex: vec2<f32>;
    switch v.vertex_index {
        default: {
            vertex = shape.v_0;
        }
        case 1u: {
            vertex = shape.v_1;
        }
        case 2u: {
            vertex = shape.v_2;
        }
    }

    // Reconstruct our transformation matrix
    let matrix = mat4x4<f32>(
        shape.matrix_0,
        shape.matrix_1,
        shape.matrix_2,
        shape.matrix_3
    );

    let l_s_0 = length(shape.v_1 - shape.v_2);
    let l_s_1 = length(shape.v_2 - shape.v_0);
    let l_s_2 = length(shape.v_0 - shape.v_1);

    let p = l_s_0 + l_s_1 + l_s_2;
    let center = (l_s_0 * shape.v_0 + l_s_1 * shape.v_1 + l_s_2 * shape.v_2) / p; 
    let s = p / 2.0;
    let in_radius = sqrt((s - l_s_0) * (s - l_s_1) * (s - l_s_2) / s);

    vertex = vertex - center;
    let v_0 = shape.v_0 - center;
    let v_1 = shape.v_1 - center;
    let v_2 = shape.v_2 - center;

    let l_v_0 = length(v_0);
    let l_v_1 = length(v_1);
    let l_v_2 = length(v_2);

    let max_dist = max(
        max(l_v_0, l_v_1),
        l_v_2
    );

    let min_dist = min(
        min(l_v_0, l_v_1),
        l_v_2
    );

    // Transform the origin into world space
    let scale = core::get_scale(matrix);
    var origin = (matrix * vec4<f32>(scale * center.xy, 0.0, 1.0)).xyz;
    var basis_vectors = core::get_basis_vectors(matrix, origin, shape.flags);

    // Get thickness data at our origin given our up vector
    var thickness_type = core::f_thickness_type(shape.flags);
    let thickness_data = core::get_thickness_data(shape.thickness, thickness_type, origin, basis_vectors[1]);

    // Calculate the local position of our vertex by scaling it
    let local_pos = vertex.xy * scale;

    // Convert our padding into world space and match direction of our vertex
    var aa_padding_u = core::AA_PADDING / thickness_data.pixels_per_u;
    let uv_ratio = (in_radius + aa_padding_u) / in_radius;

    // Pad our position and determine the ratio by which to scale uv such that uvs ignore padding
    var padded_pos = local_pos * uv_ratio;

    // Rotate the position based on our basis vectors and add the world position offset
    var world_pos = origin + (padded_pos.x * basis_vectors[0]) - (padded_pos.y * basis_vectors[1]);
    out.clip_position = view.view_proj * vec4<f32>(world_pos, 1.0);

    out.uv = vertex.xy * uv_ratio / min_dist;
    out.thickness = core::calculate_thickness(thickness_data, min_dist, shape.flags);
    out.roundness = min(shape.roundness / min_dist, 1.0);

    out.v_0 = (v_0 / min_dist) * ((min_dist - 2.0 * shape.roundness) / min_dist);
    out.v_1 = (v_1 / min_dist) * ((min_dist - 2.0 * shape.roundness) / min_dist) ;
    out.v_2 = (v_2 / min_dist) * ((min_dist - 2.0 * shape.roundness) / min_dist) ;

    out.color = shape.color;
#ifdef TEXTURED
    out.texture_uv = core::get_texture_uv(vertex.xy);
#endif
    return out;
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) thickness: f32,

    @location(3) v_0: vec2<f32>,
    @location(4) v_1: vec2<f32>,
    @location(5) v_2: vec2<f32>,
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
    var dist = triangleSDF(f.uv, f.v_0, f.v_1, f.v_2) - f.roundness;

    // Cut off points outside the shape or within the hollow area
    in_shape *= core::step_aa(-f.thickness, dist) * core::step_aa(dist, 0.);

    var color = core::color_output(vec4<f32>(f.color.rgb, in_shape));
#ifdef TEXTURED
    color = color * textureSample(image, image_sampler, f.texture_uv);
#endif

    // Discard fragments no longer in the shape
    if in_shape < 0.0001 {
        //discard;
    }

    return color;
}
#endif
