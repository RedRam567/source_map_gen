pub mod shape;
pub mod disp;

use crate::prelude::{Plane, Vector2, Vector3};

pub const SWB: usize = 0;
pub const NWB: usize = 1;
pub const NEB: usize = 2;
pub const SEB: usize = 3;
pub const SWT: usize = 4;
pub const NWT: usize = 5;
pub const NET: usize = 6;
pub const SET: usize = 7;

// upgrade to frac if too small?
// TODO: top offset and top size or smth
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SolidOptions {
    /// Number of sides of a spikes base, number of layers and number of sides
    /// of those layers of a sphere, etc.
    pub sides: u32,
    /// Allow vertexes to have a fractional part instead of rounding to an integer.
    pub allow_frac: bool,
    /// Allow fractional vertexes if the shape is too small to accurately have
    /// as many sides as it has.
    pub frac_promote: bool,
    /// Wether to align textures to the nearest axis instead of relative to the face.
    pub world_align: bool,
    // /// How to split a shape into one or more solids.
    // pub grouping: Grouping,
}

impl SolidOptions {
    pub const fn new() -> Self {
        Self {
            sides: 16,
            allow_frac: false,
            frac_promote: false,
            world_align: false,
            // grouping: Grouping::Auto,
        }
    }
    pub const fn sides(self, sides: u32) -> Self {
        Self { sides, ..self }
    }
    pub const fn allow_frac(self) -> Self {
        Self { allow_frac: true, ..self }
    }
    pub const fn frac_promote(self) -> Self {
        Self { frac_promote: true, ..self }
    }
    pub const fn world_align(self) -> Self {
        Self { world_align: true, ..self }
    }
    pub const fn face_align(self) -> Self {
        Self { world_align: false, ..self }
    }
}

impl Default for SolidOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounds in 3d space.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Bounds {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Bounds {
    /// Create bounds from any two points in 3d space.
    pub fn new(point1: Vector3<f32>, point2: Vector3<f32>) -> Self {
        Self {
            min: Vector3 {
                x: point1.x.min(point2.x),
                y: point1.y.min(point2.y),
                z: point1.z.min(point2.z),
            },
            max: Vector3 {
                x: point1.x.max(point2.x),
                y: point1.y.max(point2.y),
                z: point1.z.max(point2.z),
            },
        }
    }

    /// Returns the 8 vertexes of the [`Bounds`] in the order:
    ///
    /// `west south bottom`, `west north bottom`, `east north bottom`, `east south bottom`
    ///
    /// `west south top`, `west north top`, `east north top`, `east south top`
    pub const fn verts(&self) -> [Vector3<f32>; 8] {
        // TODO: change spiral order, way too lazy
        [
            Vector3 { ..self.min },                               // 0 south west bottom
            Vector3 { y: self.max.y, ..self.min },                // 1 north west bottom
            Vector3 { x: self.max.x, y: self.max.y, ..self.min }, // 2 north east bottom
            Vector3 { x: self.max.x, ..self.min },                // 3 south east bottom
            Vector3 { x: self.min.x, y: self.min.y, ..self.max }, // 4 south west top
            Vector3 { x: self.min.x, ..self.max },                // 5 north west top
            Vector3 { ..self.max },                               // 6 north east top
            Vector3 { y: self.min.y, ..self.max },                // 7 south east top
        ]
    }

    /// The center of `self` on the XY plane.
    pub fn center_xy(&self) -> Vector2<f32> {
        let x = (self.min.x + self.max.x) / 2.0;
        let y = (self.min.y + self.max.y) / 2.0;
        Vector2::new(x, y)
    }

    /// The center of `self` on the XY plane.
    pub fn center_yz(&self) -> Vector2<f32> {
        let y = (self.min.y + self.max.y) / 2.0;
        let z = (self.min.z + self.max.z) / 2.0;
        Vector2::new(y, z)
    }

    /// The center of `self` on the XY plane.
    pub fn center_xz(&self) -> Vector2<f32> {
        let x = (self.min.x + self.max.x) / 2.0;
        let z = (self.min.z + self.max.z) / 2.0;
        Vector2::new(x, z)
    }

    pub fn center(&self) -> Vector3<f32> {
        let x = (self.min.x + self.max.x) / 2.0;
        let y = (self.min.y + self.max.y) / 2.0;
        let z = (self.min.z + self.max.z) / 2.0;
        Vector3::new(x, y, z)
    }

    pub fn top_center(&self) -> Vector3<f32> {
        let x = (self.min.x + self.max.x) / 2.0;
        let y = (self.min.y + self.max.y) / 2.0;
        let z = self.max.x;
        Vector3::new(x, y, z)
    }

    pub fn bottom_center(&self) -> Vector3<f32> {
        let x = (self.min.x + self.max.x) / 2.0;
        let y = (self.min.y + self.max.y) / 2.0;
        let z = self.min.x;
        Vector3::new(x, y, z)
    }

    pub fn x_len(&self) -> f32 {
        (self.min.x - self.max.x).abs()
    }

    pub fn y_len(&self) -> f32 {
        (self.min.y - self.max.y).abs()
    }

    pub fn z_len(&self) -> f32 {
        (self.min.z - self.max.z).abs()
    }

    pub const fn top_plane(&self) -> Plane {
        // 7 6 5
        Plane::new(
            Vector3 { x: self.min.x, ..self.max },
            Vector3 { ..self.max },
            Vector3 { y: self.min.y, ..self.max },
        )
    }

    pub const fn bottom_plane(&self) -> Plane {
        // 2, 1, 0
        Plane::new(
            Vector3 { x: self.max.x, y: self.max.y, ..self.min },
            Vector3 { y: self.max.y, ..self.min },
            Vector3 { ..self.min },
        )
    }

    pub fn translate(mut self, trans: &Vector3<f32>) -> Self {
        self.translate_mut(trans);
        self
    }

    pub fn translate_mut(&mut self, trans: &Vector3<f32>) -> &mut Self {
        self.min += trans;
        self.max += trans;
        self
    }
}
