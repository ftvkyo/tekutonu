use std::f32::consts::FRAC_PI_2;

use cgmath::Rad;
use winit::{event::VirtualKeyCode, event_loop::ControlFlow};

use crate::{model::effect::GameModelEffect, view::GameView};


pub struct GameInput {}

impl GameInput {
    pub fn new() -> Self {
        Self {}
    }

    pub fn mouse_movement(&self, delta: (f64, f64)) -> GameModelEffect {
        const RAD_PER_PX: f32 = FRAC_PI_2 / 90.0;

        GameModelEffect::UpdateCameraLook {
            delta_pitch: Rad((delta.1 as f32) * RAD_PER_PX),
            delta_yaw: Rad(-(delta.0 as f32) * RAD_PER_PX),
        }
    }

    pub fn process_keyboard_input(
        &self,
        view: &GameView,
        key: VirtualKeyCode,
        control_flow: &mut ControlFlow,
    ) {
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
