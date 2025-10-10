use cgmath::Vector3;
use frosty_alloc::FrostyAllocatable;

pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
    fn pos(&self) -> Vector3<f32>;
    fn set_normal(&mut self, norm: Vector3<f32>);
}

pub struct ScaleFactor {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<f32> for ScaleFactor {
    fn from(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub world_pos: [f32; 3],  // x, y, z
    pub tex_coords: [f32; 2], // more information will be needed when lighting is implemented
    pub mat: u32,
    pub normal: [f32; 3],
}

impl MeshVertex {
    pub fn max_dist(verts: &Vec<MeshVertex>) -> cgmath::Vector3<f32> {
        let min_x = f32::MAX;
        let min_y = f32::MAX;
        let min_z = f32::MAX;
        let max_x = f32::MIN;
        let max_y = f32::MIN;
        let max_z = f32::MAX;
        for v in verts {
            // no switch statemente
        }
        todo!()
    }

    pub fn transform_verts(verts: &mut Vec<MeshVertex>, delta: [f32; 3]) {
        for v in verts {
            v.world_pos = [
                v.world_pos[0] + delta[0],
                v.world_pos[1] + delta[1],
                v.world_pos[2] + delta[2],
            ]
        }
    }

    fn scale_verts_from_point(
        verts: &mut Vec<MeshVertex>,
        factor: impl Into<ScaleFactor>,
        point: [f32; 3],
    ) {
        let scale: ScaleFactor = factor.into();
        for v in verts {
            let base_vector = cgmath::Vector3::new(
                v.world_pos[0] - point[0],
                v.world_pos[1] - point[1],
                v.world_pos[2] - point[2],
            );
            v.world_pos = [
                point[0] + base_vector.x * scale.x,
                point[1] + base_vector.y * scale.y,
                point[2] + base_vector.z * scale.z,
            ];
        }
    }

    fn rotate_verts_about_line(
        _verts: &Vec<MeshVertex>,
        _line: cgmath::Vector3<f32>,
        _rotation: cgmath::Rad<f32>,
    ) {
        todo!()
    }
}

unsafe impl FrostyAllocatable for MeshVertex {}

impl Vertex for MeshVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // world pos
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // for @location(n) when defining struct in shader
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1, // location(1)
                    format: wgpu::VertexFormat::Float32x2,
                },
                // material
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 2, // location(2)
                    format: wgpu::VertexFormat::Uint32,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>()
                        + std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<u32>())
                        as wgpu::BufferAddress,
                    shader_location: 3, // location(3)
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }

    fn pos(&self) -> Vector3<f32> {
        Vector3::from(self.world_pos)
    }

    fn set_normal(&mut self, norm: Vector3<f32>) {
        self.normal = norm.into();
    }
}

// A quad representing the screen. Useful for
// shaders that apply screen effects
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenQuadVertex {
    pub clip_pos: [f32; 2],
    pub tex_coords: [f32; 2],
}

unsafe impl FrostyAllocatable for ScreenQuadVertex {}
impl Vertex for ScreenQuadVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ScreenQuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // screen pos
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // tex coords
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }

    fn pos(&self) -> Vector3<f32> {
        Vector3 {
            x: self.clip_pos[0],
            y: self.clip_pos[1],
            z: 0.0,
        }
    }

    fn set_normal(&mut self, _: Vector3<f32>) {
        panic!("Cant set normal to screen quad vertex!");
    }
}
