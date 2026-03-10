use crate::{
    scheduled_pipeline::{ShaderLabel, DEFAULT_MATERIAL_LABEL, PRESCREEN_RENDER_TARGET_TEXTURES},
    vertex::{ScreenQuadVertex, Vertex},
    window_state::WindowState,
    QUAD_INDEX_ORDER,
};
use frosty_alloc::FrostyAllocatable;

// Meshes live in two places:
//      1) The GPU
//      2) The CPU
// This creates a need for dual representations of a mesh.
// A general [ Mesh<V: Vertex> ] can store the CPU data for manipulation
// while another [ MeshData ] can store the GPU data. While intrinsically
// related, they must be seperate as they have different purposes. It is
// then up to the programmer to ensure that any changes made on the CPU
// representation are reflected in the GPU representation

//
//      CPU side
//

// A way to generalize u32 and u16
trait VertexIndex {}
impl VertexIndex for u32 {}
impl VertexIndex for u16 {}

#[derive(Clone, Debug)]
pub struct IndexArray {
    format: wgpu::IndexFormat,
    len: usize,
    // (     u32    )
    // ( [u16, u16] )
    // have similar enough representation for this to work
    data: Vec<u32>,
}

impl IndexArray {
    pub fn new_u16(indices: &[u16]) -> Self {
        let len = indices.len();
        let data = indices
            .chunks(2)
            .map(|inds| ((inds[0] as u32) << 16) + *inds.get(1).unwrap_or(&0) as u32)
            .collect();
        Self {
            format: wgpu::IndexFormat::Uint16,
            len,
            data,
        }
    }

    pub fn new_u32(indices: &[u32]) -> Self {
        let len = indices.len();
        let mut data = Vec::new();
        data.extend(indices);
        Self {
            format: wgpu::IndexFormat::Uint32,
            len,
            data,
        }
    }

    pub fn get_format(&self) -> wgpu::IndexFormat {
        self.format
    }

    pub fn get_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data[..])
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter<'a>(&'a self) -> IndexArrayIter<'a> {
        IndexArrayIter {
            arr: self,
            counter: 0,
        }
    }

    pub fn map<F: Fn(u32) -> u32>(&mut self, func: F) {
        for i in 0..self.data.len() {
            self.data[i] = func(self.data[i]);
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.len += other.data.len();
        self.data.extend(other.data);
    }
}

#[derive(Debug)]
pub struct IndexArrayIter<'a> {
    arr: &'a IndexArray,
    counter: usize,
}

// NOTE: This is currently only implemented for the U32 variant
impl<'a> Iterator for IndexArrayIter<'a> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter <= self.arr.len {
            return None;
        }
        self.counter += 1;
        Some(self.arr.data[self.counter - 1])
    }
}

// This is to allow for custom and more complex mesh objects
pub trait MeshyObject {
    // gets the vertex data is dissolved into bytes
    fn get_verts(&self) -> &[u8];
    // gets the index data dissolved into bytes
    fn get_indices(&self) -> (&[u8], usize);
}

// This is a general form that will work for most mesh cases
#[derive(Clone, Debug)]
pub struct Mesh<V: Vertex> {
    pub verts: Vec<V>,
    pub indices: IndexArray,
    // this can correspond to any type of BindGroup
    // for most purposes it should be either a Texture or a TextureArray
    pub material: ShaderLabel,
}

impl<V: Vertex> Mesh<V> {
    // Dirty is set to false in constructors since
    pub fn new_u16(verts: Vec<V>, indices: Vec<u16>) -> Self {
        Self {
            verts,
            indices: IndexArray::new_u16(&indices[..]),
            material: DEFAULT_MATERIAL_LABEL,
        }
    }

    pub fn new_u32(verts: Vec<V>, indices: Vec<u32>) -> Self {
        Self {
            verts,
            indices: IndexArray::new_u32(&indices[..]),
            material: DEFAULT_MATERIAL_LABEL,
        }
    }

    pub fn with_material(mut self, mat_label: ShaderLabel) -> Self {
        self.material = mat_label;
        self
    }

    // update the normal value stored in each vertex
    pub fn calc_norms(&mut self) {
        let chunks = self.indices.iter().array_chunks::<3>();
        chunks.for_each(|[u1, u2, u3]| {
            let v1 = self.verts[u1 as usize];
            let v2 = self.verts[u2 as usize];
            let v3 = self.verts[u3 as usize];
            let norm = (v1.pos() - v2.pos()).cross(v1.pos() - v3.pos());
            // in shaders, the normal value is decided by the first vertex
            self.verts[u1 as usize].set_normal(norm);
        });
    }

    // merge this mesh with another
    pub fn merge(&mut self, mut other: Self) {
        let cur_verts = self.verts.len();
        other.indices.map(|i| i + cur_verts as u32);
        self.verts.extend(other.verts);
        self.indices.extend(other.indices);
    }
}

impl Mesh<ScreenQuadVertex> {
    pub fn new_screen_quad_u32() -> Self {
        let verts = vec![
            ScreenQuadVertex {
                clip_pos: [-1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            ScreenQuadVertex {
                clip_pos: [1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            ScreenQuadVertex {
                clip_pos: [-1.0, -1.0],
                tex_coords: [0.0, 1.0],
            },
            ScreenQuadVertex {
                clip_pos: [1.0, -1.0],
                tex_coords: [1.0, 1.0],
            },
        ];

        let indices = IndexArray::new_u32(&QUAD_INDEX_ORDER[..]);

        Self {
            verts,
            indices,
            material: PRESCREEN_RENDER_TARGET_TEXTURES,
        }
    }
}

unsafe impl<V: Vertex> FrostyAllocatable for Mesh<V> where V: FrostyAllocatable {}

impl<V: Vertex> MeshyObject for Mesh<V> {
    fn get_verts(&self) -> &[u8] {
        bytemuck::cast_slice(&self.verts[..])
    }

    fn get_indices(&self) -> (&[u8], usize) {
        (self.indices.get_bytes(), self.indices.len)
    }
}

//
//      GPU Side
//

// Collection of handles to the mesh data stored on the GPU
pub struct MeshData {
    pub v_buf: wgpu::Buffer,
    pub i_buf: wgpu::Buffer,
    pub num_indices: u32,
    pub unique_bind_groups: Vec<ShaderLabel>,
}

impl MeshData {
    // Create a mesh that doesn't actually store any data
    // This is helpful for creating a ScheduledPipeline
    // without having the mesh data loaded
    pub fn blank(
        label: &str,
        v_len: usize,
        i_len: usize,
        unique_bind_groups: Vec<ShaderLabel>,
        ws: &WindowState,
    ) -> Self {
        let dummy_data = vec![0; v_len.max(i_len)];
        let v_buf = ws.load_vertex_buffer(label, &dummy_data[..v_len]);
        let i_buf = ws.load_index_buffer(label, &dummy_data[..i_len]);
        Self {
            v_buf,
            i_buf,
            num_indices: 0,
            unique_bind_groups,
        }
    }
}
