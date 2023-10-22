#define_import_path bevy_vector_shapes::core

struct ColorGrading {
    exposure: f32,
    gamma: f32,
    pre_saturation: f32,
    post_saturation: f32,
}

struct View {
    view_proj: mat4x4<f32>,
    unjittered_view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    world_position: vec3<f32>,
    // viewport(x_origin, y_origin, width, height)
    viewport: vec4<f32>,
    color_grading: ColorGrading,
    mip_bias: f32,
};

@group(0) @binding(0)
var<uniform> view: View;

#ifdef TEXTURED
#ifdef FRAGMENT

@group(2) @binding(0)
var image: texture_2d<f32>;

@group(2) @binding(1)
var image_sampler: sampler;

#endif
#endif

// Calculate pixels per world unit from a given position and up vector
fn pixels_per_unit(pos: vec3<f32>, dir: vec3<f32>) -> f32 {
    var vp = transpose(view.view_proj);
    var mag = dot(vp[3], vec4<f32>(pos, 1.));
    var clip = vec2<f32>(
        dot(vp[0].xyz, dir) / mag,
        dot(vp[1].xyz, dir) / mag
    );
    return length(clip * view.viewport.zw) / 2.;
}

// Convert thickness from given type to pixels
fn get_thickness_pixels(thickness: f32, thickness_type: u32, pixels_per_u: f32) -> f32 {
    switch thickness_type {
        default: { // WORLD
            return thickness * pixels_per_u;
        }
        case 1u: { // PIXELS
            return thickness;
        }
        case 2u: { // SCREEN
            return min(view.viewport.z, view.viewport.w) * (thickness / 100.);
        }
    }
}

struct ThicknessData {
    // Thickness in pixels
    thickness_p: f32,
    // Pixels per world unit
    pixels_per_u: f32,
};

// Calculate thickness data at a given position with a given up vector
fn get_thickness_data(thickness: f32, thickness_type: u32, pos: vec3<f32>, dir: vec3<f32>) -> ThicknessData {
    var out: ThicknessData;
    out.pixels_per_u = pixels_per_unit(pos, dir);
    out.thickness_p = get_thickness_pixels(thickness, thickness_type, out.pixels_per_u);
    return out;
}

// Determine thickness of a shape depending on thickness_data and whether it's hollow
fn calculate_thickness(thickness_data: ThicknessData, uv_scale: f32, flags: u32) -> f32 {
    var hollow = f_hollow(flags);
    if hollow > 0u {
        // Convert from thickness in pixels to uv space, this requires the same scaling factor as size
        return thickness_data.thickness_p / thickness_data.pixels_per_u / uv_scale;
    } else {
        return 1.0;
    }
}

fn p_to_camera_dir(p: vec3<f32>) -> vec3<f32> {
#ifdef PIPELINE_2D
    return transpose(view.inverse_view)[2].xyz;
#endif

#ifdef PIPELINE_3D
    return normalize(view.world_position - p);
#endif
}

// Rotate the given 2d vector on the z axis by the given angle
fn rotate_vec_a(v: vec2<f32>, a: f32) -> vec2<f32> {
    var point = vec2<f32>(cos(a), sin(a));
    return vec2<f32>(
        point.x * v.x - point.y * v.y,
        point.y * v.x + point.x * v.y
    );
}

// Functions to extract info from flags, format should match the following field taken from render/mod.rs
// bitfield! {
//     pub struct Flags(u32);
//     pub u32, from into ThicknessType, _, set_thickness_type: 1, 0;
//     pub u32, from into Alignment, _, set_alignment: 2, 2;
//     pub u32, _, set_hollow: 3, 3;
//     pub u32, from into Cap, _, set_cap: 5, 4;
//     pub u32, _, set_arc: 6, 6;
// }

fn f_thickness_type(flags: u32) -> u32 {
    return flags & 3u;
}

fn f_alignment(flags: u32) -> u32 {
    return (flags >> 2u) & 1u;
}

fn f_hollow(flags: u32) -> u32 {
    return (flags >> 3u) & 1u;
}

fn f_cap(flags: u32) -> u32 {
    return (flags >> 4u) & 3u;
}

fn f_arc(flags: u32) -> u32 {
    return (flags >> 6u) & 1u;
}

#ifdef LOCAL_AA
const AA_PADDING: f32 = 2.0;

// Due to https://github.com/gfx-rs/naga/issues/1743 this cannot be compiled into the vertex shader on web
#ifdef FRAGMENT
fn partial_derivative(v: f32) -> f32 {
    var dv = vec2<f32>(dpdx(v), dpdy(v));
    return length(dv);
}

// Apply local anti aliasing based on the partial derivative of x and y per pixel
// This is imperfect and is open to improvement 
fn step_aa(edge: f32, x: f32) -> f32 {
    var value = x - edge;
    var pd = partial_derivative(value);
    return 1.0 - saturate(-value / pd);
}

fn step_aa_pd(edge: f32, x: f32, in: f32) -> f32 {
    var value = x - edge;
    var pd = partial_derivative(in);
    return 1.0 - saturate(-value / pd);
}
#endif
#endif

#ifdef DISABLE_LOCAL_AA
const AA_PADDING: f32 = 0.0;

fn step_aa(edge: f32, x: f32) -> f32 {
    return step(edge, x);
}

fn step_aa_pd(edge: f32, x: f32, pd: f32) -> f32 {
    return step(edge, x);
}
#endif

// Calculate xy scale by taking it directly from the length of the basis vectors in the matrix
fn get_scale(matrix: mat4x4<f32>) -> vec2<f32> {
    return vec2<f32>(length(matrix[0].xyz), length(matrix[1].xyz));
}

// Take the y basis directly from the matrix and pass along to get_basis_vectors_from_up
fn get_basis_vectors(matrix: mat4x4<f32>, origin: vec3<f32>, flags: u32) -> mat3x3<f32> {
    return get_basis_vectors_from_up(matrix, origin, normalize(matrix[1].xyz), f_alignment(flags));
}
// Calculate each of the basis vectors for our shape
// Z-basis is either taken from the mesh or from the direction to the camera depending on alignment
fn get_basis_vectors_from_up(matrix: mat4x4<f32>, origin: vec3<f32>, up: vec3<f32>, alignment: u32) -> mat3x3<f32> {
    // The z basis depends on our configured alignment, when rendering flat rotate the 
    // vector the same way as the y basis, otherwise take the direction to the camera
    var z_basis: vec3<f32>;
    var y_basis = up;
    switch alignment {
        // Alignment::Flat
        default: {
            z_basis = normalize(matrix[2].xyz);
        }
        // Alignment::Billboard
        case 1u: {
            y_basis = normalize((view.view * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz);
            z_basis = p_to_camera_dir(origin);
        }
        // Alignment::Billboard for lines
        case 2u: {
            z_basis = p_to_camera_dir(origin);
        }
    }

    // The x basis is then calculated as the cross product of the y and z basis
    var x_basis = normalize(cross(y_basis, z_basis));

    // Now that we have our accurate x basis and z basis we must correct our y basis
    // simply calculate it the same way we did the x basis
    y_basis = cross(x_basis, z_basis);

    return mat3x3<f32>(
        x_basis,
        y_basis,
        z_basis
    );
}

struct VertexData {
    thickness_data: ThicknessData,
    clip_pos: vec4<f32>,
    local_pos: vec2<f32>,
    uv_ratio: vec2<f32>,
    scale: vec2<f32>
};

// Calculate the full set of vertex data shared betwen each shape type
fn get_vertex_data(matrix: mat4x4<f32>, vertex: vec2<f32>, thickness: f32, flags: u32) -> VertexData {
    var out: VertexData;

    // Transform the origin into world space
    var origin = (matrix * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    var basis_vectors = get_basis_vectors(matrix, origin, flags);

    // Get thickness data at our origin given our up vector
    var thickness_type = f_thickness_type(flags);
    out.thickness_data = get_thickness_data(thickness, thickness_type, origin, basis_vectors[1]);

    // Calculate the local position of our vertex by scaling it
    out.scale = get_scale(matrix);
    out.local_pos = vertex.xy * out.scale;

    // Convert our padding into world space and match direction of our vertex
    var aa_padding_u = AA_PADDING / out.thickness_data.pixels_per_u;
    var aa_padding = sign(vertex.xy) * aa_padding_u;

    // Pad our position and determine the ratio by which to scale uv such that uvs ignore padding
    var padded_pos = out.local_pos + aa_padding;
    out.uv_ratio = padded_pos / out.local_pos;

    // Rotate the position based on our basis vectors and add the world position offset
    var world_pos = origin + (padded_pos.x * basis_vectors[0]) + (padded_pos.y * basis_vectors[1]);

    // Transform to clip space
    out.clip_pos = view.view_proj * vec4<f32>(world_pos, 1.0);
    return out;
}

fn get_texture_uv(vertex: vec2<f32>) -> vec2<f32> {
    return (vertex + 1.0) / 2.0;
}

#ifdef FRAGMENT
// Transform our color output to respect the alpha mode set for our shape and combine with our texture if any
fn color_output(in: vec4<f32>) -> vec4<f32> {
#ifdef BLEND_MULTIPLY
    var color = vec4<f32>(in.rgb * in.a, in.a);
#endif
#ifdef BLEND_ADD
    var color = vec4<f32>(in.rgb * in.a, 0.0);
#endif
#ifdef BLEND_ALPHA
    var color = in;
#endif

    return color;
}
#endif