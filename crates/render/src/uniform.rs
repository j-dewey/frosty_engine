// A way to pair a BindGroup to the Buffer that
// stores its data
pub struct Uniform {
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
}
