use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool},
    impl_vertex,
    memory::allocator::{MemoryUsage, StandardMemoryAllocator},
};

use crate::model::GameModel;


// How we are going to give data to the device
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    position: [f32; 3],
}
impl_vertex!(Vertex, position);

pub fn make_vertex_and_index_buffers(
    allocator: Arc<StandardMemoryAllocator>,
    game: &GameModel,
) -> (
    Arc<CpuAccessibleBuffer<[Vertex]>>,
    Arc<CpuAccessibleBuffer<[u16]>>,
) {
    let (v, i) = game
        .world
        .get_chunk([0, 0, 0])
        .get_render_data([0.0, 0.0, 0.0]);

    let v = CpuAccessibleBuffer::from_iter(
        &allocator,
        BufferUsage {
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        v.into_iter().map(|v| Vertex { position: v }),
    )
    .unwrap();

    let i = CpuAccessibleBuffer::from_iter(
        &allocator,
        BufferUsage {
            index_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        i.into_iter().map(|i| i as u16),
    )
    .unwrap();

    (v, i)
}

pub fn make_uniforms_buffer(
    allocator: Arc<StandardMemoryAllocator>,
) -> CpuBufferPool<super::shaders::vs::ty::Data> {
    CpuBufferPool::<super::shaders::vs::ty::Data>::new(
        allocator,
        BufferUsage {
            uniform_buffer: true,
            ..BufferUsage::empty()
        },
        MemoryUsage::Upload,
    )
}
