use std::f32::consts::FRAC_PI_2;

use cgmath::{Point3, Rad};
use tracing::instrument;

use super::{
    blocks::{Block, BlockKind, Region},
    effect::GameModelEffect,
};
use crate::util::normalize_angle;

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

pub struct GameModel {
    pub camera: Camera,
    pub world: Region,
}

impl Default for GameModel {
    #[instrument]
    fn default() -> Self {
        let mut world = Region::new();

        world.get_chunk_mut([0, 0, 0]).set_block(
            [0, 0, 0],
            Block {
                kind: BlockKind::Stone,
            },
        );

        Self {
            camera: Default::default(),
            world,
        }
    }
}

impl GameModel {
    pub fn apply_effect(&mut self, effect: GameModelEffect) {
        match effect {
            GameModelEffect::UpdateCameraPosition { delta } => {
                self.camera.position += delta;
            },
            GameModelEffect::UpdateCameraLook {
                delta_pitch,
                delta_yaw,
            } => {
                self.camera.pitch = Rad((self.camera.pitch + delta_pitch)
                    .0
                    .clamp(-FRAC_PI_2, FRAC_PI_2));
                self.camera.yaw = normalize_angle(self.camera.yaw + delta_yaw);
            },
        }
    }
}
