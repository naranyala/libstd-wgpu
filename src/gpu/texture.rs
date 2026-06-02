pub struct Texture {
    pub inner: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: Option<wgpu::Sampler>,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
}

impl Texture {
    pub fn new(
        device: &wgpu::Device,
        label: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        create_sampler: bool,
    ) -> Self {
        let inner = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: usage | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = inner.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = if create_sampler {
            Some(device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some(&format!("{label}_sampler")),
                ..Default::default()
            }))
        } else {
            None
        };

        Self {
            inner,
            view,
            sampler,
            width,
            height,
            format,
        }
    }

    /// Depth buffer for 3D or layered rendering.
    pub fn new_depth(
        device: &wgpu::Device,
        label: &str,
        width: u32,
        height: u32,
    ) -> Self {
        Self::new(
            device,
            label,
            width,
            height,
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
            false,
        )
    }

    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn write_rgba8(&self, queue: &wgpu::Queue, width: u32, height: u32, rgba: &[u8]) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.inner,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}
