// This is a less coupled pipeline which requires manually inserting
// bind groups and buffers. Scheduled refers to how the pipeline isn't
// closed to just the Allocator

use hashbrown::HashMap;
use wgpu::SurfaceTexture;

use crate::{
    mesh::MeshData,
    shader::{Shader, ShaderDefinition},
    texture::Texture,
    uniform::Uniform,
    wgpu,
    window_state::WindowState,
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
    pub view: Option<ShaderLabel>,
    pub depth: Option<ShaderLabel>,
    pub shader: ShaderDefinition<'a>,
}

// Details about how an ScheduledPipeline should be run.
// i.e. what shaders should be used, how larges caches should be, etc.
pub struct ScheduledPipelineDescription<'a> {
    pub shader_nodes: Vec<ScheduledShaderNodeDescription<'a>>,
    pub buffers: Vec<(ShaderLabel, Vec<MeshData>)>,
    pub bind_groups: Vec<ScheduledBindGroup<'a>>,
    pub textures: Vec<(ShaderLabel, ScheduledTexture<'a>)>,
}

impl ScheduledPipelineDescription<'_> {
    pub fn finalize(mut self, ws: &WindowState) -> ScheduledPipeline {
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

        self.bind_groups.iter().for_each(|data| {
            let name = data.label;

            match &data.form {
                ScheduledBindGroupType::ReadOnlyTexture(data) => {
                    // create texture
                    let texture = Texture::from_descs(
                        name.0,
                        &data.desc,
                        &data.sample_desc,
                        &data.view_desc,
                        &data.bg_layout_desc,
                        &ws.device,
                    );
                    ws.queue.write_texture(
                        // Tells wgpu where to copy the pixel data
                        wgpu::TexelCopyTextureInfo {
                            texture: &texture.data,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        // The actual pixel data
                        &data.data[..],
                        // The layout of the texture
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * data.desc.size.width),
                            rows_per_image: Some(data.desc.size.height),
                        },
                        data.desc.size,
                    );
                    // add to array
                    name_to_uniform.insert(name, BindGroupIndex::Texture(texture_cache.len()));
                    texture_cache.push(texture);
                }
                ScheduledBindGroupType::Uniform(data) => {
                    let (buffers, bind_group) = data.get_bind_group(name, ws);
                    name_to_uniform.insert(name, BindGroupIndex::Uniform(uniform_cache.len()));
                    uniform_cache.push(Uniform {
                        buffers,
                        bind_group,
                    });
                }
            }
        });

        self.textures.drain(..).for_each(|(name, texture)| {
            let final_texture = Texture::from_descs(
                &texture.label.0,
                &texture.desc,
                &texture.sample_desc,
                &texture.view_desc,
                &texture.bg_layout_desc,
                &ws.device,
            );
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
                        *name_to_uniform.get(name).expect(
                            &format!("Shader references a bind group not passed into pipeline description: {:?}", name.0)
                        )
                    })
                    .collect(),
                buffer_group: *name_to_buffer
                    .get(&node.buffer_group)
                    .expect("Shader references a buffer not passed into pipeline description"),
                view: if let Some(name) = node.view {
                    Some(*name_to_texture.get(&name).expect(
                        "Shader references a view texture not passed into pipeline description",
                    ))
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
                shader: node.shader.finalize(&ws.device, &ws.config),
            })
            .collect();

        ScheduledPipeline {
            shaders,
            mesh_groups,
            uniform_cache,
            texture_cache,
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
    view: Option<Index>,
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
    fn init_render_fn(
        &self,
        groups: &[MeshData],
        bind_groups: &[&wgpu::BindGroup],
        textures: &[&wgpu::BindGroup],
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth: Option<&Texture>,
    ) {
        self.shader
            .render(groups, bind_groups, textures, encoder, view, depth);
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

// A request to update data stores in the caches accessed by a node
pub struct NodeUpdateRequest<'a> {
    pub buffers: Vec<BufferUpdate<'a>>,
    pub uniforms: Vec<Option<Box<[u8]>>>,
    pub mesh_label: ShaderLabel,
}

pub struct ScheduledPipeline {
    shaders: Vec<ScheduledShaderNode>,
    // any groups held by the pipeline must be 'static to guarantee
    // they outlive its lifetime.
    mesh_groups: Vec<Vec<MeshData>>,
    uniform_cache: Vec<Uniform>,
    texture_cache: Vec<Texture>,
    name_to_buffer: HashMap<ShaderLabel, Index>,
    name_to_uniform: HashMap<ShaderLabel, BindGroupIndex>,
}

impl ScheduledPipeline {
    fn get_bind_groups<'a>(&'a self, indices: &[BindGroupIndex]) -> Vec<&'a wgpu::BindGroup> {
        indices
            .iter()
            .filter_map(|indx| match indx {
                BindGroupIndex::Uniform(i) => Some(&self.uniform_cache[*i].bind_group),
                BindGroupIndex::Texture(_) => None,
            })
            .collect()
    }

    // Update the caches used by a specific node. Since this is intended for batch processes,
    //  Queue.submit() must be called by caller to finalize buffer updates
    pub fn update_node_caches(&self, mut request: NodeUpdateRequest, ws: &WindowState) {
        let buf_arr = *self
            .name_to_buffer
            .get(&request.mesh_label)
            .expect("tried updating non-existing shader node");

        request.buffers.drain(..).enumerate().for_each(|(i, upd)| {
            let mesh = &self.mesh_groups[buf_arr][i];
            match upd {
                BufferUpdate::Vertex(verts) => ws.queue.write_buffer(&mesh.v_buf, 0, verts),
                BufferUpdate::Index(indices) => ws.queue.write_buffer(&mesh.i_buf, 0, indices),
                BufferUpdate::VertexIndex(verts, indices) => {
                    ws.queue.write_buffer(&mesh.v_buf, 0, verts);
                    ws.queue.write_buffer(&mesh.i_buf, 0, indices);
                }
                BufferUpdate::Raw(verts, indices) => unsafe {
                    ws.queue.write_buffer(
                        &mesh.v_buf,
                        0,
                        verts
                            .as_ref()
                            .expect("Passed raw pointer of uninit vertices"),
                    );
                    ws.queue.write_buffer(
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
    }

    fn update_caches<'a>(&self, mut request: ScheduledRenderRequest<'a>, ws: &WindowState) {
        request.uniforms.drain().for_each(|(name, mut updates)| {
            let indx = self.name_to_uniform.get(&name).unwrap();
            let uniform = match indx {
                BindGroupIndex::Uniform(i) => &self.uniform_cache[*i],
                BindGroupIndex::Texture(i) => todo!(),
            };
            updates
                .drain(..)
                .enumerate()
                .filter_map(|(indx, data)| Some((indx, data?)))
                .for_each(|(indx, data)| ws.queue.write_buffer(&uniform.buffers[indx], 0, data));
        });

        request.buffers.drain().for_each(|(name, data)| {
            let indx = *self.name_to_buffer.get(&name).unwrap();
            data.iter().enumerate().for_each(|(buffer, buf_update)| {
                let mesh = &self.mesh_groups[indx][buffer];
                match buf_update {
                    BufferUpdate::Vertex(verts) => ws.queue.write_buffer(&mesh.v_buf, 0, verts),
                    BufferUpdate::Index(indices) => ws.queue.write_buffer(&mesh.i_buf, 0, indices),
                    BufferUpdate::VertexIndex(verts, indices) => {
                        ws.queue.write_buffer(&mesh.v_buf, 0, verts);
                        ws.queue.write_buffer(&mesh.i_buf, 0, indices);
                    }
                    BufferUpdate::Raw(verts, indices) => unsafe {
                        ws.queue.write_buffer(
                            &mesh.v_buf,
                            0,
                            verts
                                .as_ref()
                                .expect("Passed raw pointer of uninit vertices"),
                        );
                        ws.queue.write_buffer(
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
        ws.queue.submit([]);
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
        self.update_caches(request, ws);

        self.shaders.iter().for_each(|s| {
            let groups = &self.mesh_groups[s.buffer_group];
            let bgs = self.get_bind_groups(&s.bind_groups[..]);
            let textures = self
                .texture_cache
                .iter()
                .map(|text| &text.bind_group)
                .collect::<Vec<&wgpu::BindGroup>>();

            let view = if let Some(indx) = s.view {
                &self.texture_cache[indx].view
            } else {
                &scrn_view
            };

            let depth = if let Some(indx) = s.depth {
                Some(&self.texture_cache[indx])
            } else {
                None
            };

            s.init_render_fn(groups, &bgs[..], &textures[..], &mut encoder, view, depth);
        });

        // Finished rendering
        ws.post_render(encoder, out);
        Ok(())
    }
}
