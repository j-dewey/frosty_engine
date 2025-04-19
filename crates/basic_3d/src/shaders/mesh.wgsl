// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct MeshVertex {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) material: u32,
    @location(3) normal: vec3<f32>
}

@vertex
fn vs_main(
    model: MeshVertex,
) -> MeshFragment {
    var out: MeshFragment;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.material = model.material;
    out.normal = model.normal;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var sample: sampler;

struct MeshFragment {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(2) material: u32,
    @location(3) normal: vec3<f32>
}

@fragment
fn fs_main(in: MeshFragment) -> @location(0) vec4<f32> {
    // the first vertex of a triangle sets the texture sampled
    // for the other vertices of the triangle
    return textureSample(
        texture,
        sample,
        in.tex_coords,
    );
    //return vec4(in.normal, 1.0);
}
