//! Vmf format traits and impls.
//! See [`vmfparser`] crate.

use vmf_parser_nom::ast::{Block, Property, Vmf};

use crate::{
    generation::region::Room,
    map::{Entity, Map, Side, Solid},
    StrType,
};

/// Convert to a lower level of abstraction.
/// Example: A [`Solid`](crate::map::solid::Solid) into a [`Block`](vmf_parser_nom::ast::Block).
pub trait ToLower<T>: Clone {
    /// Convert to a lower level of abstraction. See [`ToLower`]
    fn into_lower(self) -> T
    {
        self.to_lower()
    }

    /// Convert to an owned lower level of abstraction. See [`ToLower`]
    fn to_lower(&self) -> T {
        self.clone().into_lower()
    }
}

/// Convert to a higher level of abstraction.
/// Example: A [`Block`](vmf_parser_nom::ast::Block) into a [`Solid`](crate::map::solid::Solid).
pub trait ToHigher<T>: Clone {
    /// Convert this into a low level map element, consuming `self`. See [`ToHigher`].
    fn into_higher(self) -> T;
    /// Convert this into an owned low level map element. See [`ToHigher`].
    fn to_higher(&self) -> T {
        self.clone().into_higher()
    }
}

impl<'a> ToLower<Block<StrType<'a>>> for Side<'a> {
    fn into_lower(self) -> Block<StrType<'a>>
    {
        let props = vec![
            // id unnecessary as [`vmf_parser_nom`] can generate new ids
            Property::new("plane", self.plane.to_string()),
            Property::new("material", self.texture.material.to_string()),
            Property::new("uaxis", self.texture.uaxis.to_string()),
            Property::new("vaxis", self.texture.vaxis.to_string()),
            // rotation and smoothing group not mandatory
            // rotation is just for hammer display, smoothing group defaults to 0 (none)
            // Property::new("rotation", self.rotation.to_string()),
            Property::new("lightmapscale", self.texture.light_scale.to_string()),
            Property::new("smoothing_groups", "0".to_string()),
        ];
        let blocks = vec![]; // TODO: displacement info here
        Block { name: "side".into(), props, blocks }
    }
}

impl<'a> ToLower<Block<StrType<'a>>> for Solid<'a> {
    fn into_lower(self) -> Block<StrType<'a>>
    {
        Block {
            name: "solid".into(),
            // id unnecessary as [`vmf_parser_nom`] can generate new ids
            props: vec![],
            blocks: self.sides.iter().map(|x| x.to_lower()).collect(),
        }
    }
}

impl<'a> ToLower<Block<StrType<'a>>> for Entity<StrType<'a>> {
    fn into_lower(self) -> Block<StrType<'a>>
    {
        Block {
            // name: self.classname.into(),
            name: "entity".into(),
            props: self.props,
            blocks: vec![],
        }
    }
}

impl<'a> ToLower<Vmf<StrType<'a>>> for Map<'a> {
    fn into_lower(self) -> Vmf<StrType<'a>>
    {
        let mut vmf = Vmf::default();
        let mut solid_blocks: Vec<_> = self.solids.iter().map(|s| s.to_lower()).collect();
        if let Some(cordon) = self.options.cordon.clone() {
            // TODO: add as actual cordon
            solid_blocks
                .extend(Room::new(cordon).construct_sky_inside().iter().map(|s| s.to_lower()));
        }

        vmf.inner.blocks.push(Block {
            name: "versioninfo".into(),
            props: vec![
                Property::new("editorversion", "400"),
                Property::new("editorbuild", "9540"),
                Property::new("mapversion", "1"),
                Property::new("formatversion", "100"),
                Property::new("prefab", "0"),
            ],
            blocks: vec![],
        });
        vmf.inner
            .blocks
            .push(Block { name: "visgroups".into(), props: vec![], blocks: vec![] });
        vmf.inner.blocks.push(Block {
            name: "viewsettings".into(),
            props: vec![
                Property::new("bSnapToGrid", "1"),
                Property::new("bShowGrid", "1"),
                Property::new("bShowLogicalGrid", "0"),
                Property::new("nGridSpacing", "64"),
                Property::new("bShow3DGrid", "0"),
            ],
            blocks: vec![],
        });
        vmf.inner.blocks.push(Block {
            name: "world".into(),
            props: vec![
                Property::new("mapversion", "1"),
                Property::new("classname", "worldspawn"),
                Property::new("skyname", self.options.sky_name),
            ],
            blocks: solid_blocks,
        });
        // ENTS HERE
        vmf.inner.blocks.extend(self.entities.iter().map(|e| e.to_lower()));
        // cameras unnessesary
        // cordons unnessesary
        vmf
    }
}

// /// Trait to convert into a low level map element representation.
// /// Example: A mid level [`Solid`](crate::map::solid) into a low level [`Block`](vmf_parser_nom::ast::Block)
// pub trait ToLowLevel {
//     type Target: ?Sized;
//     /// Convert this into a low level map element, consuming `self`.
//     fn into_low_level(self) -> Self::Target;
//     /// Convert this into an owned low level map element.
//     fn to_low_level(&self) -> Self::Target;
// }

// /// Trait to convert into a mid level map element representation.
// /// Example: A high level <TODO: ROOM> into a mid level vec [`Solid`](crate::map::solid)s
// pub trait ToMidLevel {
//     type Target: ?Sized;
//     /// Convert this into a mid level map element, consuming `self`.
//     fn into_low_level(self) -> Self::Target;
//     /// Convert this into an owned mid level map element.
//     fn to_low_level(&self) -> Self::Target;
// }

// /// Trait to convert into a high level map element representation.
// /// Example: A vec of solids into a room.
// pub trait ToHighLevel {
//     type Target: ?Sized;
//     /// Convert this into a high level map element, consuming `self`.
//     fn into_low_level(self) -> Self::Target;
//     /// Convert this into an owned high level map element.
//     fn to_low_level(&self) -> Self::Target;
// }

// use std::{ops::{Deref, DerefMut}, fmt::Display};

// pub use vmf_impl::*;
// use vmf_parser_nom::ast::{Block, Property};

// #[derive(Clone, Debug, Default)]
// pub struct Vmf<S>(Vec<Block<S>>);

// pub trait ToVmf<S, T, E> {
//     /// Convert into vmf ast.
//     fn to_vmf(&self, state: &mut T) -> Vmf<S>;
// }

// pub trait ToBlock<S, T, E> {
//     /// Convert into vmf [`Block`].
//     fn to_block(&self, state: &mut T) -> Block<S>;
// }

// pub trait ToProps<T, K, V, E> {
//     /// Convert into vmf [`Property`]s.
//     fn to_props(&self, state: &mut T) -> Vec<Property<K, V>>;
// }

// pub trait FromVmf<T, U, E>
// where
//     Self: Sized,
// {
//     /// Parse from a part of vmf file.
//     fn from_vmf(vmf: Vmf<U>, state: &mut T) -> Result<Self, E>;
// }

// // convenience traits:

// // pub trait PushProp<T, K> {
// //     fn push_prop(&mut self, key: T, value: String);
// // }

// // impl<T, K> PushProp<T, K> for Vec<Property<K>>
// // where
// //     T: Into<K>,
// // {
// //     fn push_prop(&mut self, key: T, value: String) {
// //         self.push(Property { key: key.into(), value })
// //     }
// // }

// pub trait PropertyExt<T, K> {
//     fn new(key: T, value: String) -> Self;
// }

// impl<S> Deref for Vmf<S> {
//     type Target = Vec<Block<S>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl<S> DerefMut for Vmf<S> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl<S: Display> Display for Vmf<S> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "// auto generated vmf file")?;
//         for block in self.iter() {
//             write!(f, "{block:#}")?;
//         }
//         Ok(())
//     }
// }

// // impl<T, K, V> PropertyExt<T, K> for Property<K, V>
// // where
// //     T: Into<K>,
// // {
// //     fn new(key: T, value: String) -> Property<K, V> {
// //         Property { key: key.into(), value }
// //     }
// // }
