//! Vmf format traits and impls.
//! See [`vmfparser`] crate.
mod vmf_impl;

pub use vmf_impl::*;

type Vmf<S> = Vec<Block<S>>;
use vmfparser::ast::{Block, Property};

pub trait ToVmf<S, T, E> {
    /// Convert into vmf ast.
    fn to_vmf(&self, state: &mut T) -> Vmf<S>;
}

pub trait ToBlock<S, T, E> {
    /// Convert into vmf [`Block`].
    fn to_block(&self, state: &mut T) -> Block<S>;
}

pub trait ToProps<K, T, E> {
    /// Convert into vmf [`Property`]s.
    fn to_props(&self, state: &mut T) -> Vec<Property<K>>;
}

pub trait FromVmf<T, U, E>
where
    Self: Sized,
{
    /// Parse from a part of vmf file.
    fn from_vmf(vmf: Vmf<U>, state: &mut T) -> Result<Self, E>;
}

// convenience traits:

// pub trait PushProp<T, K> {
//     fn push_prop(&mut self, key: T, value: String);
// }

// impl<T, K> PushProp<T, K> for Vec<Property<K>>
// where
//     T: Into<K>,
// {
//     fn push_prop(&mut self, key: T, value: String) {
//         self.push(Property { key: key.into(), value })
//     }
// }

pub trait PropertyExt<T, K> {
    fn new(key: T, value: String) -> Self;
}

impl<T, K> PropertyExt<T, K> for Property<K>
where
    T: Into<K>,
{
    fn new(key: T, value: String) -> Property<K> {
        Property { key: key.into(), value }
    }
}
