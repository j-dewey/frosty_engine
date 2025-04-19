use wgpu::BindGroup;

use crate::window_state::WindowState;

use super::{ScheduledBuffer, ShaderLabel};

// Uniforms and Textures are conceptually different
// from eachother, but both communicate with the GPU
// through BindGroups

pub struct ScheduledBindGroup<'a> {
    pub label: ShaderLabel,
    pub form: ScheduledBindGroupType<'a>,
}

impl ScheduledBindGroup<'_> {
    pub fn to_bind_group(self, ws: &WindowState) -> BindGroup {
        match self.form {
            ScheduledBindGroupType::Uniform(data) => {
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
                ws.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(self.label.0),
                    layout: &data.layout,
                    entries: &entries[..],
                })
            }
            ScheduledBindGroupType::ReadOnlyTexture(_) => {
                todo!()
            }
        }
    }
}

pub enum ScheduledBindGroupType<'a> {
    ReadOnlyTexture(ScheduledTexture<'a>),
    Uniform(ScheduledUniform<'a>),
}

pub struct ScheduledTexture<'a> {
    pub label: ShaderLabel,
    pub desc: wgpu::TextureDescriptor<'a>,
    pub sample_desc: wgpu::SamplerDescriptor<'a>,
    pub view_desc: wgpu::TextureViewDescriptor<'a>,
    pub bg_layout_desc: wgpu::BindGroupLayoutDescriptor<'a>,
    pub data: Box<[u8]>,
}

pub struct ScheduledUniform<'a> {
    pub layout: &'a wgpu::BindGroupLayout,
    pub buffers: &'a [ScheduledBuffer<'a>],
}

impl ScheduledUniform<'_> {
    pub fn get_bind_group(
        &self,
        label: ShaderLabel,
        ws: &WindowState,
    ) -> (Vec<wgpu::Buffer>, BindGroup) {
        let buffers = self
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
            label: Some(label.0),
            layout: &self.layout,
            entries: &entries[..],
        });
        (buffers, bind_group)
    }
}
