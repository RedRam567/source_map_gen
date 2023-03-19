//! Handles generating regions, contains sub regions.
//! Ex: city region containing interior region, containing floors, containing rooms

// pub struct Region {

mod room;

pub use room::*;

use super::Bounds;
use indextree::{Arena, NodeId};
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

/// half wall thickness because both sides of the rooms use this
pub const WALL_THICKNESS: f32 = 8.0;

// }
// connections
// bounds
// sub rooms
// theme
// theme builder
//  textures
//  sub rooms
//  props

// 3d matrix of 64x64 tiles?

// root
//  maps
//   regions (city)
//    roads and buildings
//     floors
//      room
#[derive(Clone, Debug, PartialEq)]
pub struct MapTree<'a> {
    pub tree: Arena<Room<'a>>,
    // incredibly annoying `Arena` doesn't have a root node or
    // node methods on it (how do you even pretty print an `Arena`?)
    pub root: NodeId,
    pub theme: Theme,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    // maps, regions, roads, buildings, floors, rooms
    pub max_level: u8,
    pub rand: ChaCha8Rng,
}

impl Theme {
    pub fn new(num_levels: u8, seed: u64) -> Self {
        let rand = ChaCha8Rng::seed_from_u64(seed);
        Self { max_level: num_levels, rand }
    }
}

impl<'a> MapTree<'a> {
    // bounds of the enitre level
    pub fn new(bounds: Bounds<f32>, theme: Theme) -> Self {
        let mut tree = Arena::new();
        let root = tree.new_node(Room::new(bounds));
        Self { tree, root, theme }
    }

    // TODO: traitify build methods
    pub fn build(&mut self) {
        for level in 0..=self.theme.max_level {
            eprintln!("BUILDING LEVEL {level}...");
            self.build_level(level);
        }
    }

    pub fn build_level(&mut self, level: u8) {
        match level {
            0 => self.build_level_0(),
            _ => panic!("unsupported level number"),
        }
    }

    pub(crate) fn build_level_0(&mut self) {
        let root = self.root;
        // root.ad
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test() {
        let mut arena = Arena::new();
        let one = arena.new_node(1);
        one.append(arena.new_node(2), &mut arena);
        one.append(arena.new_node(3), &mut arena);
        // dbg!(&arena);
        eprintln!("{}", one.debug_pretty_print(&arena));
        panic!()
    }
}
