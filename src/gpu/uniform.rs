use encase::{ShaderType, internal::WriteInto};
use wgpu::util::DeviceExt;

/// GPU uniform buffer with correct `encase` layout and alignment.
pub struct UniformBuffer<T: ShaderType> {
    pub inner: wgpu::Buffer,
    _marker: std::marker::PhantomData<T>,
}

impl<T: ShaderType + WriteInto> UniformBuffer<T> {
    pub fn new(device: &wgpu::Device, label: &str, value: &T) -> Self {
        let bytes = Self::encode(value);
        let inner = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            inner,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, value: &T) {
        queue.write_buffer(&self.inner, 0, &Self::encode(value));
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource<'_> {
        self.inner.as_entire_binding()
    }

    pub fn min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(T::min_size().get() as u64)
            .expect("uniform size must be non-zero")
    }

    fn encode(value: &T) -> Vec<u8> {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(value).expect("uniform encode");
        buffer.into_inner()
    }
}
