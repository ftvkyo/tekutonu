use std::f64::consts::FRAC_PI_2;

use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3};
use tracing::instrument;

use super::{
    block::{Block, BlockKind},
    consts,
    effect::GameModelEffect,
    region::Region,
};
use crate::util::normalize_angle;

pub struct Camera {
    /// Y is up, opposite to vulkan
    pub position: Point3<f64>,
    /// Pitch from XZ towards negative Y
    pub pitch: Rad<f64>,
    /// Yaw from positive Z towards positive X
    pub yaw: Rad<f64>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.5, 0.0),
            pitch: Rad(0.0),
            yaw: Rad(0.0),
        }
    }
}

impl std::fmt::Debug for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = self.position;
        let l = self.get_look();

        f.write_str("Camera:\n")?;
        f.write_fmt(format_args!(
            "  position: {:.2} {:.2} {:.2}\n",
            p.x, p.y, p.z
        ))?;
        f.write_fmt(format_args!(
            "  look    : {:.2} {:.2} {:.2}\n",
            l.x, l.y, l.z
        ))?;
        f.write_fmt(format_args!("    yaw   : {:.3} rad\n", self.yaw.0))?;
        f.write_fmt(format_args!("    pitch : {:.3} rad\n", self.pitch.0))?;

        Ok(())
    }
}

impl Camera {
    pub fn get_look(&self) -> Vector3<f64> {
        // When pitch is higher, Y is lower because Y is directed down
        let direction_y = f64::sin(-self.pitch.0);

        // Scale horizontal coordinates down to how much they matter based on the pitch
        let horizontal_weight = f64::cos(self.pitch.0);

        // Angle is from Z towards X, so the trig functions are swapped
        let direction_x = f64::sin(self.yaw.0) * horizontal_weight;
        let direction_z = f64::cos(self.yaw.0) * horizontal_weight;

        Vector3::new(direction_x, direction_y, direction_z).normalize()
    }

    pub fn get_pitch_axis(&self) -> Vector3<f64> {
        let look = self.get_look();

        // Rotate the projection of the look vector on the XZ plane PI/2 from positive Z
        // towards positive X
        Vector3::new(look.z, 0.0, -look.x).normalize()
    }

    /// `v` is relative to the camera look. Get `v'` in real world.
    ///
    /// self.camera_to_world(self.get_look()) === [0.0, 0.0, 1.0]
    pub fn camera_to_world(&self, v: Vector3<f64>) -> Vector3<f64> {
        let yaw = Matrix4::from_angle_y(self.yaw);
        let pitch = Matrix4::from_axis_angle(self.get_pitch_axis(), self.pitch);

        // Pitch is on the left because axis for the pitch rotation is dependent on
        // look. Not sure how to explain it properly, but basically in other
        // order it would cause inconsistent transformations depending on where
        // the camera is looking.
        (pitch * yaw * v.extend(1.0)).truncate()
    }
}

pub struct GameModel {
    pub camera: Camera,
    pub world: Region,
}

impl Default for GameModel {
    #[instrument]
    fn default() -> Self {
        let mut world = Region::default();

        let s = Block {
            kind: BlockKind::Stone,
        };

        let c = world.get_chunk_mut([0, 0, 0]);

        c.set_block([0, 0, 0], s);

        // Orientation
        c.set_block([3, 3, 3], s);
        c.set_block([3, 3, 4], s);
        c.set_block([3, 4, 3], s);
        c.set_block([4, 3, 3], s);

        // Smileyface
        c.set_block([1, 2, 8], s);
        c.set_block([2, 1, 8], s);
        c.set_block([3, 1, 8], s);
        c.set_block([4, 1, 8], s);
        c.set_block([5, 2, 8], s);
        c.set_block([2, 4, 8], s);
        c.set_block([4, 4, 8], s);

        for y in 0..consts::CHUNK_Y_BLOCKS {
            for z in 0..consts::CHUNK_Z_BLOCKS {
                c.set_block([consts::CHUNK_X_BLOCKS - 1, y, z], s);
            }
        }

        Self {
            camera: Default::default(),
            world,
        }
    }
}

impl GameModel {
    pub fn apply_effect(&mut self, effect: GameModelEffect) {
        use GameModelEffect::*;

        match effect {
            Debug => {
                println!("Camera: {:#?}", self.camera);
            },
            TeleportCamera { point } => {
                self.camera.position = point;
            },
            AdjustCameraAngles {
                // Increasing the pitch should make us look more up
                delta_pitch,
                // Increasing the yaw should make us look more to the right
                delta_yaw,
            } => {
                self.camera.pitch = Rad((self.camera.pitch + delta_pitch)
                    .0
                    .clamp(-FRAC_PI_2, FRAC_PI_2));
                self.camera.yaw = normalize_angle(self.camera.yaw + delta_yaw);
            },
            ShiftCamera { direction } => {
                let movement = self.camera.camera_to_world(direction);
                self.camera.position += movement;
            },
        }
    }
}

#[cfg(test)]
mod test {
    use cgmath::assert_relative_eq;

    use super::Camera;

    #[test]
    fn camera_to_world() {
        let c = Camera::default();

        let v = c.camera_to_world(c.get_look());

        assert_relative_eq!(v.x, 0.0);
        assert_relative_eq!(v.y, 0.0);
        assert_relative_eq!(v.z, 1.0);
    }
}
