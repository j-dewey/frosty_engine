use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

pub struct GPUBindings {
    instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
}

impl GPUBindings {
    pub async fn new(desired_surface: Option<&wgpu::Surface<'_>>) -> Self {
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: desired_surface,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let mut limits = wgpu::Limits::default();
        limits.max_binding_array_elements_per_shader_stage = 8;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::default()
                        | wgpu::Features::TEXTURE_BINDING_ARRAY // for texture array
                        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING, // for texture array
                required_limits: limits,
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        Self {
            instance,
            device,
            adapter,
            queue,
        }
    }

    pub async fn new_from_window<'a>(window: &'a Window) -> (Self, wgpu::Surface<'a>) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let mut limits = wgpu::Limits::default();
        limits.max_binding_array_elements_per_shader_stage = 8;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::default()
                        | wgpu::Features::TEXTURE_BINDING_ARRAY // for texture array
                        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING, // for texture array
                required_limits: limits,
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        (
            Self {
                instance,
                device,
                adapter,
                queue,
            },
            surface,
        )
    }

    pub fn load_vertex_buffer(&self, label: &str, verts: &[u8]) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(verts),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
    }

    pub fn load_index_buffer(&self, label: &str, indices: &[u8]) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            })
    }
}

pub struct WindowState<'a> {
    surface: wgpu::Surface<'a>,
    pub bindings: GPUBindings,
    pub config: wgpu::SurfaceConfiguration,
    pub window: &'a Window,
    pub size: PhysicalSize<u32>,
}

impl<'a> WindowState<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let (bindings, surface) = GPUBindings::new_from_window(window).await;

        let surface_caps = surface.get_capabilities(&bindings.adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 10,
        };
        surface.configure(&bindings.device, &config);

        Self {
            window,
            surface,
            bindings,
            config,
            size,
        }
    }

    #[inline]
    pub fn load_vertex_buffer(&self, label: &str, verts: &[u8]) -> wgpu::Buffer {
        self.bindings.load_vertex_buffer(label, verts)
    }

    #[inline]
    pub fn load_index_buffer(&self, label: &str, indices: &[u8]) -> wgpu::Buffer {
        self.bindings.load_index_buffer(label, indices)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.bindings.device, &self.config);
        }
    }

    pub fn prep_render(
        &mut self,
    ) -> Result<
        (
            wgpu::TextureView,
            wgpu::CommandEncoder,
            wgpu::SurfaceTexture,
        ),
        wgpu::SurfaceError,
    > {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });

        let encoder =
            self.bindings
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        Ok((view, encoder, output))
    }

    pub fn post_render(&mut self, encoder: wgpu::CommandEncoder, output: wgpu::SurfaceTexture) {
        self.bindings
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
