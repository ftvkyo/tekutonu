use std::f64::consts::FRAC_PI_2;

use cgmath::{Rad, Vector3};
use winit::{
    event::{ElementState, VirtualKeyCode},
    event_loop::ControlFlow,
};

use crate::{model::effect::GameModelEffect, view::GameView};


pub struct GameInput {}

impl GameInput {
    pub fn new() -> Self {
        Self {}
    }

    pub fn mouse_movement(&self, delta: (f64, f64)) -> GameModelEffect {
        const RAD_PER_PX: f64 = FRAC_PI_2 / 90.0;

        GameModelEffect::AdjustCameraAngles {
            delta_pitch: Rad(delta.1 * RAD_PER_PX),
            delta_yaw: Rad(delta.0 * RAD_PER_PX),
        }
    }

    pub fn keyboard(
        &self,
        _view: &GameView,
        key: VirtualKeyCode,
        state: ElementState,
        control_flow: &mut ControlFlow,
    ) -> Option<GameModelEffect> {
        use winit::event::{ElementState::*, VirtualKeyCode::*};


        match (key, state) {
            (Escape, Released) => {
                control_flow.set_exit();
                None
            },
            (W, Released) => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(0.0, 0.0, 0.05),
            }),
            (A, Released) => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(-0.05, 0.0, 0.0),
            }),
            (S, Released) => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(0.0, 0.0, -0.05),
            }),
            (D, Released) => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(0.05, 0.0, 0.0),
            }),
            (R, Released) => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(0.0, 0.05, 0.0),
            }),
            (F, Released) => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(0.0, -0.05, 0.0),
            }),
            (Tab, Released) => Some(GameModelEffect::Debug),
            _ => None,
        }
    }
}
