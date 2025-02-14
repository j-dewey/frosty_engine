#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GuiVertex {
    pub screen_pos: [f32; 2], // x, y
    pub color: [f32; 3],
}

impl GuiVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GuiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // screen pos
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // for @location(n) when defining struct in shader
                    format: wgpu::VertexFormat::Float32x2,
                },
                // color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1, // location(1)
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GuiTextureVertex {
    pub screen_pos: [f32; 2], // x, y
    pub tex_coords: [f32; 2],
    pub text: u32,
}

impl GuiTextureVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GuiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // screen pos
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // for @location(n) when defining struct in shader
                    format: wgpu::VertexFormat::Float32x2,
                },
                // color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1, // location(1)
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2, // location(2)
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }

    pub fn generate_quad_verts(
        tlc: [f32; 2],
        trc: [f32; 2],
        blc: [f32; 2],
        brc: [f32; 2],
        text: u32,
    ) -> [Self; 4] {
        [
            Self {
                screen_pos: tlc,
                tex_coords: [0.0, 0.0],
                text,
            },
            Self {
                screen_pos: trc,
                tex_coords: [1.0, 0.0],
                text,
            },
            Self {
                screen_pos: blc,
                tex_coords: [0.0, 1.0],
                text,
            },
            Self {
                screen_pos: brc,
                tex_coords: [1.0, 1.0],
                text,
            },
        ]
    }
}
