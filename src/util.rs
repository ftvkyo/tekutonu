use std::f64::consts::{PI, TAU};

use cgmath::Rad;

pub fn normalize_angle(a: Rad<f64>) -> Rad<f64> {
    Rad(a.0 - (TAU * f64::floor((a.0 + PI) / TAU)))
}
