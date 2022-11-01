use std::f32::consts::{PI, TAU};

use cgmath::Rad;

pub fn normalize_angle(a: Rad<f32>) -> Rad<f32> {
    Rad(a.0 - (TAU * f32::floor((a.0 + PI) / TAU)))
}
