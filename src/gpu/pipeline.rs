use std::sync::Arc;

pub struct SimplePipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: Option<Arc<wgpu::BindGroup>>,
}

impl SimplePipeline {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        shader_source: &str,
        bind_group_layout: Option<&wgpu::BindGroupLayout>,
        bind_group: Option<Arc<wgpu::BindGroup>>,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SimplePipeline Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let layouts: Vec<&wgpu::BindGroupLayout> = bind_group_layout
            .map(|l| vec![l])
            .unwrap_or_default();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SimplePipeline Layout"),
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SimplePipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
        }
    }
}

pub struct PipelineBuilder<'a> {
    device: &'a wgpu::Device,
    shader_source: &'a str,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    vertex_layouts: Vec<wgpu::VertexBufferLayout<'a>>,
    targets: Vec<Option<wgpu::ColorTargetState>>,
    primitive: wgpu::PrimitiveState,
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
    push_constant_ranges: Vec<wgpu::PushConstantRange>,
    vertex_entry: &'a str,
    fragment_entry: &'a str,
    label: Option<&'a str>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(device: &'a wgpu::Device, shader_source: &'a str) -> Self {
        Self {
            device,
            shader_source,
            bind_group_layouts: Vec::new(),
            vertex_layouts: Vec::new(),
            targets: Vec::new(),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            push_constant_ranges: Vec::new(),
            vertex_entry: "vs_main",
            fragment_entry: "fs_main",
            label: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn vertex_entry(mut self, entry: &'a str) -> Self {
        self.vertex_entry = entry;
        self
    }

    pub fn fragment_entry(mut self, entry: &'a str) -> Self {
        self.fragment_entry = entry;
        self
    }

    pub fn bind_group_layout(mut self, layout: &'a wgpu::BindGroupLayout) -> Self {
        self.bind_group_layouts.push(layout);
        self
    }

    pub fn vertex_layout(mut self, layout: wgpu::VertexBufferLayout<'a>) -> Self {
        self.vertex_layouts.push(layout);
        self
    }

    pub fn color_target(mut self, target: wgpu::ColorTargetState) -> Self {
        self.targets.push(Some(target));
        self
    }

    /// Alpha blending for UI / sprites.
    pub fn blend_alpha(mut self, format: wgpu::TextureFormat) -> Self {
        self.targets.push(Some(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        }));
        self
    }

    /// Disable face culling (typical for 2D).
    pub fn no_cull(mut self) -> Self {
        self.primitive.cull_mode = None;
        self
    }

    pub fn topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.primitive.topology = topology;
        self
    }

    pub fn depth_stencil(mut self, format: wgpu::TextureFormat) -> Self {
        self.depth_stencil = Some(wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });
        self
    }

    pub fn push_constants(
        mut self,
        visibility: wgpu::ShaderStages,
        size_bytes: u32,
    ) -> Self {
        self.push_constant_ranges.push(wgpu::PushConstantRange {
            stages: visibility,
            range: 0..size_bytes,
        });
        self
    }

    pub fn build(self, surface_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: self.label.map(|l| format!("{l}_shader")).as_deref(),
            source: wgpu::ShaderSource::Wgsl(self.shader_source.into()),
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self.label.map(|l| format!("{l}_layout")).as_deref(),
            bind_group_layouts: &self.bind_group_layouts,
            push_constant_ranges: &self.push_constant_ranges,
        });

        let targets = if self.targets.is_empty() {
            vec![Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })]
        } else {
            self.targets
        };

        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: self.vertex_entry,
                buffers: &self.vertex_layouts,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: self.fragment_entry,
                targets: &targets,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            multiview: None,
            cache: None,
        })
    }
}
