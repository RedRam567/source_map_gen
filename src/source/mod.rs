//! Source entities and higher level structs. See also [`crate::map`].

pub(crate) mod entity;

pub use entity::*;

use crate::{light::GlobalLighting, map::Angles};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
/// RGB with brightness
pub struct ColorBrightness {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: i32,
}

impl ColorBrightness {
    pub const fn new(r: u8, g: u8, b: u8, a: i32) -> Self {
        Self { r, g, b, a }
    }
}

impl Display for ColorBrightness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ColorBrightness { r, g, b, a } = *self;
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
