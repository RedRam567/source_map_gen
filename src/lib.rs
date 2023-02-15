//! # Info
//! 
//! In this libray, the xyz axes are right handed Z up. +X is right, +Y is forward, +Z is up.
//! Hammer, Source, Blender are all right handed Z up
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

#![warn(clippy::missing_safety_doc)]
#![warn(clippy::missing_const_for_fn)]
#![allow(clippy::zero_prefixed_literal)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::undocumented_unsafe_blocks)]

const MAT_DEV_WALL: &str = "DEV/DEV_MEASUREWALL01C";
const MAT_DEV_FLOOR: &str = "DEV/DEV_MEASUREGENERIC01B";

// my representation of the world
// trait for converting to vmf part
// others can do trait to convert to their map formats


/// Map structs
pub mod map;
pub mod vmf;
pub mod generation;