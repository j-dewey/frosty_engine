// This is a less coupled pipeline which requires manually inserting
// bind groups and buffers. Scheduled refers to how the pipeline isn't
// closed to just the Allocator

use hashbrown::HashMap;
use wgpu::SurfaceTexture;

use crate::{
    mesh::MeshData,
    shader::{BindGroupCollection, Shader, ShaderDefinition},
    texture::Texture,
    uniform::Uniform,
    wgpu,
    window_state::{GPUBindings, WindowState},
};

mod binding;
pub use binding::*;
mod buffer;
pub use buffer::*;
mod name;
pub use name::*;

pub struct ScheduledShaderNodeDescription<'a> {
    pub bind_groups: Vec<ShaderLabel>,
    pub buffer_group: ShaderLabel,
    pub targets: Option<Vec<ShaderLabel>>,
    pub depth: Option<ShaderLabel>,
    pub shader: ShaderDefinition<'a>,
}

// Details about how an ScheduledPipeline should be run.
// i.e. what shaders should be used, how larges caches should be, etc.
pub struct ScheduledPipelineDescription<'a> {
    pub shader_nodes: Vec<ScheduledShaderNodeDescription<'a>>,
    pub buffers: Vec<(ShaderLabel, Vec<MeshData>)>,
    pub bind_groups: Vec<ScheduledBindGroup<'a>>,
    pub textures: Vec<ScheduledTexture<'a>>,
}

impl ScheduledPipelineDescription<'_> {
    pub fn finalize(mut self, gpu: &GPUBindings) -> ScheduledPipeline {
        let mut mesh_groups = Vec::new();
        let mut uniform_cache = Vec::new();
        let mut texture_cache = Vec::new();
        let mut name_to_buffer = HashMap::new();
        let mut name_to_uniform = HashMap::new();
        let mut name_to_texture = HashMap::new();

        self.buffers.drain(..).for_each(|(name, buffers)| {
            name_to_buffer.insert(name, mesh_groups.len());
            mesh_groups.push(buffers);
        });

        self.bind_groups.drain(..).for_each(|data| {
            let name = data.label;

            match data.form {
                ScheduledBindGroupType::ReadOnlyTexture(data) => {
                    // create texture
                    let texture = data.to_texture(gpu);

                    // add to array
                    name_to_uniform.insert(name, BindGroupIndex::Texture(texture_cache.len()));
                    texture_cache.push(texture);
                }
                ScheduledBindGroupType::ReadOnlyTextureArray(textures) => {
                    let bind_group = ScheduledBindGroupType::ReadOnlyTextureArray(textures)
                        .to_bind_group(name.clone(), gpu);
                    name_to_uniform
                        .insert(name.clone(), BindGroupIndex::Uniform(uniform_cache.len()));
                    uniform_cache.push(Uniform {
                        buffers: vec![],
                        bind_group,
                    });
                }
                ScheduledBindGroupType::Uniform(data) => {
                    let (buffers, bind_group) = data.get_bind_group(name.clone(), gpu);
                    name_to_uniform
                        .insert(name.clone(), BindGroupIndex::Uniform(uniform_cache.len()));
                    uniform_cache.push(Uniform {
                        buffers,
                        bind_group,
                    });
                }
            }
        });

        self.textures.drain(..).for_each(|texture| {
            let name = texture.get_label().clone();
            let final_texture = match texture {
                ScheduledTexture::Unloaded {
                    label,
                    desc,
                    sample_desc,
                    view_desc,
                    bg_layout_desc,
                    data,
                    size,
                } => {
                    let text = Texture::from_descs(
                        &label.label(),
                        &desc,
                        &sample_desc,
                        &view_desc,
                        &bg_layout_desc,
                        size,
                        &gpu.device,
                    );
                    if let Some(data) = data {
                        gpu.queue.write_texture(
                            // Tells wgpu where to copy the pixel data
                            wgpu::TexelCopyTextureInfo {
                                texture: &text.data,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            // The actual pixel data
                            &data[..],
                            // The layout of the texture
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(4 * desc.size.width),
                                rows_per_image: Some(desc.size.height),
                            },
                            desc.size,
                        );
                    }
                    text
                }
                ScheduledTexture::Loaded { texture, .. } => texture,
            };
            name_to_texture.insert(name, texture_cache.len());
            texture_cache.push(final_texture);
        });

        let shaders = self
            .shader_nodes
            .drain(..)
            .map(|node| ScheduledShaderNode {
                bind_groups: node
                    .bind_groups
                    .iter()
                    .map(|name| {
                        name_to_uniform.get(name).map(|i| *i)
                            .or_else(|| name_to_texture.get(name).map(|i| BindGroupIndex::Texture(*i)))
                            .expect(
                                &format!("Shader references a bind group not passed into pipeline description: {:?}", name.label())
                            )
                    })
                    .collect(),
                buffer_group: *name_to_buffer
                    .get(&node.buffer_group)
                    .expect("Shader references a buffer not passed into pipeline description"),
                targets: if let Some(names) = node.targets {
                    Some(
                        names
                            .iter()
                            .map(|name| *name_to_texture
                                .get(name)
                                .expect("Shader references a view texture not passed into pipeline description",)
                            ).collect()
                    )
                } else {
                    None
                },
                depth: if let Some(name) = node.depth {
                    Some(*name_to_texture.get(&name).expect(
                        "Shader references a depth texture not passed into pipeline description",
                    ))
                } else {
                    None
                },
                shader: node.shader.finalize(node.buffer_group.label(), &gpu.device),
            })
            .collect();

        let material_array_cache = Vec::new();

        ScheduledPipeline {
            shaders,
            mesh_groups,
            uniform_cache,
            texture_cache,
            material_array_cache,
            name_to_buffer,
            name_to_uniform,
        }
    }
}

//
// The following are the actual pipeline objects
//

// This connects to the caches so the right data is pumped
// into the shader
pub struct ScheduledShaderNode {
    // Indices into uniform and texture caches
    bind_groups: Vec<BindGroupIndex>,
    // Index into buffer_groups array
    // Points to an array of ShaderGroups
    buffer_group: Index,
    // Index of the texture being output to
    // None means output should go to the screen
    targets: Option<Vec<Index>>,
    // Index of depth texture in texture_cache
    // None means no depth texture
    depth: Option<Index>,
    shader: Shader,
}

impl ScheduledShaderNode {
    // Begin a render pass.
    // TODO:
    //      Make this return a future that returns
    //      only after the pass has finished
    fn init_render_fn<'a>(
        &self,
        groups: &[MeshData],
        bind_groups: BindGroupCollection<'a>,
        encoder: &mut wgpu::CommandEncoder,
        targets: &[&wgpu::TextureView],
        depth: Option<&Texture>,
    ) {
        self.shader
            .render(groups, bind_groups, encoder, targets, depth);
    }
}

// A request to update certain buffers and uniforms in the Pipeline
// and to begin rendering based on the updated data
// Uses the builder pattern
pub struct ScheduledRenderRequest<'a> {
    buffers: HashMap<ShaderLabel, Vec<BufferUpdate<'a>>>,
    uniforms: HashMap<ShaderLabel, Vec<Option<&'a [u8]>>>,
}

impl<'a> ScheduledRenderRequest<'a> {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            uniforms: HashMap::new(),
        }
    }

    pub fn add_buffer(mut self, name: ShaderLabel, data: Vec<BufferUpdate<'a>>) -> Self {
        self.buffers.insert(name, data);
        self
    }

    pub fn add_uniform(mut self, name: ShaderLabel, data: Vec<Option<&'a [u8]>>) -> Self {
        self.uniforms.insert(name, data);
        self
    }
}

pub struct UniformUpdate {
    pub data: Box<[u8]>,
    // currently this is the largest size the tags can hold
    pub uniform_indx: u16,
    pub buffer_indx: u16,
}

// A request to update data stores in the caches accessed by a node
pub struct NodeUpdateRequest<'a> {
    pub buffers: Vec<BufferUpdate<'a>>,
    pub uniforms: Vec<UniformUpdate>,
    pub mesh_label: ShaderLabel,
}

pub struct ScheduledPipeline {
    // Shaders
    shaders: Vec<ScheduledShaderNode>,
    // Bindings
    mesh_groups: Vec<Vec<MeshData>>,
    uniform_cache: Vec<Uniform>,
    texture_cache: Vec<Texture>,
    material_array_cache: Vec<ScheduledMaterialList>,
    // Label Maps
    name_to_buffer: HashMap<ShaderLabel, Index>,
    name_to_uniform: HashMap<ShaderLabel, BindGroupIndex>,
}

impl ScheduledPipeline {
    fn get_bind_groups<'a>(
        &'a self,
        meshes: &[MeshData],
        bg_indices: &[BindGroupIndex],
    ) -> BindGroupCollection<'a> {
        let unique: Vec<&wgpu::BindGroup> = meshes
            .iter()
            .flat_map(|m| &m.unique_bind_groups)
            .map(|lbl| {
                let indx = self
                    .name_to_uniform
                    .get(lbl)
                    .expect("Failed to update unique bind group to ScheduledPipeline");

                match indx {
                    BindGroupIndex::Uniform(i) => &self.uniform_cache[*i].bind_group,
                    BindGroupIndex::Texture(i) => &self.texture_cache[*i].bind_group,
                    BindGroupIndex::MaterialList(i) => {
                        self.material_array_cache[*i].get_bg().unwrap()
                    }
                }
            })
            .collect();

        let unique_count = (unique.len() / meshes.len()) as u32;
        let mut shared_iter = bg_indices.iter();
        shared_iter
            .advance_by(unique_count as usize)
            .expect("More unique bind groups than bind groups");
        let shared = shared_iter
            .map(|indx| match indx {
                BindGroupIndex::Uniform(i) => &self.uniform_cache[*i].bind_group,
                BindGroupIndex::Texture(i) => &self.texture_cache[*i].bind_group,
                BindGroupIndex::MaterialList(i) => &self.material_array_cache[*i].get_bg().unwrap(),
            })
            .collect();

        BindGroupCollection {
            shared,
            unique,
            unique_count,
        }
    }

    // Update the caches used by a specific node. Since this is intended for batch processes,
    //  Queue.submit() must be called by caller to finalize buffer updates
    pub fn update_node_caches(&mut self, mut request: NodeUpdateRequest, gpu: &GPUBindings) {
        let buf_arr = *self
            .name_to_buffer
            .get(&request.mesh_label)
            .expect("tried updating non-existing shader node");

        request.buffers.drain(..).enumerate().for_each(|(i, upd)| {
            let mesh = &mut self.mesh_groups[buf_arr][i];
            match upd {
                BufferUpdate::Vertex(verts) => gpu.queue.write_buffer(&mesh.v_buf, 0, verts),
                BufferUpdate::Index(indices, new_index_count) => {
                    gpu.queue.write_buffer(&mesh.i_buf, 0, indices);
                    mesh.num_indices = mesh.num_indices.max(new_index_count);
                }
                BufferUpdate::VertexIndex(verts, indices, new_index_count) => {
                    gpu.queue.write_buffer(&mesh.v_buf, 0, verts);
                    gpu.queue.write_buffer(&mesh.i_buf, 0, indices);
                    mesh.num_indices = mesh.num_indices.max(new_index_count);
                }
                BufferUpdate::Raw(verts, indices) => unsafe {
                    gpu.queue.write_buffer(
                        &mesh.v_buf,
                        0,
                        verts
                            .as_ref()
                            .expect("Passed raw pointer of uninit vertices"),
                    );
                    gpu.queue.write_buffer(
                        &mesh.i_buf,
                        0,
                        indices
                            .as_ref()
                            .expect("Passed raw pointer of uninit indices"),
                    );
                },
                BufferUpdate::None => {}
            }
        });

        request.uniforms.drain(..).for_each(|upd| {
            let uniform = &mut self.uniform_cache[upd.uniform_indx as usize];
            let buffer = &mut uniform.buffers[upd.buffer_indx as usize];
            gpu.queue.write_buffer(buffer, 0, &upd.data);
        });
    }

    fn update_caches<'a>(&mut self, mut request: ScheduledRenderRequest<'a>, gpu: &GPUBindings) {
        request.uniforms.drain().for_each(|(name, mut updates)| {
            let indx = self.name_to_uniform.get(&name).unwrap();
            let uniform = match indx {
                BindGroupIndex::Uniform(i) => &self.uniform_cache[*i],
                BindGroupIndex::Texture(i) => todo!(),
                BindGroupIndex::MaterialList(i) => todo!(),
            };
            updates
                .drain(..)
                .enumerate()
                .filter_map(|(indx, data)| Some((indx, data?)))
                .for_each(|(indx, data)| gpu.queue.write_buffer(&uniform.buffers[indx], 0, data));
        });

        request.buffers.drain().for_each(|(name, data)| {
            let indx = *self.name_to_buffer.get(&name).unwrap();
            data.iter().enumerate().for_each(|(buffer, buf_update)| {
                let mesh = &mut self.mesh_groups[indx][buffer];
                match buf_update {
                    BufferUpdate::Vertex(verts) => gpu.queue.write_buffer(&mesh.v_buf, 0, verts),
                    BufferUpdate::Index(indices, new_index_count) => {
                        gpu.queue.write_buffer(&mesh.i_buf, 0, indices);
                        mesh.num_indices = mesh.num_indices.max(*new_index_count);
                    }
                    BufferUpdate::VertexIndex(verts, indices, new_index_count) => {
                        gpu.queue.write_buffer(&mesh.v_buf, 0, verts);
                        gpu.queue.write_buffer(&mesh.i_buf, 0, indices);
                        mesh.num_indices = mesh.num_indices.max(*new_index_count);
                    }
                    BufferUpdate::Raw(verts, indices) => unsafe {
                        gpu.queue.write_buffer(
                            &mesh.v_buf,
                            0,
                            verts
                                .as_ref()
                                .expect("Passed raw pointer of uninit vertices"),
                        );
                        gpu.queue.write_buffer(
                            &mesh.i_buf,
                            0,
                            indices
                                .as_ref()
                                .expect("Passed raw pointer of uninit indices"),
                        );
                    },
                    BufferUpdate::None => {}
                }
            });
        });
        gpu.queue.submit([]);
    }

    pub fn draw<'a>(
        &mut self,
        request: ScheduledRenderRequest<'a>,
        scrn_view: wgpu::TextureView,
        mut encoder: wgpu::CommandEncoder,
        out: SurfaceTexture,
        ws: &mut WindowState,
    ) -> Result<(), wgpu::SurfaceError> {
        // Update stored data
        self.update_caches(request, &ws.bindings);
        self.shaders.iter().for_each(|s| {
            let meshes = &self.mesh_groups[s.buffer_group];
            let shared_bgs = self.get_bind_groups(&meshes[..], &s.bind_groups[..]);

            let targets = if let Some(ref indices) = s.targets {
                indices
                    .iter()
                    .map(|indx| &self.texture_cache[*indx].view)
                    .collect()
            } else {
                vec![&scrn_view]
            };

            let depth = if let Some(indx) = s.depth {
                Some(&self.texture_cache[indx])
            } else {
                None
            };

            s.init_render_fn(meshes, shared_bgs, &mut encoder, &targets[..], depth);
        });

        // Finished rendering
        ws.post_render(encoder, out);
        Ok(())
    }
}
