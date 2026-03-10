use std::num::NonZeroU32;

use wgpu::{BindGroup, Device};
use winit::dpi::PhysicalSize;

use crate::{texture::Texture, window_state::GPUBindings};

use super::{ScheduledBuffer, ShaderLabel};

// Uniforms and Textures are conceptually different
// from eachother, but both communicate with the GPU
// through BindGroups

pub struct ScheduledBindGroup<'a> {
    pub label: ShaderLabel,
    pub form: ScheduledBindGroupType<'a>,
}

impl ScheduledBindGroup<'_> {
    pub fn to_bind_group(self, gpu: &GPUBindings) -> BindGroup {
        self.form.to_bind_group(self.label, gpu)
    }
}

pub enum ScheduledBindGroupType<'a> {
    ReadOnlyTexture(ScheduledTexture<'a>),
    ReadOnlyTextureArray(Vec<Texture>),
    Uniform(ScheduledUniform<'a>),
}

impl<'a> ScheduledBindGroupType<'a> {
    pub fn to_bind_group(self, label: ShaderLabel, gpu: &GPUBindings) -> wgpu::BindGroup {
        match self {
            ScheduledBindGroupType::Uniform(data) => {
                let buffers = data
                    .buffers
                    .iter()
                    .map(|raw| raw.get_buffer(&gpu.device))
                    .collect::<Vec<wgpu::Buffer>>();
                let entries = buffers
                    .iter()
                    .enumerate()
                    .map(|(indx, buf)| wgpu::BindGroupEntry {
                        binding: indx as u32,
                        resource: buf.as_entire_binding(),
                    })
                    .collect::<Vec<wgpu::BindGroupEntry>>();
                gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(label.label()),
                    layout: &data.layout,
                    entries: &entries[..],
                })
            }
            ScheduledBindGroupType::ReadOnlyTextureArray(textures) => {
                let views: Vec<&wgpu::TextureView> = textures.iter().map(|t| &t.view).collect();

                let layout =
                    gpu.device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            entries: &[
                                wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::FRAGMENT,
                                    ty: wgpu::BindingType::Texture {
                                        multisampled: false,
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                        sample_type: wgpu::TextureSampleType::Float {
                                            filterable: true,
                                        },
                                    },
                                    count: NonZeroU32::new(textures.len() as u32),
                                },
                                wgpu::BindGroupLayoutEntry {
                                    binding: 1,
                                    visibility: wgpu::ShaderStages::FRAGMENT,
                                    ty: wgpu::BindingType::Sampler(
                                        wgpu::SamplerBindingType::Filtering,
                                    ),
                                    count: None,
                                },
                            ],
                            label: Some("texture_array_bind_group_layout"),
                        });

                gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("texture_array"),
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureViewArray(&views[..]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&textures[0].sampler),
                        },
                    ],
                })
            }
            ScheduledBindGroupType::ReadOnlyTexture(_) => {
                todo!()
            }
        }
    }
}

pub enum ScheduledTexture<'a> {
    Unloaded {
        label: &'a ShaderLabel,
        desc: wgpu::TextureDescriptor<'a>,
        sample_desc: wgpu::SamplerDescriptor<'a>,
        view_desc: wgpu::TextureViewDescriptor<'a>,
        bg_layout_desc: wgpu::BindGroupLayoutDescriptor<'a>,
        size: wgpu::Extent3d,
        data: Option<Box<[u8]>>,
    },
    Loaded {
        label: ShaderLabel,
        texture: Texture,
    },
}

impl<'a> ScheduledTexture<'a> {
    // Create a new depth texture
    pub fn depth(label: &'a ShaderLabel, size: winit::dpi::PhysicalSize<u32>) -> Self {
        let texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label.label()),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Depth32Float],
        };
        let view_desc = wgpu::TextureViewDescriptor {
            ..Default::default()
        };
        let sample_desc = wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        };
        let bg_layout_desc = wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        };

        Self::Unloaded {
            label,
            desc,
            sample_desc,
            view_desc,
            bg_layout_desc,
            size: texture_size,
            data: None,
        }
    }

    // create a new texture that can be rendered to and read from
    pub fn render_target(label: ShaderLabel, size: PhysicalSize<u32>, device: &Device) -> Self {
        let texture = Texture::new_render(label.label(), size, device);
        Self::Loaded { label, texture }
    }

    pub fn get_label(&self) -> &ShaderLabel {
        match self {
            &ScheduledTexture::Loaded { ref label, .. } => &label,
            &ScheduledTexture::Unloaded { ref label, .. } => &label,
        }
    }

    pub fn to_texture(self, gpu: &GPUBindings) -> Texture {
        match self {
            Self::Unloaded {
                label,
                desc,
                sample_desc,
                view_desc,
                bg_layout_desc,
                size,
                data,
            } => {
                let texture = Texture::from_descs(
                    label.label(),
                    &desc,
                    &sample_desc,
                    &view_desc,
                    &bg_layout_desc,
                    size,
                    &gpu.device,
                );

                if let Some(pix_data) = &data {
                    gpu.queue.write_texture(
                        // Tells wgpu where to copy the pixel data
                        wgpu::TexelCopyTextureInfo {
                            texture: &texture.data,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        // The actual pixel data
                        &pix_data[..],
                        // The layout of the texture
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * desc.size.width),
                            rows_per_image: Some(desc.size.height),
                        },
                        desc.size,
                    );
                }
                texture
            }
            Self::Loaded { label, texture } => texture,
        }
    }
}

pub struct ScheduledUniform<'a> {
    pub layout: &'a wgpu::BindGroupLayout,
    pub buffers: &'a [ScheduledBuffer<'a>],
}

impl ScheduledUniform<'_> {
    pub fn get_bind_group(
        &self,
        label: ShaderLabel,
        gpu: &GPUBindings,
    ) -> (Vec<wgpu::Buffer>, BindGroup) {
        let buffers = self
            .buffers
            .iter()
            .map(|raw| raw.get_buffer(&gpu.device))
            .collect::<Vec<wgpu::Buffer>>();
        let entries = buffers
            .iter()
            .enumerate()
            .map(|(indx, buf)| wgpu::BindGroupEntry {
                binding: indx as u32,
                resource: buf.as_entire_binding(),
            })
            .collect::<Vec<wgpu::BindGroupEntry>>();
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label.label()),
            layout: &self.layout,
            entries: &entries[..],
        });
        (buffers, bind_group)
    }
}

// This is just for semantic distinction
// Materials are really stored as TextureArrays
pub struct ScheduledMaterialList {
    pub textures: Vec<ShaderLabel>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl ScheduledMaterialList {
    pub fn get_bg(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }
}
