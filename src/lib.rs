//! # Info
//!
//! In this libray, the 3d coordinate grid is right handed Z up. +X is right, +Y is forward, +Z is up.
//! Hammer, Source, Blender, Math are all right handed Z up.
//! (Forward direction is inconsistent, -X is "the convention" in math at least)
//! <https://en.wikipedia.org/wiki/Cartesian_coordinate_system#Notations_and_conventions>
//!
//! # Definitions
//!
//! - East, Right face: The direction or face closest to +X.
//! - West, Left face: The direction or face closest to -X.
//! - North, Front face: The direction or face closest to +Y.
//! - South, Back face: The direction or face closest to -Y.
//! - Top face: The direction or face closest to +Z.
//! - Bottom face: The direction or face closest to -Z.
//! - Width how long on the X axis.
//! - Length how long on the Y axis.
//! - Height how long on the Z axis.
// TODO: change forward axis??

#![allow(clippy::bool_assert_comparison)] // like bro chill
#![allow(clippy::zero_prefixed_literal)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::semicolon_if_nothing_returned)]

// #[deprecated]
pub mod generation;
pub mod light;
pub mod map;
pub mod source;
pub mod vmf;
// pub mod scripting;

pub mod prelude {
    pub use crate::map::solid::*;
    pub use crate::map::texture::*;
    pub use crate::map::vector::*;
    pub(crate) use crate::StrType;
}

/// String type for the library. Might change or be in-lined.
pub(crate) type StrType<'a> = std::borrow::Cow<'a, str>;

// preferred rust group order (pub first in the same group)
// extern
// mod
// inline mod
// use
// const, static, type
// struct, enum, union
// traits
// impl
// fn
// mod test {}

// MACROS???

// normal slope stair default

// my representation of the world
// trait for converting to vmf part
// others can do trait to convert to their map formats

// https://en.wikipedia.org/wiki/Cross_product#Matrix_notation
// https://en.wikipedia.org/wiki/Dot_product
// https://en.wikipedia.org/wiki/Rotation_matrix#In_three_dimensions

// house region with room regions, negative space is hallways? also hallways

// get large super region: city, subway, hostital
//  random walk room builder with sub rooms: house, hallway
//

// outside:
//  unconnected walls = extend a tile and add decor and skybox
// add hallways and rooms with wave collaspe "directed" toward goal
//  nah just do smth similar with wandering "room builder"

// no mercy:
// house region
//  roof
//  rooms floors etc
// city region
//  alley
//  alley rooms
//  negative is infected houses
// city region
//  3 road regions, cull large negative flow
//   branching alleys
//  office floor house
//  warehousey house
//  subway house

// dustbowl
// spawn region
// dustbowl region 1
//  trench
//  houses
// dustbowl connector
// dustbowl region 1
// connector
// dustbowl region 2 variant
// connector
// ..
// 8 mega regions (8 points), connected by connectors and s
