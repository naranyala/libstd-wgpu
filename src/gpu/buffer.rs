use wgpu::util::DeviceExt;

pub fn next_capacity_for_growth(current_capacity: usize, min_capacity: usize) -> usize {
    (current_capacity.max(1) * 2).max(min_capacity.max(1))
}

pub struct Buffer<T> {
    pub inner: wgpu::Buffer,
    capacity: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> Buffer<T> {
    pub fn new(device: &wgpu::Device, label: &str, data: &[T], usage: wgpu::BufferUsages) -> Self {
        let inner = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage,
        });
        Self {
            inner,
            capacity: data.len(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, data: &[T]) {
        debug_assert!(
            data.len() <= self.capacity,
            "write exceeds buffer capacity ({} > {})",
            data.len(),
            self.capacity
        );
        queue.write_buffer(&self.inner, 0, bytemuck::cast_slice(data));
    }

    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.inner.slice(..)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Growable GPU buffer for per-frame instance or dynamic data.
pub struct GpuVec<T> {
    buffer: wgpu::Buffer,
    capacity: usize,
    usage: wgpu::BufferUsages,
    label: String,
    _marker: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> GpuVec<T> {
    pub fn with_capacity(
        device: &wgpu::Device,
        label: impl Into<String>,
        capacity: usize,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let label = label.into();
        let buffer = Self::alloc(device, &label, capacity.max(1), usage);
        Self {
            buffer,
            capacity: capacity.max(1),
            usage,
            label,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        if data.len() > self.capacity {
            self.grow(device, data.len());
        }
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }

    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    fn grow(&mut self, device: &wgpu::Device, min_capacity: usize) {
        self.capacity = next_capacity_for_growth(self.capacity, min_capacity);
        self.buffer = Self::alloc(device, &self.label, self.capacity, self.usage);
    }

    fn alloc(
        device: &wgpu::Device,
        label: &str,
        capacity: usize,
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        let size = (std::mem::size_of::<T>() * capacity) as u64;
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}
