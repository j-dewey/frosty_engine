// A way to pair a BindGroup to the Buffer that
// stores its data
pub struct Uniform {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}
