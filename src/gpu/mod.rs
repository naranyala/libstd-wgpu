use std::sync::Arc;
use thiserror::Error;
use winit::window::Window;

pub mod buffer;
pub mod bind_group;
pub mod mesh;
pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod texture;
pub mod uniform;
pub mod vertex;

pub use buffer::{Buffer, GpuVec};
pub use bind_group::{BindGroup, BindGroupBuilder};
pub use mesh::{InstancedMesh, Mesh, QUAD_INDICES};
pub use pipeline::{PipelineBuilder, SimplePipeline};
pub use render_pass::{ColorPassDesc, ColorRenderPass};
pub use texture::Texture;
pub use uniform::UniformBuffer;
pub use vertex::VertexLayoutBuilder;

#[derive(Error, Debug)]
pub enum GpuError {
    #[error("surface lost; reconfigured")]
    SurfaceLost,
    #[error("surface out of memory")]
    OutOfMemory,
    #[error("surface timeout")]
    Timeout,
}

pub struct Context {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    depth: Option<Texture>,
}

pub struct Frame {
    pub output: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
}

impl Context {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("GpuContext Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let mut ctx = Self {
            device,
            queue,
            surface,
            config,
            depth: None,
        };
        ctx.ensure_depth();
        ctx
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.ensure_depth();
        }
    }

    /// (Re)create the depth texture to match the current swapchain size.
    pub fn ensure_depth(&mut self) {
        self.depth = Some(Texture::new_depth(
            &self.device,
            "depth",
            self.config.width,
            self.config.height,
        ));
    }

    pub fn depth_view(&self) -> Option<&wgpu::TextureView> {
        self.depth.as_ref().map(|d| d.depth_view())
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.config.height as f32 / self.config.width as f32
    }

    pub fn get_current_frame(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn create_buffer<T: bytemuck::Pod>(
        &self,
        label: &str,
        data: &[T],
        usage: wgpu::BufferUsages,
    ) -> Buffer<T> {
        Buffer::new(&self.device, label, data, usage)
    }

    pub fn create_uniform_buffer<T: encase::ShaderType + encase::internal::WriteInto>(
        &self,
        label: &str,
        data: &T,
    ) -> UniformBuffer<T> {
        UniformBuffer::new(&self.device, label, data)
    }

    pub fn start_frame(&self) -> Result<Frame, wgpu::SurfaceError> {
        let output = self.get_current_frame()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            });
        Ok(Frame {
            output,
            view,
            encoder,
        })
    }

    pub fn submit_frame(&self, frame: Frame) {
        self.queue.submit(std::iter::once(frame.encoder.finish()));
        frame.output.present();
    }

    pub fn handle_surface_error(&mut self, err: wgpu::SurfaceError) -> Result<(), GpuError> {
        match err {
            wgpu::SurfaceError::Lost => {
                self.resize(self.config.width, self.config.height);
                Ok(())
            }
            wgpu::SurfaceError::OutOfMemory => Err(GpuError::OutOfMemory),
            wgpu::SurfaceError::Timeout => Err(GpuError::Timeout),
            wgpu::SurfaceError::Outdated => Ok(()),
        }
    }
}

impl Frame {
    pub fn begin_color_pass<'a>(
        &'a mut self,
        desc: ColorPassDesc,
        depth_view: Option<&'a wgpu::TextureView>,
    ) -> ColorRenderPass<'a> {
        ColorRenderPass::new(&mut self.encoder, &self.view, desc, depth_view)
    }
}
