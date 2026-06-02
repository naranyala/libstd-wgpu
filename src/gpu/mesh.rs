use super::buffer::{Buffer, GpuVec};
use super::render_pass::ColorRenderPass;

/// Two triangles (CCW) covering corners 0–3 of a unit quad in [-0.5, 0.5]².
pub const QUAD_INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

/// Static vertex (+ optional index) geometry.
pub struct Mesh<V: bytemuck::Pod> {
    pub vertices: Buffer<V>,
    pub indices: Option<Buffer<u32>>,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl<V: bytemuck::Pod> Mesh<V> {
    pub fn new(device: &wgpu::Device, label: &str, vertices: &[V], indices: Option<&[u32]>) -> Self {
        let vertex_buffer = Buffer::new(
            device,
            &format!("{label}_vb"),
            vertices,
            wgpu::BufferUsages::VERTEX,
        );

        let (index_buffer, index_count) = if let Some(indices) = indices {
            (
                Some(Buffer::new(
                    device,
                    &format!("{label}_ib"),
                    indices,
                    wgpu::BufferUsages::INDEX,
                )),
                indices.len() as u32,
            )
        } else {
            (None, 0)
        };

        Self {
            vertices: vertex_buffer,
            indices: index_buffer,
            vertex_count: vertices.len() as u32,
            index_count,
        }
    }

    pub fn draw<'a>(&'a self, pass: &mut ColorRenderPass<'a>, slot: u32) {
        pass.set_vertex_buffer_slice(slot, self.vertices.slice());
        if let Some(ref ib) = self.indices {
            pass.set_index_buffer_u32(ib.slice());
            pass.draw_indexed_mesh(self.index_count, 0..1);
        } else {
            pass.draw(0..self.vertex_count, 0..1);
        }
    }
}

/// One shared mesh (e.g. quad) drawn many times via instancing.
pub struct InstancedMesh<V: bytemuck::Pod, I: bytemuck::Pod> {
    pub mesh: Mesh<V>,
    pub instances: GpuVec<I>,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl<V: bytemuck::Pod, I: bytemuck::Pod> InstancedMesh<V, I> {
    pub fn new(
        device: &wgpu::Device,
        label: &str,
        vertices: &[V],
        indices: Option<&[u32]>,
        instance_capacity: usize,
    ) -> Self {
        let mesh = Mesh::new(device, label, vertices, indices);
        let instances = GpuVec::with_capacity(
            device,
            format!("{label}_instances"),
            instance_capacity,
            wgpu::BufferUsages::VERTEX,
        );
        Self {
            vertex_count: mesh.vertex_count,
            index_count: mesh.index_count,
            mesh,
            instances,
        }
    }

    pub fn write_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[I],
    ) {
        self.instances.write(device, queue, data);
    }

    pub fn draw<'a>(
        &'a self,
        pass: &mut ColorRenderPass<'a>,
        vertex_slot: u32,
        instance_slot: u32,
        instance_count: u32,
    ) {
        pass.set_vertex_buffer_slice(vertex_slot, self.mesh.vertices.slice());
        pass.set_vertex_buffer_slice(instance_slot, self.instances.slice());

        if self.index_count > 0 {
            let ib = self
                .mesh
                .indices
                .as_ref()
                .expect("index_count > 0 requires index buffer");
            pass.set_index_buffer_u32(ib.slice());
            pass.draw_indexed(0..self.index_count, 0, 0..instance_count);
        } else {
            pass.draw(0..self.vertex_count, 0..instance_count);
        }
    }
}
