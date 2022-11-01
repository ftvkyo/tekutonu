use std::f32::consts::{FRAC_PI_2, PI};

use crate::model::Game;


pub struct InputProcessor {}

impl InputProcessor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process_mouse_movement(&self, game: &mut Game, delta: (f64, f64)) {
        const RAD_PER_PX: f32 = FRAC_PI_2 / 90.0;

        // X is pointing right?
        game.camera.yaw.0 -= (delta.0 as f32) * RAD_PER_PX;
        // Y is pointing down
        game.camera.pitch.0 += (delta.1 as f32) * RAD_PER_PX;

        // Bring yaw to the [-PI, PI] range
        while game.camera.yaw.0 > PI {
            game.camera.yaw.0 -= PI * 2.0;
        }
        while game.camera.yaw.0 < -PI {
            game.camera.yaw.0 += PI * 2.0;
        }

        // Stop pitching when it's vertical
        game.camera.pitch.0 = game.camera.pitch.0.min(FRAC_PI_2).max(-FRAC_PI_2);
    }
}
