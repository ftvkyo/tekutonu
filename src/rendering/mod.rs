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
            position: [-0.5, -0.25],
        },
        Vertex {
            position: [0.0, 0.5],
        },
        Vertex {
            position: [0.25, -0.1],
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
