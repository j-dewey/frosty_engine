// Loading .obj files

use render::{
    mesh::{IndexArray, Mesh},
    vertex::MeshVertex,
};
use tobj::Model;

use crate::Entity;

// Load a mesh from model data taken tobj
// Assumes that no quads are in play
// Each part that has a different material is listed as a seperate model,
// so an array of Model's refers to one Mesh
fn load_mesh_from_model(sections: &[Model]) -> Mesh<MeshVertex> {
    let mut verts: Vec<MeshVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut index_offset = 0;

    for sec in sections {
        let mesh = &sec.mesh;
        let new_verts: Vec<MeshVertex> = mesh
            .positions
            .iter()
            .array_chunks::<3>()
            .zip(mesh.texcoords.iter().array_chunks::<2>())
            .zip(mesh.normals.iter().array_chunks::<3>())
            .map(|((world_pos, tex_coords), normal)| MeshVertex {
                world_pos: [*world_pos[0], *world_pos[1], *world_pos[2]],
                tex_coords: [*tex_coords[0], *tex_coords[1]],
                mat: mesh.material_id.unwrap() as u32,
                normal: [*normal[0], *normal[1], *normal[2]],
            })
            .collect();
        let num_new_verts = new_verts.len();

        verts.extend(new_verts);
        indices.extend(mesh.indices.iter().map(|i| i + index_offset));
        index_offset += num_new_verts as u32;
    }

    Mesh {
        verts,
        indices: IndexArray::new_u32(&indices[..]),
    }
}

pub fn read_mesh_from_file(mesh_path: &str) -> (Mesh<MeshVertex>, Vec<String>) {
    let (sections, materials) = tobj::load_obj(mesh_path, &tobj::GPU_LOAD_OPTIONS)
        .expect("Failed to locate and load obj file");

    let mesh = load_mesh_from_model(&sections[..]);
    let mats = materials
        .expect("Failed to load materials from obj file")
        .iter()
        .map(|m| m.diffuse_texture.as_ref().unwrap().clone())
        .collect();

    (mesh, mats)
}

// Load a .obj file as a mesh and textures
pub fn spawn_mesh_from_obj(path: &str, offset: [f32; 3]) -> Entity {
    let (mut mesh, textures) = read_mesh_from_file(path);
    mesh.verts.iter_mut().for_each(|v| {
        v.world_pos[0] += offset[0];
        v.world_pos[1] += offset[1];
        v.world_pos[2] += offset[2];
    });
    Entity::new().chain_push_obj(mesh)
}
