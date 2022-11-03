use cgmath::{Point3, Rad, Vector3};

pub enum GameModelEffect {
    Debug,
    TeleportCamera {
        point: Point3<f64>,
    },
    /// Shift camera relatively to where it is looking.
    /// X is right, Y is down, Z is forward
    ShiftCamera {
        direction: Vector3<f64>,
    },
    AdjustCameraAngles {
        delta_pitch: Rad<f64>,
        delta_yaw: Rad<f64>,
    },
}
