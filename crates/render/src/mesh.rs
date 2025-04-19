use crate::vertex::Vertex;
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
}

// This is to allow for custom and more complex mesh objects
pub trait MeshyObject {
    // gets the vertex data is dissolved into bytes
    fn get_verts(&self) -> &[u8];
    // gets the index data dissolved into bytes
    fn get_indices(&self) -> (&[u8], usize);
}

// This is a general form that will work for most mesh cases
pub struct Mesh<V: Vertex> {
    pub verts: Vec<V>,
    pub indices: IndexArray,
}

impl<V: Vertex> Mesh<V> {
    // Dirty is set to false in constructors since
    pub fn new_u16(verts: Vec<V>, indices: Vec<u16>) -> Self {
        Self {
            verts,
            indices: IndexArray::new_u16(&indices[..]),
        }
    }
    pub fn new_u32(verts: Vec<V>, indices: Vec<u32>) -> Self {
        Self {
            verts,
            indices: IndexArray::new_u32(&indices[..]),
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
    pub texture_index: usize,
}
