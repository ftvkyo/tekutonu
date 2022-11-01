use cgmath::{Rad, Vector3};

pub enum GameModelEffect {
    UpdateCameraPosition {
        delta: Vector3<f64>,
    },
    UpdateCameraLook {
        delta_pitch: Rad<f32>,
        delta_yaw: Rad<f32>,
    },
}
