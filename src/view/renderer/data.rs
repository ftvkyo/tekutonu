use bytemuck::{Pod, Zeroable};
use cgmath::{Point3, Vector3};
use vulkano::impl_vertex;


// How we are going to give data to the device
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 3],
}
impl_vertex!(Vertex, position);

impl From<Point3<f32>> for Vertex {
    fn from(v: Point3<f32>) -> Self {
        Self {
            position: [v.x, v.y, v.z],
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Normal {
    pub normal: [f32; 3],
}
impl_vertex!(Normal, normal);

impl From<Vector3<f32>> for Normal {
    fn from(v: Vector3<f32>) -> Self {
        Self {
            normal: [v.x, v.y, v.z],
        }
    }
}
