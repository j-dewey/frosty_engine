use wgpu::util::DeviceExt;

pub enum BufferUpdate<'a> {
    Vertex(&'a [u8]),
    Index(&'a [u8], u32),
    VertexIndex(&'a [u8], &'a [u8], u32),
    Raw(*const [u8], *const [u8]),
    None,
}

impl BufferUpdate<'_> {
    pub fn is_some(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }
}

pub struct ScheduledBuffer<'a> {
    pub desc: wgpu::util::BufferInitDescriptor<'a>,
}

impl ScheduledBuffer<'_> {
    // create a buffer based on the data described in the desc
    pub(crate) fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let buffer = device.create_buffer_init(&self.desc);
        buffer
    }
}
