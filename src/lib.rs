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
#![deny(rustdoc::broken_intra_doc_links)] // impossible to deny normal compile :/
#![deny(clippy::semicolon_if_nothing_returned)]

use std::iter::Peekable;

// #[deprecated]
pub mod generation;
pub mod generation2;
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OneOrVec<T> {
    One(T),
    Vec(Vec<T>),
}

impl<T> OneOrVec<T> {
    pub const fn new() -> Self {
        OneOrVec::Vec(Vec::new())
    }
    pub fn with_capacity(cap: usize) -> Self {
        OneOrVec::Vec(Vec::with_capacity(cap))
    }
    pub const fn is_one(&self) -> bool {
        matches!(*self, Self::One(_))
    }
    pub const fn is_vec(&self) -> bool {
        matches!(*self, Self::Vec(_))
    }
    pub fn to_vec(self) -> Vec<T> {
        match self {
            OneOrVec::One(item) => vec![item],
            OneOrVec::Vec(vec) => vec,
        }
    }
    pub fn to_vec_reserve(self, additional: usize) -> Vec<T> {
        match self {
            OneOrVec::One(item) => {
                let mut vec = Vec::with_capacity(additional + 1);
                vec.push(item);
                vec
            }
            OneOrVec::Vec(mut vec) => {
                vec.reserve(additional);
                vec
            }
        }
    }
    pub fn len(&self) -> usize {
        match self {
            OneOrVec::One(_) => 1,
            OneOrVec::Vec(vec) => vec.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Clone> OneOrVec<T> {
    pub fn push_or_extend(&mut self, other: Self) {
        // get around cant mutate self or use of moved self errors (wtf)
        let mut myself = self;

        match (&mut myself, other) {
            (OneOrVec::One(this), OneOrVec::One(other)) => {
                *myself = OneOrVec::Vec(vec![this.clone(), other]);
            }
            (OneOrVec::One(this), OneOrVec::Vec(mut other)) => {
                other.insert(0, this.clone());
                *myself = OneOrVec::Vec(other);
            }
            (OneOrVec::Vec(this), OneOrVec::One(other)) => {
                this.push(other);
            }
            (OneOrVec::Vec(this), OneOrVec::Vec(other)) => {
                this.extend(other);
            }
        }
    }
}

// TODO:DOCS: not tested because dont want public, use make::visibility
/// An iterator of the current and next value. Last `next()` call returns the last
/// item and the first item.
///
/// # Examples
///
/// ```rust,ignore
#[doc = concat!("use ", std::module_path!(), "::IterWithNext;")]
///
/// let mut iter = IterWithNext::new([0].into_iter());
/// assert_eq!(Some((0, 0)), iter.next());
/// assert_eq!(None, iter.next());
///
/// let mut iter = IterWithNext::new([0, 1].into_iter());
/// assert_eq!(Some((0, 1)), iter.next());
/// assert_eq!(Some((1, 0)), iter.next());
/// assert_eq!(None, iter.next());
///
///  let mut iter = IterWithNext::new([0, 1, 2].into_iter());
/// assert_eq!(Some((0, 1)), iter.next());
/// assert_eq!(Some((1, 2)), iter.next());
/// assert_eq!(Some((2, 0)), iter.next());
/// assert_eq!(None, iter.next());
/// ```
#[derive(Clone, Debug)]
pub(crate) struct IterWithNext<I, T>
where
    I: Iterator<Item = T>,
{
    iter: Peekable<I>,
    first_item: Option<T>,
}

impl<I, T> IterWithNext<I, T>
where
    I: Iterator<Item = T>,
{
    pub fn new(iter: I) -> Self {
        Self { iter: iter.peekable(), first_item: None }
    }

    pub const fn new_peekable(iter: Peekable<I>) -> Self {
        Self { iter, first_item: None }
    }
}

impl<I, T> Iterator for IterWithNext<I, T>
where
    I: Iterator<Item = T>,
    T: Clone,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.iter.next()?; // if none, iteration over
        if self.first_item.is_none() {
            // set first item
            self.first_item = Some(current.clone());
        }

        // peek else first_item else None
        let next = match self.iter.peek() {
            Some(v) => v.clone(),
            // if none, iter was len 1, return none
            // TODO: is this ever none?
            None => match self.first_item.clone() {
                Some(v) => v,
                None => {
                    if cfg!(debug_assertions) {
                        unreachable!("impossible for first_item to be None");
                    } else {
                        // SAFETY: `first_item` is always set here as if None on
                        // first iteration, it imediatly returns none
                        unsafe { std::hint::unreachable_unchecked() }
                    }
                }
            },
        };

        Some((current, next))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I, T> ExactSizeIterator for IterWithNext<I, T>
where
    I: ExactSizeIterator<Item = T>,
    T: Clone,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

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
