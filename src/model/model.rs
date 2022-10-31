use cgmath::{Point3, Rad};

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f64>,
    pub pitch: Rad<f32>,
    pub yaw: Rad<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 1.0),
            pitch: Rad(0.0),
            yaw: Rad(std::f32::consts::FRAC_PI_2),
        }
    }
}

#[derive(Default)]
pub struct Game {
    pub camera: Camera,
}
