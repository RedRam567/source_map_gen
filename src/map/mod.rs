//! Representation of a map containing [`Solid`]s and [`Entity`]s

pub(crate) mod entity;
pub(crate) mod solid;
pub(crate) mod texture;
pub(crate) mod vector;

pub use entity::*;
pub use solid::*;
pub use texture::*;
pub use vector::*;

use crate::generation::Bounds;

/// The entire world, consiting of [`Solid`]s, [`Entity`]s, and global info
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Map<'a> {
    pub options: MapOptions,
    pub world_entity: WorldEntity,
    pub solids: Vec<Solid<'a>>,
    pub entities: Vec<Entity<'a>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapOptions {
    /// Surround the level with a giant box with skybox textures.
    /// Notoriously bad for compile times and optimization but will prevent leaks.
    /// Good for quick testing.
    /// TODO: add as actual cordon
    pub cordon: Option<Bounds<f32>>,
}

// TODO:
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldEntity {}

// /// A property for a map.
// #[derive(Clone, Debug, PartialEq)]
// pub struct Prop {
//     pub key: String,
//     pub value: Value,
// }

// /// A number or string
// #[derive(Clone, Debug, PartialEq)]
// pub enum Value {
//     Float(f64),
//     Int(i64),
//     String(String),
// }

// impl Display for Value {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Value::Float(v) => v.fmt(f),
//             Value::Int(v) => v.fmt(f),
//             Value::String(v) => v.fmt(f),
//         }
//     }
// }

// impl From<Prop> for Property<String> {
//     fn from(value: Prop) -> Self {
//         Property { key: value.key, value: value.value.to_string() }
//     }
// }

impl<'a> Map<'a> {
    pub fn add_solid(&mut self, solid: Solid<'a>) {
        self.solids.push(solid);
    }
}
