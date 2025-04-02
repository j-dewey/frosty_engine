use cgmath::{Point3, Vector3};
use frosty_alloc::{AllocId, FrostyAllocatable};
use render::{
    mesh::MeshyObject, vertex::Vertex, wgpu, window_state::WindowState, QUAD_INDEX_ORDER,
};

#[derive(Copy, Clone, Debug)]
pub(crate) struct SnowVertex {
    pos: [f32; 3],
    tex_coords: [f32; 2],
    depth: u32,
}
unsafe impl bytemuck::Pod for SnowVertex {}
unsafe impl bytemuck::Zeroable for SnowVertex {}

impl Vertex for SnowVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SnowVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // world pos
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // for @location(n) when defining struct in shader
                    format: wgpu::VertexFormat::Float32x3,
                },
                // texture coords
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // height in shell texture
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

impl SnowVertex {
    fn new(pos: [f32; 3], tex_coords: [f32; 2], depth: u32) -> Self {
        Self {
            pos,
            tex_coords,
            depth,
        }
    }
}

pub(crate) struct SnowMesh {
    verts: Vec<SnowVertex>,
    indices: Vec<u32>,
    v_buf: wgpu::Buffer,
    i_buf: wgpu::Buffer,
    layers: u32,
}

impl SnowMesh {
    // This creates snow from square  quads, but can easily be altered to
    // make any shape. Seed center describes the center of the lowest plane
    pub fn new(
        width: f32,
        layers: u32,
        shell_gap: f32,
        seed_center: Point3<f32>,
        ws: &WindowState,
    ) -> Self {
        let mut verts: Vec<SnowVertex> = Vec::new();
        let forward = Vector3::new(0.0, 0.0, width / 2.0);
        let right = Vector3::new(width / 2.0, 0.0, 0.0);
        let up = Vector3::new(0.0, shell_gap, 0.0);

        for i in 0..layers {
            let plane_center = seed_center + up * i as f32;
            let fl_pos: [f32; 3] = (plane_center - right + forward).into();
            let fr_pos: [f32; 3] = (plane_center + right + forward).into();
            let bl_pos: [f32; 3] = (plane_center - right - forward).into();
            let br_pos: [f32; 3] = (plane_center + right - forward).into();

            let fl_vert = SnowVertex::new(fl_pos, [0.0, 0.0], i);
            let fr_vert = SnowVertex::new(fr_pos, [1.0, 0.0], i);
            let bl_vert = SnowVertex::new(bl_pos, [0.0, 1.0], i);
            let br_vert = SnowVertex::new(br_pos, [1.0, 1.0], i);

            verts.extend(&[fl_vert, fr_vert, bl_vert, br_vert]);
        }

        let indices: Vec<u32> = QUAD_INDEX_ORDER
            .iter()
            .cycle()
            .take((layers * 6) as usize)
            .enumerate()
            .map(|(i, val)| val + (4 * i / 6) as u32)
            .collect();

        let v_buf = ws.load_vertex_buffer("Snow Vertices", bytemuck::cast_slice(&verts[..]));
        let i_buf = ws.load_index_buffer("Snow Indices", bytemuck::cast_slice(&indices[..]));

        Self {
            verts,
            indices,
            v_buf,
            i_buf,
            layers,
        }
    }
}

unsafe impl FrostyAllocatable for SnowMesh {
    fn id() -> frosty_alloc::AllocId
    where
        Self: Sized,
    {
        AllocId::new(100000)
    }
}

impl MeshyObject for SnowMesh {
    fn get_verts(&self) -> &[u8] {
        bytemuck::cast_slice(&self.verts[..])
    }

    fn get_indices(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices[..])
    }
}
