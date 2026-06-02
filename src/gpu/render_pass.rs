use super::bind_group::BindGroup;
use glam::Vec4;

/// Clear color and pass metadata for a single color (+ optional depth) pass.
pub struct ColorPassDesc {
    pub clear: wgpu::Color,
    pub label: Option<&'static str>,
}

impl ColorPassDesc {
    pub fn clear_rgb(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            clear: wgpu::Color {
                r: f64::from(r),
                g: f64::from(g),
                b: f64::from(b),
                a: f64::from(a),
            },
            label: None,
        }
    }

    pub fn clear_vec4(color: Vec4) -> Self {
        Self::clear_rgb(color.x, color.y, color.z, color.w)
    }

    pub fn label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }
}

/// Wrapper around an active `wgpu::RenderPass` with convenience draw helpers.
pub struct ColorRenderPass<'a> {
    pass: wgpu::RenderPass<'a>,
}

impl<'a> ColorRenderPass<'a> {
    pub fn new(
        encoder: &'a mut wgpu::CommandEncoder,
        color_view: &'a wgpu::TextureView,
        desc: ColorPassDesc,
        depth_view: Option<&'a wgpu::TextureView>,
    ) -> Self {
        let depth_stencil_attachment = depth_view.map(|view| wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        });

        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: desc.label,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(desc.clear),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        Self { pass }
    }

    pub fn set_pipeline(&mut self, pipeline: &wgpu::RenderPipeline) {
        self.pass.set_pipeline(pipeline);
    }

    pub fn set_bind_group(&mut self, index: u32, bind_group: &BindGroup) {
        self.pass.set_bind_group(index, &bind_group.inner, &[]);
    }

    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &wgpu::Buffer, offset: u64) {
        self.pass
            .set_vertex_buffer(slot, buffer.slice(offset..));
    }

    pub fn set_vertex_buffer_slice(
        &mut self,
        slot: u32,
        slice: wgpu::BufferSlice<'_>,
    ) {
        self.pass.set_vertex_buffer(slot, slice);
    }

    /// Procedural draw (e.g. triangle from `vertex_index` only).
    pub fn draw(&mut self, vertices: std::ops::Range<u32>, instances: std::ops::Range<u32>) {
        self.pass.draw(vertices, instances);
    }

    /// Indexed mesh draw.
    pub fn draw_indexed(
        &mut self,
        indices: std::ops::Range<u32>,
        base_vertex: i32,
        instances: std::ops::Range<u32>,
    ) {
        self.pass.draw_indexed(indices, base_vertex, instances);
    }

    pub fn set_index_buffer_u32(&mut self, slice: wgpu::BufferSlice<'_>) {
        self.pass.set_index_buffer(slice, wgpu::IndexFormat::Uint32);
    }

    /// Draw indexed geometry already bound with `set_vertex_buffer_slice` + `set_index_buffer_u32`.
    pub fn draw_indexed_mesh(
        &mut self,
        index_count: u32,
        instances: std::ops::Range<u32>,
    ) {
        self.draw_indexed(0..index_count, 0, instances);
    }

    /// Pipeline + optional bind groups + non-indexed draw.
    pub fn draw_simple(
        &mut self,
        pipeline: &wgpu::RenderPipeline,
        bind_groups: &[&BindGroup],
        vertices: std::ops::Range<u32>,
        instances: std::ops::Range<u32>,
    ) {
        self.set_pipeline(pipeline);
        for (i, bg) in bind_groups.iter().enumerate() {
            self.set_bind_group(i as u32, bg);
        }
        self.draw(vertices, instances);
    }
}
