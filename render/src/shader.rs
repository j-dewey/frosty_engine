//
//   A shader is an object which can just take some [Vertex] and [ExtraInfo]
//   and render to the screen with it
//

use crate::SceneRenderInfo;

use super::texture::Texture;

enum BufferType<'a> {
    Owned(wgpu::Buffer),
    Borrowed(&'a wgpu::Buffer),
}

impl<'a> BufferType<'a> {
    fn as_ref(&self) -> &wgpu::Buffer {
        match self {
            Self::Owned(buf) => &buf,
            Self::Borrowed(buf) => buf,
        }
    }
}

pub struct ShaderGroup<'a> {
    v_buf: BufferType<'a>,
    i_buf: BufferType<'a>,
    mat_index: u32,
    num_indices: u32,
}

impl<'a> ShaderGroup<'a> {
    pub fn new_owned(
        v_buf: wgpu::Buffer,
        i_buf: wgpu::Buffer,
        mat_index: u32,
        num_indices: u32,
    ) -> Self {
        Self {
            v_buf: BufferType::Owned(v_buf),
            i_buf: BufferType::Owned(i_buf),
            mat_index,
            num_indices,
        }
    }

    pub fn new_borrowed(
        v_buf: &'a wgpu::Buffer,
        i_buf: &'a wgpu::Buffer,
        mat_index: u32,
        num_indices: u32,
    ) -> Self {
        Self {
            v_buf: BufferType::Borrowed(v_buf),
            i_buf: BufferType::Borrowed(i_buf),
            mat_index,
            num_indices,
        }
    }

    // (v buf, i buf)
    pub fn get_buffers(&self) -> (&wgpu::Buffer, &wgpu::Buffer) {
        (self.v_buf.as_ref(), self.i_buf.as_ref())
    }
}

pub struct ShaderDefinition<'a> {
    pub shader_source: &'a str,
    pub bg_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub const_ranges: &'a [wgpu::PushConstantRange],
    pub vertex_desc: wgpu::VertexBufferLayout<'a>,
    pub bind_groups: Vec<Option<wgpu::BindGroup>>,
    pub blend_state: Option<wgpu::BlendState>,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub depth_buffer: Option<Texture>,
}

impl<'a> ShaderDefinition<'a> {
    pub fn finalize(self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Shader {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(self.shader_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: self.bg_layouts,
            push_constant_ranges: self.const_ranges,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[self.vertex_desc],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: self.blend_state,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: self.depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Shader {
            pipeline,
            bind_groups: self.bind_groups,
            depth_buffer: self.depth_buffer,
        }
    }
}

pub struct Shader {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<Option<wgpu::BindGroup>>,
    pub depth_buffer: Option<Texture>,
}

impl Shader {
    pub fn render(
        &self,
        meshes: &[&ShaderGroup],
        bind_groups: &[&wgpu::BindGroup],
        ri: &SceneRenderInfo,
        bg_offset: usize,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: if self.depth_buffer.is_some() {
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_buffer.as_ref().unwrap().view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                })
            } else {
                None
            },
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        for (i, bg) in bind_groups.iter().enumerate() {
            render_pass.set_bind_group((i + bg_offset) as u32, bg, &[]);
        }
        if bg_offset > 0 {
            // at some point make this not an if statement
            render_pass.set_bind_group(0, &ri.material_array, &[]);
        }
        for mesh in meshes {
            let (v_buf, i_buf) = mesh.get_buffers();
            /*
            if let (Some(bind_group), true) = (
                ri.mats.get(mesh.mat_index as usize),
                mesh.mat_index != u32::MAX,
            ) {
                render_pass.set_bind_group(0, &bind_group.get_text().bind_group, &[]);
            }
            */
            render_pass.set_vertex_buffer(0, v_buf.slice(..));
            render_pass.set_index_buffer(i_buf.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
        }
    }
}
