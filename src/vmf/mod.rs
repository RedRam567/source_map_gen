//! Vmf format traits and impls.
//! See [`vmfparser`] crate.
mod vmf_impl;

use std::{ops::{Deref, DerefMut}, fmt::Display};

pub use vmf_impl::*;
use vmf_parser_nom::ast::{Block, Property};

#[derive(Clone, Debug, Default)]
pub struct Vmf<S>(Vec<Block<S>>);

pub trait ToVmf<S, T, E> {
    /// Convert into vmf ast.
    fn to_vmf(&self, state: &mut T) -> Vmf<S>;
}

pub trait ToBlock<S, T, E> {
    /// Convert into vmf [`Block`].
    fn to_block(&self, state: &mut T) -> Block<S>;
}

pub trait ToProps<T, K, V, E> {
    /// Convert into vmf [`Property`]s.
    fn to_props(&self, state: &mut T) -> Vec<Property<K, V>>;
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

impl<S> Deref for Vmf<S> {
    type Target = Vec<Block<S>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> DerefMut for Vmf<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Display> Display for Vmf<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "// auto generated vmf file")?;
        for block in self.iter() {
            write!(f, "{block:#}")?;
        }
        Ok(())
    }
}

// impl<T, K, V> PropertyExt<T, K> for Property<K, V>
// where
//     T: Into<K>,
// {
//     fn new(key: T, value: String) -> Property<K, V> {
//         Property { key: key.into(), value }
//     }
// }
