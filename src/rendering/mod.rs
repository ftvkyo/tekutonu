use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
    impl_vertex,
};

pub mod instance;
pub mod renderer;

mod device;
mod framebuffer;
mod pipeline;
mod render_pass;
mod shaders;
mod swapchain;


// How we are going to give data to the device
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    position: [f32; 2],
}
impl_vertex!(Vertex, position);

pub fn make_vertex_buffer(device: Arc<Device>) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
    let vertices = [
        Vertex {
            position: [-0.5, -0.5],
        },
        Vertex {
            position: [-0.5, 0.5],
        },
        Vertex {
            position: [0.5, 0.5],
        },
        Vertex {
            position: [0.5, -0.5],
        },
    ];

    CpuAccessibleBuffer::from_iter(
        device,
        BufferUsage {
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        vertices,
    )
    .unwrap()
}

pub fn make_index_buffer(device: Arc<Device>) -> Arc<CpuAccessibleBuffer<[u16]>> {
    let indices = [0, 1, 2, 0, 2, 3];

    CpuAccessibleBuffer::from_iter(
        device,
        BufferUsage {
            index_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        indices,
    )
    .unwrap()
}
