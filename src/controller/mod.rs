use std::f32::consts::{FRAC_PI_2, PI};

use winit::{event::VirtualKeyCode, event_loop::ControlFlow};

use crate::{model::Game, view::GameView};


pub struct GameInput {}

impl GameInput {
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

    pub fn process_keyboard_input(&self, game: &mut Game, view: &GameView, key: VirtualKeyCode, control_flow: &mut ControlFlow) {
        use winit::event::VirtualKeyCode as kc;
        let result = match key {
            kc::Escape => {
                control_flow.set_exit();
                Ok(())
            },
            kc::L => {
                view.set_cursor_hidden(true);
                view.set_cursor_locked(true)
            },
            kc::U => {
                view.set_cursor_hidden(false);
                view.set_cursor_locked(false)
            },
            _ => Ok(()),
        };

        if let Err(err) = result {
            println!("error: {}", err);
        }
    }
}
