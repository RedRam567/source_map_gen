use std::fmt::Display;

use crate::MAT_DEV_WALL;

/// A peice of map geometry. Ex: a cube, cylinder
#[derive(Clone, Debug, PartialEq)]
pub struct Solid {
    pub sides: Vec<Side>,
}

/// A side of a [`Solid`].
#[derive(Clone, Debug, PartialEq)]
pub struct Side {
    pub plane: Plane,
    pub texture: Texture,
}

/// A flat geometric plane.
/// When looking directly at the plane, `bottom_left` will be in the bottom left
/// and so on, with the normal being towards you.
/// <https://developer.valvesoftware.com/wiki/Valve_Map_Format#Planes>
#[derive(Clone, Debug, PartialEq)]
pub struct Plane {
    pub bottom_left: Point<f32>,
    pub top_left: Point<f32>,
    pub top_right: Point<f32>,
}

impl Plane {
    pub const fn new(bottom_left: Point<f32>, top_left: Point<f32>, top_right: Point<f32>) -> Self {
        Self { bottom_left, top_left, top_right }
    }
}

/// A point in 3d space.
#[derive(Clone, Debug, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Point<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

/// Infomation about a texture on a [`Plane`]
#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    pub material: String,
    pub uaxis: UVAxis<f32>,
    pub vaxis: UVAxis<f32>,
    pub light_scale: u8,
}

/// Texture coords.
/// <https://developer.valvesoftware.com/wiki/Valve_Map_Format#U.2FV_Axis>
#[derive(Clone, Debug, PartialEq)]
pub struct UVAxis<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub trans: T,
    pub scale: T,
}

impl Display for Plane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.bottom_left, self.top_left, self.top_right)
    }
}

impl<T: Display> Display for Point<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.x, self.y, self.z)
    }
}

impl<T: Display> Display for UVAxis<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} {} {} {}] {}", self.x, self.y, self.z, self.trans, self.scale)
    }
}

#[repr(transparent)]
pub struct TextureBuilder(Texture);

impl TextureBuilder {
    pub fn new() -> Self {
        Self(Texture {
            material: MAT_DEV_WALL.to_string(),
            uaxis: UVAxis::default_top().0,
            vaxis: UVAxis::default_top().1,
            light_scale: 16,
        })
    }
    // prop unnecessary micro optimization to allow const
    pub const fn build(self) -> Texture {
        // SAFETY: safe as `self` is repr(transparent)
        unsafe { std::mem::transmute(self) }
    }
    /// Set the material.
    /// Allocates if `s` isnt already a [`String`].
    pub fn mat(mut self, s: impl Into<String>) -> Self {
        self.0.material = s.into();
        self
    }
    /// Set the lightmap scale. Cannot be 0
    pub const fn light_scale(mut self, scale: u8) -> Self {
        debug_assert!(scale != 0);
        self.0.light_scale = scale;
        self
    }
    /// Set the uvmap to the default for a side facing upwards.
    pub const fn top(mut self) -> Self {
        self.0.uaxis = UVAxis::default_top().0;
        self.0.vaxis = UVAxis::default_top().1;
        self
    }
    /// Set the uvmap to the default for a side facing downwards.
    pub const fn bottom(self) -> Self {
        self.top()
    }
    /// Set the uvmap to the default for a side facing left.
    pub const fn left(mut self) -> Self {
        self.0.uaxis = UVAxis::default_left().0;
        self.0.vaxis = UVAxis::default_left().1;
        self
    }
    /// Set the uvmap to the default for a side facing right.
    pub const fn right(self) -> Self {
        self.left()
    }
    /// Set the uvmap to the default for a side facing backwards.
    pub const fn back(mut self) -> Self {
        self.0.uaxis = UVAxis::default_back().0;
        self.0.vaxis = UVAxis::default_back().1;
        self
    }
    /// Set the uvmap to the default for a side facing forwards.
    pub const fn front(self) -> Self {
        self.back()
    }
}

impl Default for TextureBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl UVAxis<f32> {
    /// Return the uvmap for a side facing upwards.
    pub const fn default_top() -> (Self, Self) {
        (
            Self { x: 1.0, y: 0.0, z: 0.0, trans: 0.0, scale: 0.25 },
            Self { x: 0.0, y: -1.0, z: 0.0, trans: 0.0, scale: 0.25 },
        )
    }
    /// Return the uvmap for a side facing downwards.
    pub const fn default_bottom() -> (Self, Self) {
        Self::default_top()
    }

    /// Return the uvmap for a side facing left.
    pub const fn default_left() -> (Self, Self) {
        (
            Self { x: 0.0, y: 1.0, z: 0.0, trans: 0.0, scale: 0.25 },
            Self { x: 0.0, y: 0.0, z: -1.0, trans: 0.0, scale: 0.25 },
        )
    }
    /// Return the uvmap for a side facing right.
    pub const fn default_right() -> (Self, Self) {
        Self::default_left()
    }

    /// Return the uvmap for a side facing backwards.
    pub const fn default_back() -> (Self, Self) {
        (
            Self { x: 1.0, y: 0.0, z: 0.0, trans: 0.0, scale: 0.25 },
            Self { x: 0.0, y: 0.0, z: -1.0, trans: 0.0, scale: 0.25 },
        )
    }
    /// Return the uvmap for a side facing forwards.
    pub const fn default_front() -> (Self, Self) {
        Self::default_back()
    }
}
