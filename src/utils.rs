//! Misc structs and traits
// TODO: merge into lib.rs?

use std::iter::Peekable;
use std::ops::{Index, IndexMut};
pub use std_stabilized::*;

pub mod std_stabilized {
    //! Things that should be in std or are nightly and I want to use in stable
    use std::mem::MaybeUninit;

    /// A non-nightly alternative to [`std::iter::Iterator::next_chunk()`].
    /// Advances the iterator by `N` elements and return as an array.
    /// If any `next()` call returns `None`, drops results and returns None.
    ///
    /// FIXME: pretty sure it leaks memory if `next()` panics
    pub trait NextChunk: Iterator {
        // Basically from std.
        fn next_chunk2<const N: usize>(&mut self) -> Option<[Self::Item; N]> {
            // Create an array but dont init it for performance ig
            let mut array = uninit_array::<N, Self::Item>();

            // Populate it, dropping entire array if not enough
            for i in 0..N {
                if let Some(v) = self.next() {
                    // populate
                    array[i] = MaybeUninit::new(v);
                } else {
                    // drop all initialized
                    // SAFETY: 0..i is initialized
                    unsafe { slice_assume_init_drop(&mut array[0..i]) }
                    return None;
                }
            }

            // Assume the whole array is initialized by casting // isnt this just a transmute?
            // SAFETY: All were populated. See `MaybeUninit::array_assume_init`
            // let ret = unsafe { (&array as *const _ as *const [Self::Item; N]).read() };
            let ret = unsafe { array_assume_init(array) };
            Some(ret)
        }
    }

    impl<I: Iterator<Item = T>, T> NextChunk for I {}

    /// from std
    const fn uninit_array<const N: usize, T>() -> [MaybeUninit<T>; N] {
        // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
        unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
    }

    /// # Safety
    /// The array must be initialized.
    unsafe fn array_assume_init<const N: usize, T>(array: [MaybeUninit<T>; N]) -> [T; N] {
        (&array as *const _ as *const [T; N]).read()
    }

    /// Drop the contents of the slice in place.
    /// See [`MaybeUninit::assume_init_drop()`]
    /// # Safety
    /// The slice must be initialized.
    unsafe fn slice_assume_init_drop<T>(slice: &mut [MaybeUninit<T>]) {
        for v in slice {
            v.assume_init_drop();
        }
    }

    pub trait TryMap<const N: usize, T, U, E, F> {
        fn try_map2(self, f: F) -> Result<[U; N], E>;
    }

    impl<const N: usize, T, U, E, F> TryMap<N, T, U, E, F> for [T; N]
    where
        F: FnMut(T) -> Result<U, E>,
    {
        /// Array try_map
        fn try_map2(self, mut f: F) -> Result<[U; N], E>
        where
            F: FnMut(T) -> Result<U, E>,
        {
            let mut array = uninit_array::<N, U>();

            for (i, v) in self.into_iter().enumerate() {
                match f(v) {
                    // populate
                    Ok(u) => array[i] = MaybeUninit::new(u),
                    // drop on error
                    Err(e) => {
                        // SAFETY: 0..i is initialized
                        unsafe {
                            slice_assume_init_drop(&mut array[0..i]);
                        }
                        return Err(e);
                    }
                }
            }

            // SAFETY: All were populated. See `MaybeUninit::array_assume_init`
            let ret = unsafe { array_assume_init(array) };
            Ok(ret)
        }
    }
}

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
    pub fn into_vec(self) -> Vec<T> {
        match self {
            OneOrVec::One(item) => vec![item],
            OneOrVec::Vec(vec) => vec,
        }
    }
    pub fn into_vec_reserve(self, additional: usize) -> Vec<T> {
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
    /// Get first element
    ///
    /// # Panics
    /// Panics if it is a `Vec` of len 0
    pub fn first(&self) -> &T {
        match self {
            OneOrVec::One(t) => t,
            OneOrVec::Vec(v) => &v[0],
        }
    }
    /// Get first element
    ///
    /// # Panics
    /// Panics if it is a `Vec` of len 0
    pub fn first_mut(&mut self) -> &mut T {
        match self {
            OneOrVec::One(t) => t,
            OneOrVec::Vec(v) => &mut v[0],
        }
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
    pub iter: Peekable<I>,
    pub first_item: Option<T>,
}

impl<I, T> IterWithNext<I, T>
where
    I: Iterator<Item = T>,
{
    pub fn new(iter: I) -> Self {
        Self { iter: iter.peekable(), first_item: None }
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

pub type Vec1d<T> = VecNd<T, 1>;
pub type Vec2d<T> = VecNd<T, 2>;
pub type Vec3d<T> = VecNd<T, 3>;

// TODO: SliceNd
/// A basic n-th dimensional Vec.
/// Think of it as a `Vec<Vec<T>>` but implemented as one `Vec`.
/// Strides/indexes are in the order Z,Y,X, etc.
/// Capable of growing in the "last" dimension. (think if it as the outer `Vec<Vec<T>>`)
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct VecNd<T, const D: usize> {
    pub inner: Vec<T>,
    /// A stride is the offset for a dimension in the inner Vec.
    /// The "width" of a dimension. Last stride is per-element space, where 2
    /// represents a gap of 1, effectively halving the length
    pub strides: [usize; D],
}

impl<T, const D: usize> VecNd<T, D> {
    pub const fn new(strides: [usize; D]) -> Self {
        Self { inner: Vec::new(), strides }
    }
    pub const fn from_parts(vec: Vec<T>, strides: [usize; D]) -> Self {
        Self { inner: vec, strides }
    }
    pub fn with_capacity(capacity: usize, strides: [usize; D]) -> Self {
        Self { inner: Vec::with_capacity(capacity), strides }
    }
    pub fn get(&self, index: [usize; D]) -> Option<&T> {
        let i = index
            .into_iter()
            .zip(self.strides)
            // map(index * stride).sum()
            .fold(0, |acc, (index, stride)| acc + index * stride);
        self.inner.get(i)
    }
    pub fn get_mut(&mut self, index: [usize; D]) -> Option<&mut T> {
        let i = index
            .into_iter()
            .zip(self.strides)
            // map(index * stride).sum()
            .fold(0, |acc, (index, stride)| acc + index * stride);
        self.inner.get_mut(i)
    }
}

// NOTE: pretty sure this is what I want. An associated fn that doesn't depend
// on the type at all but you still use as `VecNd::<N>::strides()`
impl Vec1d<()> {
    /// Get the stride for a Vec2d with per-element stride of 1.
    pub const fn strides() -> [usize; 1] {
        [1] // lmao
    }
}
impl Vec2d<()> {
    /// Get the `strides` for a Vec2d with per-element stride of 1.
    /// Height can grow endlessly and is omitted.
    pub const fn strides(width: usize) -> [usize; 2] {
        [width, 1]
    }
}
impl Vec3d<()> {
    /// Get the `strides` for a Vec2d with per-element stride of 1.
    /// Depth can grow endlessly and is omitted.
    pub const fn strides(height: usize, width: usize) -> [usize; 3] {
        [height, width, 1]
    }
}

impl<T, const D: usize> Index<[usize; D]> for VecNd<T, D> {
    type Output = T;

    fn index(&self, index: [usize; D]) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T, const D: usize> IndexMut<[usize; D]> for VecNd<T, D> {
    fn index_mut(&mut self, index: [usize; D]) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

/// Iter over rows of a [`Vec2d`]. Returns a truncated slice on last iteration
/// if there are less elements than the width.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Vec2dRows<'a, T> {
    vec2d: &'a Vec2d<T>,
    index: usize,
}

impl<'a, T> Vec2dRows<'a, T> {
    pub const fn new(vec2d: &'a Vec2d<T>) -> Self {
        Self { vec2d, index: 0 }
    }
}

impl<'a, T> Iterator for Vec2dRows<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        // oob, iteration done
        let len = self.vec2d.inner.len();
        if self.index >= len {
            return None;
        }

        let start = self.index;
        let width = self.vec2d.strides[0];
        self.index += width;

        let end = start + width;
        let end = end.min(len); // return remaining

        Some(&self.vec2d.inner[start..end])
    }

    // TODO: size_hint, exact size, fuse maybe
}

impl<T> Vec2d<T> {
    pub const fn rows(&self) -> Vec2dRows<T> {
        Vec2dRows::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows() {
        let vec_2d = Vec2d::from_parts((0..9).collect(), [3, 1]);
        let mut rows = vec_2d.rows();
        assert_eq!(&[0, 1, 2], rows.next().unwrap());
        assert_eq!(&[3, 4, 5], rows.next().unwrap());
        assert_eq!(&[6, 7, 8], rows.next().unwrap());
        assert_eq!(None, rows.next());
    }
}
