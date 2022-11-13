use bytemuck::{Pod, Zeroable};
use vulkano::{
    impl_vertex,
};


// How we are going to give data to the device
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 3],
}
impl_vertex!(Vertex, position);
