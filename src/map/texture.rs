use crate::generation::{LIGHTMAP_SCALE, MAT_SCALE};
use crate::prelude::*;
use std::{borrow::Cow, fmt::Display};

/// Infomation about a texture on a [`Plane`]
#[derive(Clone, Debug, PartialEq)]
pub struct Texture<'a> {
    pub material: StrType<'a>,
    pub uaxis: UVAxis<f32>,
    pub vaxis: UVAxis<f32>,
    pub light_scale: u8,
}

/// Texture transformation matrix. `x` is how much the X axis affects the `UVAxis`, similar for `y` and `z`.
/// `trans` is a translation along the axis.
/// `scale` seems to multiply the output result.
/// <https://developer.valvesoftware.com/wiki/Valve_Map_Format#U.2FV_Axis>
#[derive(Clone, Debug, PartialEq)]
pub struct UVAxis<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub trans: T,
    pub scale: T,
}

#[repr(transparent)]
pub struct TextureBuilder<'a>(Texture<'a>);

impl<'a> Texture<'a> {
    pub const fn new(
        material: StrType<'a>,
        uaxis: UVAxis<f32>,
        vaxis: UVAxis<f32>,
        light_scale: u8,
    ) -> Self {
        Self { material, uaxis, vaxis, light_scale }
    }

    pub const fn cube_textures(mats: [&str; 6]) -> [Texture; 6] {
        let [top, bottom, left, right, back, front] = mats;

        let top = TextureBuilder::new_mat(Cow::Borrowed(top)).top().build();
        let bottom = TextureBuilder::new_mat(Cow::Borrowed(bottom)).bottom().build();
        let left = TextureBuilder::new_mat(Cow::Borrowed(left)).left().build();
        let right = TextureBuilder::new_mat(Cow::Borrowed(right)).right().build();
        let back = TextureBuilder::new_mat(Cow::Borrowed(back)).back().build();
        let front = TextureBuilder::new_mat(Cow::Borrowed(front)).front().build();

        [top, bottom, left, right, back, front]
    }
}

impl<'a> TextureBuilder<'a> {
    pub const fn new() -> Self {
        Self(Texture {
            material: Cow::Borrowed(""),
            uaxis: UVAxis::default_top().0,
            vaxis: UVAxis::default_top().1,
            light_scale: LIGHTMAP_SCALE,
        })
    }
    pub const fn new_mat(material: StrType<'a>) -> Self {
        Self(Texture {
            material,
            uaxis: UVAxis::default_top().0,
            vaxis: UVAxis::default_top().1,
            light_scale: LIGHTMAP_SCALE,
        })
    }
    pub const fn build(self) -> Texture<'a> {
        // SAFETY: safe as `self` is repr(transparent), allows const
        unsafe { std::mem::transmute(self) }
    }
    /// Set the material.
    /// Preferred to use [`TextureBuilder::new_mat`] instead as it's const.
    #[allow(clippy::missing_const_for_fn)] // cannot be const due to String drop
    pub fn mat(mut self, material: StrType<'a>) -> Self {
        self.0.material = material;
        self
    }
    /// Set the lightmap scale. Cannot be 0. Debug asserted.
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
    /// Set the arbitrary uvmap.
    pub const fn uv(mut self, uaxis: UVAxis<f32>, vaxis: UVAxis<f32>) -> Self {
        self.0.uaxis = uaxis;
        self.0.vaxis = vaxis;
        self
    }
}

impl<T> UVAxis<T> {
    pub const fn new(x: T, y: T, z: T, trans: T, scale: T) -> Self {
        Self { x, y, z, trans, scale }
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

    /// Gives the uv for the closest axis for a normal of a plane.
    /// `normal` does not need to be normalized.
    /// See: [`Vector3::greatest_axis`]
    pub fn from_norm_align_world(normal: &Vector3<f32>) -> (Self, Self) {
        let normal_abs = normal.abs();
        let axis = normal_abs.greatest_axis();
        match axis {
            Axis3::X => Self::default_left(),
            Axis3::Y => Self::default_back(),
            Axis3::Z => Self::default_top(),
        }
    }

    /// Gives the uv transform for a plane, given the normal of a plane.
    /// `normal` does not need to be normalized.
    pub fn from_norm(normal: &Vector3<f32>) -> (Self, Self) {
        // avoid NaNs
        if normal.is_axis_aligned() {
            return Self::from_norm_align_world(normal);
        }

        // get the uaxis by getting vector perpendiuclar to normal
        // and normal mirrored vertically
        let flip_z = Vector3::new(normal.x, normal.y, -normal.z);
        let mut uaxis = normal.cross(&flip_z).normalize();

        // get vaxis by getting vector perpendicular to uaxis and normal
        let mut vaxis = -normal.cross(&uaxis).normalize();

        // uh magic fix for signs of downwards facing normals
        if normal.z.is_sign_negative() {
            uaxis.x = -uaxis.x;
            uaxis.y = -uaxis.y;
            uaxis.z = -uaxis.z; // not nessessary in testing data but should prob use

            vaxis.x = -vaxis.x;
            vaxis.y = -vaxis.y;
            vaxis.z = -vaxis.z;
        }

        let uaxis = UVAxis::new(uaxis.x, uaxis.y, uaxis.z, 0.0, MAT_SCALE);
        let vaxis = UVAxis::new(vaxis.x, vaxis.y, vaxis.z, 0.0, MAT_SCALE);

        (uaxis, vaxis)
    }
}

impl<T: Display> Display for UVAxis<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} {} {} {}] {}", self.x, self.y, self.z, self.trans, self.scale)
    }
}

impl<'a> Default for TextureBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}