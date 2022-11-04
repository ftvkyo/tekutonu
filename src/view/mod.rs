use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool},
    impl_vertex,
    memory::allocator::{MemoryUsage, StandardMemoryAllocator},
};

pub mod instance;
pub use game_view::*;

mod device;
mod framebuffer;
mod game_view;
mod pipeline;
mod render_pass;
mod shaders;
mod swapchain;


// How we are going to give data to the device
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    position: [f32; 3],
}
impl_vertex!(Vertex, position);

pub fn make_vertex_buffer(
    allocator: Arc<StandardMemoryAllocator>,
) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
    let vertices = [
        Vertex {
            position: [-0.5, -0.5, 0.5],
        },
        Vertex {
            position: [0.5, -0.5, 0.5],
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
        },
        Vertex {
            position: [-0.5, -0.5, 1.5],
        },
        Vertex {
            position: [0.5, -0.5, 1.5],
        },
        Vertex {
            position: [0.5, 0.5, 1.5],
        },
        Vertex {
            position: [-0.5, 0.5, 1.5],
        },
        Vertex {
            position: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.1, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.0, 0.1],
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
        },
        Vertex {
            position: [-0.5, -0.5, -0.5],
        },
    ];

    CpuAccessibleBuffer::from_iter(
        &allocator,
        BufferUsage {
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        vertices,
    )
    .unwrap()
}

pub fn make_index_buffer(
    allocator: Arc<StandardMemoryAllocator>,
) -> Arc<CpuAccessibleBuffer<[u16]>> {
    let indices = [
        // Front
        0, 1, 3, 1, 2, 3, // Left
        4, 0, 7, 0, 3, 4, // Back
        5, 4, 6, 4, 7, 6, // Right,
        1, 5, 2, 5, 6, 2, // Top
        0, 4, 1, 1, 4, 5, // Bottom
        2, 7, 3, 7, 2, 6, // Horizontal triangle
        8, 9, 10, // Triangle pointing up, behind us
        11, 12, 13,
    ];

    CpuAccessibleBuffer::from_iter(
        &allocator,
        BufferUsage {
            index_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        indices,
    )
    .unwrap()
}

pub fn make_uniforms_buffer(
    allocator: Arc<StandardMemoryAllocator>,
) -> CpuBufferPool<shaders::vs::ty::Data> {
    CpuBufferPool::<shaders::vs::ty::Data>::new(
        allocator,
        BufferUsage {
            uniform_buffer: true,
            ..BufferUsage::empty()
        },
        MemoryUsage::Upload,
    )
}
