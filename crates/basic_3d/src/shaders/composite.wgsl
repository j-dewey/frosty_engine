// Vertex shader

struct MeshVertex {
    @location(0) screen_pos: vec2<f32>,
    @location(1) tex_coords: vec2<f32>
}

@vertex
fn vs_main(
    model: MeshVertex,
) -> MeshFragment {
    var out: MeshFragment;

    out.clip_position = vec4<f32>(model.screen_pos, 0.0, 1.0);
    out.tex_coords = model.tex_coords;

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
}

@fragment
fn fs_main(in: MeshFragment) -> @location(0) vec4<f32> {
    return textureSample(
        texture,
        sample,
        in.tex_coords,
    );
}
