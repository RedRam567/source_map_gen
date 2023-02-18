//! # Info
//!
//! In this libray, the xyz axes are right handed Z up. +X is right, +Y is forward, +Z is up.
//! Hammer, Source, Blender, Math are all right handed Z up.
//! (Forward direction is inconsistent, -X is "the convention")
//! [https://en.wikipedia.org/wiki/Cartesian_coordinate_system#Notations_and_conventions](https://en.wikipedia.org/wiki/Cartesian_coordinate_system#Notations_and_conventions)
//!
//! # Definitions
//!
//! - Right / right face: The direction or face closest to +X.
//! - Left / left face: The direction or face closest to -X.
//! - Front / front face: The direction or face closest to +Y.
//! - Back / back face: The direction or face closest to -Y.
//! - Top / top face: The direction or face closest to +Z.
//! - Bottom / bottom face: The direction or face closest to -Z.
//! - Width how long on the X axis.
//! - Length how long on the Y axis.
//! - Height how long on the Z axis.
// TODO: change forward axis??

#![warn(clippy::missing_safety_doc)]
#![warn(clippy::missing_const_for_fn)]
#![allow(clippy::zero_prefixed_literal)]
#![deny(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod generation;
pub mod map;
pub mod vmf;

pub(crate) type StrType<'a> = std::borrow::Cow<'a, str>;

// normal slope stair default

// my representation of the world
// trait for converting to vmf part
// others can do trait to convert to their map formats
