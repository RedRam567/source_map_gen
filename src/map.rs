//! Representation of a map containing [`Solid`]s and [`Entity`]s.

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
    pub solids: Vec<Solid<'a>>,
    pub entities: Vec<Entity<'a>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapOptions {
    /// Surround the level with a giant box with skybox textures.
    /// Notoriously bad for compile times and optimization but will prevent leaks.
    /// Good for quick testing.
    /// TODO: add as actual cordon.
    pub cordon: Option<Bounds<f32>>,
    // TODO: skybox, detail texture, name or smth
    // TODO: ooo aditional files, nav, missions or smth, pop
}

impl<'a> Map<'a> {
    pub fn add_solid(&mut self, solid: Solid<'a>) {
        self.solids.push(solid);
    }
}
