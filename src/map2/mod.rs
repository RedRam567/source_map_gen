//! Specific representations of a level. See [`crate::map`]

pub(crate) mod entity2;

use std::fmt::Display;

pub use entity2::*;
use spa::SolarPos;

use crate::{light::GlobalLighting, vmf::ToLower};

// TODO:DOCS: TODO:LOC:
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct Angles {
    /// +down/-up, degrees.
    pub pitch: f64,
    /// +left/-right, degrees.
    pub yaw: f64,
    /// +clockwise/-counterclockwise, degrees.
    pub roll: f64,
}

impl Angles {
    // Assumes +Y is north.
    // seems good, checked in hammer and irl (scary)
    pub(crate) fn from_solar_pos(pos: SolarPos) -> Self {
        let pitch = pos.zenith_angle - 90.0;
        // angle right from north (azimuth) to angle left from +X/east (yaw)
        // rem_euclid() means slam into 0..360 range
        let yaw = (270.0 - pos.azimuth).rem_euclid(360.0);
        let roll = 0.0;
        Angles { pitch, yaw, roll }
    }
}

impl Display for Angles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.pitch, self.yaw, self.roll)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
/// RGB with brightness
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: i32,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: i32) -> Self {
        Self { r, g, b, a }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Color {r, g, b, a} = *self;
        write!(f, "{r} {g} {b} {a}")
    }
}

/// All information for global lighting, shadowing and post processing fog, etc.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlobalLightShadows<S> {
    pub light: GlobalLighting,
    pub shadows: ShadowControl<S>,
    pub fog: EnvFogController<S>,
}

// impl<'a> ToLower<'a, [PointEntity; 3]> for GlobalLightShadows {
//     fn into_lower(self) -> [PointEntity; 3] {
//         todo!()
//     }
// }
