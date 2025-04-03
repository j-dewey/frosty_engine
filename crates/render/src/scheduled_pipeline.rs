// This is a less coupled pipeline which requires manually inserting
// bind groups and buffers. Scheduled refers to how the pipeline isn't
// closed to just the Allocator

use hashbrown::HashMap;

use crate::{
    mesh::MeshData,
    shader::{Shader, ShaderDefinition},
    texture::Texture,
    uniform::Uniform,
    wgpu::{self, util::DeviceExt},
    window_state::WindowState,
};

type Index = usize;

// Buffer / BindGroup / Texture Name
// This is a name to allow for accessing specific Buffers
// and BindGroups in ScheduledRequests and the ScheduledPipeline. Also
// for setting up Textures in ScheduledDescription.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ShaderLabel(&'static str);

// Uniforms and Textures are coneceptually different
// from eachother, but both communicate wit the GPU
// through BindGroups. This index allows for easily
// accessing BindGroups in the required order even
// though they are placed in seperate caches.
enum BindGroupIndex {
    Texture(Index),
    Uniform(Index),
}

pub enum BufferUpdate<'a> {
    Vertex(&'a [u8]),
    Index(&'a [u8]),
    VertexIndex(&'a [u8], &'a [u8]),
    None,
}

//
//  The following are descriptions used to define
//  Certain parts of the pipeline
//

pub struct ScheduledTexture<'a> {
    pub label: ShaderLabel,
    pub desc: wgpu::TextureDescriptor<'a>,
    pub sample_desc: wgpu::SamplerDescriptor<'a>,
    pub view_desc: wgpu::TextureViewDescriptor<'a>,
}

pub struct ScheduledBuffer<'a> {
    pub desc: wgpu::util::BufferInitDescriptor<'a>,
}

impl ScheduledBuffer<'_> {
    // create a buffer based on the data described in the desc
    fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let buffer = device.create_buffer_init(&self.desc);
        buffer
    }
}

pub struct ScheduledBindGroup<'a> {
    pub label: ShaderLabel,
    pub layout: wgpu::BindGroupLayout,
    pub buffers: &'a [ScheduledBuffer<'a>],
}

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
    pub bind_groups: Vec<(ShaderLabel, ScheduledBindGroup<'a>)>,
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

        self.bind_groups.iter().for_each(|(name, data)| {
            let buffers = data
                .buffers
                .iter()
                .map(|raw| raw.get_buffer(&ws.device))
                .collect::<Vec<wgpu::Buffer>>();
            let entries = buffers
                .iter()
                .enumerate()
                .map(|(indx, buf)| wgpu::BindGroupEntry {
                    binding: indx as u32,
                    resource: buf.as_entire_binding(),
                })
                .collect::<Vec<wgpu::BindGroupEntry>>();
            let bind_group = ws.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(data.label.0),
                layout: &data.layout,
                entries: &entries[..],
            });

            name_to_uniform.insert(*name, uniform_cache.len());
            uniform_cache.push(Uniform {
                buffers,
                bind_group,
            });
        });

        self.textures.drain(..).for_each(|(name, texture)| {
            let final_texture = Texture::from_descs(
                &texture.label.0,
                texture.desc,
                texture.sample_desc,
                texture.view_desc,
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
                        BindGroupIndex::Uniform(*name_to_buffer.get(name).expect(
                            "Shader references a bind group not passed into pipeline description",
                        ))
                    })
                    .collect(),
                buffer_group: *name_to_buffer
                    .get(&node.buffer_group)
                    .expect("Shader references a buffer not pased into pipeline description"),
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

// This is a mirror of the regular ShaderNode
struct ScheduledShaderNode {
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
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth: Option<&Texture>,
    ) {
        self.shader
            .render(groups, bind_groups, encoder, view, depth);
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

pub struct ScheduledPipeline {
    shaders: Vec<ScheduledShaderNode>,
    // any groups held by the pipeline must be 'static to guarantee
    // they outlive its lifetime.
    mesh_groups: Vec<Vec<MeshData>>,
    uniform_cache: Vec<Uniform>,
    texture_cache: Vec<Texture>,
    name_to_buffer: HashMap<ShaderLabel, Index>,
    name_to_uniform: HashMap<ShaderLabel, Index>,
}

impl ScheduledPipeline {
    fn get_bind_groups<'a>(&'a self, indices: &[BindGroupIndex]) -> Vec<&'a wgpu::BindGroup> {
        indices
            .iter()
            .map(|indx| match indx {
                BindGroupIndex::Uniform(i) => &self.uniform_cache[*i].bind_group,
                BindGroupIndex::Texture(i) => &self.texture_cache[*i].bind_group,
            })
            .collect()
    }

    fn update_caches<'a>(&self, mut request: ScheduledRenderRequest<'a>, ws: &WindowState) {
        request.uniforms.drain().for_each(|(name, mut updates)| {
            let indx = self.name_to_uniform.get(&name).unwrap();
            let uniform = &self.uniform_cache[*indx];
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
                    BufferUpdate::None => {}
                }
            });
        });
        ws.queue.submit([]);
    }

    pub fn draw<'a>(
        &mut self,
        request: ScheduledRenderRequest<'a>,
        ws: &mut WindowState,
    ) -> Result<(), wgpu::SurfaceError> {
        // Update stored data
        self.update_caches(request, ws);

        // Call each render fn
        let (scrn_view, mut encoder, out) = ws.prep_render()?;
        self.shaders.iter().for_each(|s| {
            let groups = &self.mesh_groups[s.buffer_group];
            let bgs = self.get_bind_groups(&s.bind_groups[..]);

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

            s.init_render_fn(groups, &bgs[..], &mut encoder, view, depth);
        });

        // Finished rendering
        ws.post_render(encoder, out);
        Ok(())
    }
}
