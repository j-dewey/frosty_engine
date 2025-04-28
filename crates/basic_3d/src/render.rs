use engine_core::render_core::{DynamicNodeDefinition, DynamicRenderPipeline, GivesBindGroup};
use engine_core::Spawner;
use frosty_alloc::FrostyAllocatable;
use render::mesh::{MeshData, MeshyObject};
use render::scheduled_pipeline::{
    ScheduledBindGroup, ScheduledBindGroupType, ScheduledBuffer, ScheduledTexture, ScheduledUniform,
};
use render::wgpu::{self, BindGroupLayout, BufferUsages};
use render::{
    mesh::Mesh,
    scheduled_pipeline::{
        ScheduledPipelineDescription, ScheduledShaderNodeDescription, ShaderLabel,
    },
    shader::ShaderDefinition,
    vertex::{MeshVertex, Vertex},
    window_state::WindowState,
};

// These node layouts will automatically init missing components required for
// rendering
use engine_core::{query::DynQuery, MASTER_THREAD};

use crate::camera::Camera3d;

pub const MESH_BUFFER_LABEL: ShaderLabel = ShaderLabel("mesh-buffer");
pub const MESH_CAMERA_LABEL: ShaderLabel = ShaderLabel("mesh-camera-uniform");
pub const MESH_SHADER_LABEL: ShaderLabel = ShaderLabel("mesh-shader");
pub const MESH_TEXTURE_LABEL: ShaderLabel = ShaderLabel("mesh-texture-array");
pub const MESH_TEXTURE_SAMPLER_LABEL: ShaderLabel = ShaderLabel("mesh-texture-array");
pub const MESH_TEXTURE_VIEWS_LABEL: ShaderLabel = ShaderLabel("mesh-texture-array");

pub struct Material {}
unsafe impl FrostyAllocatable for Material {}

pub fn load_default_textures<'a>(
    ws: &'a WindowState,
) -> (
    wgpu::TextureDescriptor<'a>,
    wgpu::TextureViewDescriptor<'a>,
    wgpu::SamplerDescriptor<'a>,
) {
    let raw_texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("diffuse_texture"),
        view_formats: &[],
    };
    let view = wgpu::TextureViewDescriptor::default();
    let sampler = wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    };

    (raw_texture_desc, view, sampler)
}

pub fn load_mesh_shader_layout<'a>(
    alloc: &mut Spawner,
    layouts: &'a [&'a BindGroupLayout],
    ws: &WindowState,
) -> (
    ScheduledShaderNodeDescription<'a>,
    DynamicNodeDefinition<Mesh<MeshVertex>>,
    Vec<MeshData>,
    Box<[u8]>,
) {
    let mut bind_groups: DynQuery<dyn GivesBindGroup> = DynQuery::new_empty();
    let mut camera = alloc
        .get_query::<Camera3d>(MASTER_THREAD)
        .expect("No Camera3d detected during mesh shader init")
        .next_handle()
        .expect("Failed to get Camera3D from Query");
    let camera_data = camera
        .get_access(MASTER_THREAD)
        .expect("Failed to access camera handle during shader init")
        .as_ref()
        .get_uniform_data();

    bind_groups.push(&mut camera);

    let mut mesh_data = Vec::new();
    if let Some(mut meshes) = alloc.get_query::<Mesh<MeshVertex>>(MASTER_THREAD) {
        meshes.for_each(|mesh| {
            mesh.print_id();
            let (inds, inds_count) = mesh.as_ref().get_indices();
            let verts = mesh.as_ref().get_verts();
            let i_buf = ws.load_index_buffer("mesh_indices", &inds[..]);
            let v_buf = ws.load_vertex_buffer("mesh_vertices", &verts[..]);
            mesh_data.push(MeshData {
                v_buf,
                i_buf,
                num_indices: inds_count as u32,
                texture_index: 0,
            });
        });
    }

    let mut material_data: Vec<Material> = Vec::new();
    if let Some(mut materials) = alloc.get_query::<Material>(MASTER_THREAD) {
        materials.for_each(|mat| todo!("Implement material"));
    }

    let schedule_node = ScheduledShaderNodeDescription {
        buffer_group: MESH_BUFFER_LABEL,
        bind_groups: vec![MESH_TEXTURE_LABEL, MESH_CAMERA_LABEL], // camera, texture array
        view: None,                                               // output to screen
        depth: None,                                              // not set up yet
        shader: ShaderDefinition {
            shader_source: include_str!("shaders/mesh.wgsl"),
            bg_layouts: layouts, // camera, texture array
            const_ranges: &[],
            vertex_desc: MeshVertex::desc(),
            primitive_state: render::wgpu::PrimitiveState::default(),
            blend_state: None,
            depth_buffer: None, // not set up yet
            depth_stencil: None,
        },
    };

    let dynamic_node = DynamicNodeDefinition {
        bind_groups,
        node: MESH_BUFFER_LABEL,
        _pd: std::marker::PhantomData {},
    };

    (schedule_node, dynamic_node, mesh_data, camera_data)
}

// For now this just renders meshes
pub fn general_3d_pipeline(alloc: &mut Spawner, ws: &WindowState) -> DynamicRenderPipeline {
    let (texture_desc, view_desc, sample_desc) = load_default_textures(ws);
    let texture_bg_layout_desc = wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
    let texture_bg_layout = ws.device.create_bind_group_layout(&texture_bg_layout_desc);

    let camera_layout = Camera3d::get_bind_group_layout(ws);
    let layouts = &[&texture_bg_layout, &camera_layout];
    let (scheduled_mesh_node, dynamic_mesh_node, mesh_data, camera) =
        load_mesh_shader_layout(alloc, &layouts[..], ws);

    let rp = ScheduledPipelineDescription {
        shader_nodes: vec![scheduled_mesh_node],
        buffers: vec![(MESH_BUFFER_LABEL, mesh_data)],
        bind_groups: vec![
            ScheduledBindGroup {
                label: MESH_TEXTURE_LABEL,
                form: ScheduledBindGroupType::ReadOnlyTexture(ScheduledTexture {
                    label: MESH_TEXTURE_LABEL,
                    desc: texture_desc,
                    sample_desc,
                    view_desc,
                    bg_layout_desc: texture_bg_layout_desc,
                    data: Box::new([255, 0, 0, 0]), // red
                }),
            },
            ScheduledBindGroup {
                label: MESH_CAMERA_LABEL,
                form: ScheduledBindGroupType::Uniform(ScheduledUniform {
                    layout: &Camera3d::get_bind_group_layout(ws),
                    buffers: &[ScheduledBuffer {
                        desc: render::wgpu::util::BufferInitDescriptor {
                            label: Some(MESH_CAMERA_LABEL.0),
                            contents: &camera[..],
                            usage: BufferUsages::UNIFORM,
                        },
                    }],
                }),
            },
        ],
        textures: vec![],
    }
    .finalize(ws);

    DynamicRenderPipeline::new(rp, vec![MESH_BUFFER_LABEL])
        .register_shader::<Mesh<MeshVertex>, MeshVertex>(dynamic_mesh_node, ws, alloc)
}
