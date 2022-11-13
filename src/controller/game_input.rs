use std::{collections::HashSet, f64::consts::FRAC_PI_2};

use cgmath::{Rad, Vector3, Zero};
use winit::{
    event::{ElementState, VirtualKeyCode},
    event_loop::ControlFlow,
};

use crate::model::effect::GameModelEffect;


#[derive(Default)]
pub struct GameInput {
    keys_held: HashSet<VirtualKeyCode>,
}

impl GameInput {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn mouse_movement(&self, delta: (f64, f64)) -> GameModelEffect {
        const RAD_PER_PX: f64 = FRAC_PI_2 / 90.0;

        GameModelEffect::AdjustCameraAngles {
            delta_pitch: Rad(delta.1 * RAD_PER_PX),
            delta_yaw: Rad(delta.0 * RAD_PER_PX),
        }
    }

    pub fn keyboard(
        &mut self,
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
            (Tab, Released) => Some(GameModelEffect::Debug),
            (W | A | S | D | R | F, Pressed) => {
                self.keys_held.insert(key);
                None
            },
            (W | A | S | D | R | F, Released) => {
                self.keys_held.remove(&key);
                None
            },
            _ => None,
        }
    }

    fn keyboard_held(key: &VirtualKeyCode) -> Option<GameModelEffect> {
        use winit::event::VirtualKeyCode::*;

        match key {
            W => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(0.0, 0.0, 0.05),
            }),
            A => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(-0.05, 0.0, 0.0),
            }),
            S => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(0.0, 0.0, -0.05),
            }),
            D => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(0.05, 0.0, 0.0),
            }),
            R => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(0.0, 0.05, 0.0),
            }),
            F => Some(GameModelEffect::ShiftCamera {
                direction: Vector3::new(0.0, -0.05, 0.0),
            }),
            _ => None,
        }
    }

    pub fn tick(&self) -> Option<GameModelEffect> {
        let mut camera_shift_acc = None;

        for key in self.keys_held.iter() {
            match Self::keyboard_held(key) {
                Some(GameModelEffect::ShiftCamera { direction }) => {
                    camera_shift_acc =
                        Some(camera_shift_acc.unwrap_or_else(Vector3::zero) + direction);
                },
                _ => (),
            }
        }

        camera_shift_acc.map(|direction| GameModelEffect::ShiftCamera { direction })
    }
}
