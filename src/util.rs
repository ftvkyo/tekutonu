use std::f64::consts::{PI, TAU, FRAC_PI_2};

use cgmath::Rad;

pub fn limit_yaw(y: Rad<f64>) -> Rad<f64> {
    Rad(y.0.clamp(-FRAC_PI_2, FRAC_PI_2))
}

pub fn normalize_angle(a: Rad<f64>) -> Rad<f64> {
    Rad(a.0 - (TAU * f64::floor((a.0 + PI) / TAU)))
}
