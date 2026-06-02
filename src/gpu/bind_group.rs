use super::buffer::Buffer;
use super::texture::Texture;
use super::uniform::UniformBuffer;
use encase::{ShaderType, internal::WriteInto};

pub struct BindGroup {
    pub layout: wgpu::BindGroupLayout,
    pub inner: wgpu::BindGroup,
}

pub struct BindGroupBuilder<'a> {
    device: &'a wgpu::Device,
    entries: Vec<wgpu::BindGroupEntry<'a>>,
    layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    label: Option<&'a str>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self {
            device,
            entries: Vec::new(),
            layout_entries: Vec::new(),
            label: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn binding(
        mut self,
        binding: u32,
        resource: wgpu::BindingResource<'a>,
        visibility: wgpu::ShaderStages,
        ty: wgpu::BindingType,
    ) -> Self {
        self.entries.push(wgpu::BindGroupEntry { binding, resource });
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count: None,
        });
        self
    }

    /// `encase`-aligned uniform buffer (`UniformBuffer<T>`).
    pub fn uniform<T: ShaderType + WriteInto>(
        self,
        binding: u32,
        buffer: &'a UniformBuffer<T>,
        visibility: wgpu::ShaderStages,
    ) -> Self {
        let ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(UniformBuffer::<T>::min_binding_size()),
        };
        self.binding(binding, buffer.binding_resource(), visibility, ty)
    }

    /// Plain POD uniform buffer (legacy / simple cases).
    pub fn uniform_pod<T: bytemuck::Pod>(
        self,
        binding: u32,
        buffer: &'a Buffer<T>,
        visibility: wgpu::ShaderStages,
    ) -> Self {
        let ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        self.binding(binding, buffer.inner.as_entire_binding(), visibility, ty)
    }

    pub fn storage_read_only<T: bytemuck::Pod>(
        self,
        binding: u32,
        buffer: &'a Buffer<T>,
        visibility: wgpu::ShaderStages,
    ) -> Self {
        let ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        self.binding(binding, buffer.inner.as_entire_binding(), visibility, ty)
    }

    pub fn storage<T: bytemuck::Pod>(
        self,
        binding: u32,
        buffer: &'a Buffer<T>,
        visibility: wgpu::ShaderStages,
    ) -> Self {
        let ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        self.binding(binding, buffer.inner.as_entire_binding(), visibility, ty)
    }

    /// Binds a 2D texture at `binding` and a sampler at `binding + 1`.
    pub fn texture(
        self,
        binding: u32,
        texture: &'a Texture,
        visibility: wgpu::ShaderStages,
    ) -> Self {
        let sampler = texture
            .sampler
            .as_ref()
            .expect("Texture::sampler required for texture bind group");

        self.binding(
            binding,
            wgpu::BindingResource::TextureView(&texture.view),
            visibility,
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
        )
        .binding(
            binding + 1,
            wgpu::BindingResource::Sampler(sampler),
            visibility,
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        )
    }

    pub fn build(self) -> BindGroup {
        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: self.label.map(|l| format!("{l}_layout")).as_deref(),
            entries: &self.layout_entries,
        });

        let inner = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.label,
            layout: &layout,
            entries: &self.entries,
        });

        BindGroup { layout, inner }
    }
}
