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
#![allow(clippy::needless_lifetimes)] // I dont like to ever use '_
#![allow(clippy::zero_prefixed_literal)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::missing_safety_doc)]
#![deny(clippy::semicolon_if_nothing_returned)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(rustdoc::broken_intra_doc_links)] // impossible to deny normal compile :/

// #[deprecated]
pub mod generation;
pub mod generation2;
pub mod light;
pub mod map;
pub mod source;
pub mod vmf;
pub(crate) mod utils;
// pub mod scripting;

pub mod prelude {
    pub use crate::map::solid::*;
    pub use crate::map::texture::*;
    pub use crate::map::vector::*;
    pub(crate) use crate::StrType;
}

// open questions:
// casing of valve stuff brought into Rust
//      preserve or convert to snake?

/// Fork of [`std::dbg`] but with the output not pretty printed.
macro_rules! dbg2 {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        std::eprintln!("[{}:{}]", std::file!(), std::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                std::eprintln!("[{}:{}] {} = {:?}",
                    std::file!(), std::line!(), std::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(std::dbg!($val)),+,)
    };
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

#[cfg(test)]
mod tests {
    use super::*;
    use utils::*;

    #[test]
    fn iter_peek() {
        let mut iter = IterWithNext::new([0; 0].into_iter());
        assert_eq!(0, iter.len());
        assert_eq!(None, iter.next());

        // desired I think, eh yeah
        let mut iter = IterWithNext::new([0].into_iter());
        assert_eq!(1, iter.len());
        assert_eq!(Some((0, 0)), iter.next());
        assert_eq!(0, iter.len());
        assert_eq!(None, iter.next());

        let mut iter = IterWithNext::new([0, 1].into_iter());
        assert_eq!(2, iter.len());
        assert_eq!(Some((0, 1)), iter.next());
        assert_eq!(1, iter.len());
        assert_eq!(Some((1, 0)), iter.next());
        assert_eq!(0, iter.len());
        assert_eq!(None, iter.next());

        let mut iter = IterWithNext::new([0, 1, 2].into_iter());
        assert_eq!(3, iter.len());
        assert_eq!(Some((0, 1)), iter.next());
        assert_eq!(2, iter.len());
        assert_eq!(Some((1, 2)), iter.next());
        assert_eq!(1, iter.len());
        assert_eq!(Some((2, 0)), iter.next());
        assert_eq!(0, iter.len());
        assert_eq!(None, iter.next());

        let mut iter = IterWithNext::new([0, 1, 2, 3].into_iter());
        assert_eq!(4, iter.len());
        assert_eq!(Some((0, 1)), iter.next());
        assert_eq!(3, iter.len());
        assert_eq!(Some((1, 2)), iter.next());
        assert_eq!(2, iter.len());
        assert_eq!(Some((2, 3)), iter.next());
        assert_eq!(1, iter.len());
        assert_eq!(Some((3, 0)), iter.next());
        assert_eq!(0, iter.len());
        assert_eq!(None, iter.next());

        let mut iter = IterWithNext::new([0, 1, 2, 3, 4].into_iter());
        assert_eq!(5, iter.len());
        assert_eq!(Some((0, 1)), iter.next());
        assert_eq!(4, iter.len());
        assert_eq!(Some((1, 2)), iter.next());
        assert_eq!(3, iter.len());
        assert_eq!(Some((2, 3)), iter.next());
        assert_eq!(2, iter.len());
        assert_eq!(Some((3, 4)), iter.next());
        assert_eq!(1, iter.len());
        assert_eq!(Some((4, 0)), iter.next());
        assert_eq!(0, iter.len());
        assert_eq!(None, iter.next());

        // panic!("{}", std::module_path!().split_once("::").unwrap().0);
        // panic!("{}", std::module_path!());
    }
}
