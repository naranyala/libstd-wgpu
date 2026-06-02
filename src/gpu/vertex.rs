pub struct VertexLayoutBuilder {
    array_stride: wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode,
    attributes: Vec<wgpu::VertexAttribute>,
    offset: wgpu::BufferAddress,
    start_location: u32,
}

impl VertexLayoutBuilder {
    pub fn new(stride: wgpu::BufferAddress) -> Self {
        Self {
            array_stride: stride,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Vec::new(),
            offset: 0,
            start_location: 0,
        }
    }

    pub fn start_location(mut self, location: u32) -> Self {
        self.start_location = location;
        self
    }

    pub fn step_mode(mut self, step_mode: wgpu::VertexStepMode) -> Self {
        self.step_mode = step_mode;
        self
    }

    pub fn attribute(mut self, format: wgpu::VertexFormat) -> Self {
        let shader_location = self.start_location + self.attributes.len() as u32;
        self.attributes.push(wgpu::VertexAttribute {
            offset: self.offset,
            shader_location,
            format,
        });
        self.offset += format.size();
        self
    }

    pub fn build(self) -> wgpu::VertexBufferLayout<'static> {
        let attributes = Box::leak(self.attributes.into_boxed_slice());
        wgpu::VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: self.step_mode,
            attributes,
        }
    }
}
