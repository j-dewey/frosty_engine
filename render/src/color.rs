use super::texture::Texture;
use super::window_state::WindowState;

pub struct Color(pub [f32; 4]);

impl Color{
    pub fn as_texture(&self, ws: &WindowState, bg_layout: &wgpu::BindGroupLayout) -> Texture{
        let texture_size = wgpu::Extent3d{
            width: 1,
            height: 1,
            depth_or_array_layers: 1
        };
        let data = ws.device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("color"),
                view_formats: &[],
            }
        );

        ws.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &data,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &[(255.0 * self.0[0]) as u8,(255.0 * self.0[1]) as u8, (255.0 * self.0[2]) as u8, (255.0 * self.0[3]) as u8],
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            texture_size
        );

        let view = data.create_view(&wgpu::TextureViewDescriptor{ ..Default::default() });
        let sampler = ws.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = ws.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: bg_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
                label: Some("color"),
            }
        );
        Texture::from_parts(data, view, sampler, bind_group)
    }
}