//
//   A shader is an object which can just take some [Vertex] and [ExtraInfo]
//   and render to the screen with it
//

use wgpu::BindGroup;

use crate::mesh::MeshData;

use super::texture::Texture;

pub struct BindGroupCollecton<'a> {
    pub shared: Vec<&'a BindGroup>,
    pub unique: Vec<&'a BindGroup>,
    pub unique_offset: u32,
}

pub struct ShaderDefinition<'a> {
    pub shader_source: &'a str,
    pub bg_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub const_ranges: &'a [wgpu::PushConstantRange],
    pub vertex_desc: wgpu::VertexBufferLayout<'a>,
    pub primitive_state: wgpu::PrimitiveState,
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
                entry_point: Some("vs_main"),
                buffers: &[self.vertex_desc],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: self.blend_state,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: self.primitive_state,
            depth_stencil: self.depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Shader { pipeline }
    }
}

pub struct Shader {
    pub pipeline: wgpu::RenderPipeline,
}

impl Shader {
    pub fn render<'a>(
        &self,
        meshes: &[MeshData],
        bind_groups: BindGroupCollecton<'a>,
        textures: &[&wgpu::BindGroup],
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth: Option<&Texture>,
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
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: if depth.is_some() {
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth.unwrap().view,
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
        bind_groups.shared.iter().enumerate().for_each(|(i, bg)| {
            render_pass.set_bind_group(i as u32 + bind_groups.unique_offset, *bg, &[])
        });

        for (i, mesh) in meshes.iter().enumerate() {
            // reserve group 0 for textures
            let unique_offset = i * bind_groups.unique_offset as usize;
            for (j, bg) in bind_groups.unique
                [unique_offset..unique_offset + bind_groups.unique_offset as usize]
                .iter()
                .enumerate()
            {
                render_pass.set_bind_group(j as u32, *bg, &[]);
            }

            render_pass.set_vertex_buffer(0, mesh.v_buf.slice(..));
            render_pass.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
        }
    }
}
