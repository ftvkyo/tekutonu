use bytemuck::{Pod, Zeroable};
use cgmath::Point3;
use vulkano::impl_vertex;


// How we are going to give data to the device
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub v_position: [f32; 3],
}
impl_vertex!(Vertex, v_position);

impl From<Point3<f32>> for Vertex {
    fn from(v: Point3<f32>) -> Self {
        Self {
            v_position: [v.x, v.y, v.z],
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Light {
    pub v_light: f32,
}
impl_vertex!(Light, v_light);

impl From<u8> for Light {
    fn from(light: u8) -> Self {
        Self {
            v_light: (light as f32) / 15.,
        }
    }
}
