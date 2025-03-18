use std::cell::RefCell;

use crate::query::{DynQuery, Query};
use crate::{Spawner, MASTER_THREAD};

use super::{layout::ShaderNodeLayout, GivesBindGroup};
use frosty_alloc::{DataAccess, FrostyAllocatable};
use render::shader::{ShaderDefinition, ShaderGroup};
use render::vertex::Vertex;
use render::window_state::WindowState;
use render::{mesh::MeshyObject, shader::Shader, wgpu};

pub struct ShaderNode<M: MeshyObject + FrostyAllocatable> {
    //cache: Statics<'a>,
    meshes: Query<M>,
    bind_groups: RefCell<DynQuery<dyn GivesBindGroup + 'static>>,
    shader: Shader,
}

impl<M> ShaderNode<M>
where
    M: MeshyObject + FrostyAllocatable,
{
    pub fn new<'a, V: Vertex>(
        mut details: ShaderNodeLayout<'a>,
        ws: &WindowState,
        spawner: &Spawner,
    ) -> Self {
        let mut bg_layouts = Vec::new();
        loop {
            if let Some(bg_holder) = details.bind_groups.next() {
                bg_layouts.push(
                    bg_holder
                        .get_access(MASTER_THREAD)
                        .expect("Bindgroup object deleted before shader init")
                        .as_ref()
                        .get_bind_group_layout(ws),
                );
            } else {
                break;
            }
        }
        details.bind_groups.reset();

        // need to convert Vec<wgpu::BindGroupLayout> into Vec<&wgpu::BindGroupLayout>
        // for pipeline layout
        let bg_layout_references: Vec<&wgpu::BindGroupLayout> =
            bg_layouts.iter().map(|t| t).collect();

        let shader = ShaderDefinition {
            shader_source: details.source,
            bg_layouts: &bg_layout_references[..],
            const_ranges: &[],
            vertex_desc: V::desc(),
            blend_state: Some(wgpu::BlendState::REPLACE),
            depth_stencil: details.depth_stencil,
            depth_buffer: details.depth_buffer,
            primitive_state: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
        }
        .finalize(&ws.device, &ws.config);

        Self {
            meshes: spawner
                .get_query()
                .expect("Cannot init shader before Mesh registered to spawner"),
            bind_groups: RefCell::new(details.bind_groups),
            shader,
        }
    }

    pub fn init_render_fn(self) -> Box<dyn Fn(Query<u8>, &mut WindowState) + 'static>
    where
        M: 'static,
    {
        Box::new(move |meshes: Query<u8>, ws: &mut WindowState| {
            self.draw_meshes(meshes, ws);
        })
    }

    pub fn draw(
        &self,
        mesh: ShaderGroup,
        bind_groups: &[&wgpu::BindGroup],
        encoder: &mut wgpu::CommandEncoder,
    ) {
        //self.shader.render(&[&mesh], bind_groups, encoder, view);
    }

    pub fn draw_meshes(
        &self,
        dynamic_meshes: Query<u8>,
        ws: &mut WindowState,
    ) -> Result<(), wgpu::SurfaceError> {
        let (view, mut encoder, out) = ws.prep_render()?;
        let converted_query: Vec<DataAccess<M>> = unsafe {
            dynamic_meshes
                .cast::<M>()
                .as_slice()
                .expect("Failed to cast mesh query into ObjectHandle slice")
                .iter()
                .filter_map(|h| h.cast_clone::<M>().get_access(MASTER_THREAD))
                .collect()
        };
        let shader_groups: Vec<ShaderGroup> = converted_query
            .iter()
            .map(|m| m.as_ref().get_shader_group())
            .collect();
        let bind_groups = self
            .bind_groups
            .borrow_mut()
            .into_iter()
            .filter_map(|mut c| Some(c.get_access(MASTER_THREAD)?.as_ref().get_bind_group(ws)))
            .collect::<Vec<wgpu::BindGroup>>();

        self.shader.render(
            &shader_groups[..],
            &[], //bind_groups[..],
            &mut encoder,
            &view,
            None,
        );

        Ok(())
    }
}
