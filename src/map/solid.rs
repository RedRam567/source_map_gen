use std::{borrow::Cow, fmt::Display};

use crate::StrType;

/// A peice of map geometry. Ex: a cube, cylinder
#[derive(Clone, Debug, PartialEq)]
pub struct Solid<'a> {
    pub sides: Vec<Side<'a>>,
}

impl<'a> Solid<'a> {
    pub const fn new(sides: Vec<Side<'a>>) -> Self {
        Self { sides }
    }
}

/// A side of a [`Solid`].
#[derive(Clone, Debug, PartialEq)]
pub struct Side<'a> {
    pub plane: Plane,
    pub texture: Texture<'a>,
}

impl<'a> Side<'a> {
    pub fn new(plane: Plane, texture: Texture<'a>) -> Self {
        Self { plane, texture }
    }
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
pub struct Texture<'a> {
    pub material: StrType<'a>,
    pub uaxis: UVAxis<f32>,
    pub vaxis: UVAxis<f32>,
    pub light_scale: u8,
}

impl<'a> Texture<'a> {
    pub const fn new(
        material: StrType<'a>,
        uaxis: UVAxis<f32>,
        vaxis: UVAxis<f32>,
        light_scale: u8,
    ) -> Self {
        Self { material, uaxis, vaxis, light_scale }
    }
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

impl<T> UVAxis<T> {
    pub const fn new(x: T, y: T, z: T, trans: T, scale: T) -> Self {
        Self { x, y, z, trans, scale }
    }
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
pub struct TextureBuilder<'a>(Texture<'a>);

impl<'a> TextureBuilder<'a> {
    pub const fn new() -> Self {
        Self(Texture {
            material: Cow::Borrowed(""),
            uaxis: UVAxis::default_top().0,
            vaxis: UVAxis::default_top().1,
            light_scale: 16,
        })
    }
    // prop unnecessary micro optimization to allow const
    pub const fn build(self) -> Texture<'a> {
        // SAFETY: safe as `self` is repr(transparent)
        unsafe { std::mem::transmute(self) }
    }
    /// Set the material.
    pub fn mat(mut self, s: StrType<'a>) -> Self {
        self.0.material = s;
        self
    }
    /// Set the lightmap scale. Cannot be 0.
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

impl<'a> Default for TextureBuilder<'a> {
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
