//! Texture and UVs

use crate::generation::{LIGHTMAP_SCALE, MAT_SCALE};
use crate::prelude::*;
use std::ops::{Deref, DerefMut};
use std::{borrow::Cow, fmt::Display};

pub const NO_DRAW: Material<'static> = Material::new(Cow::Borrowed("tools/toolsnodraw"));

// TODO:FEATURE: allow relative xyz
// TODO:DOCS: use shape link
/// A [`Texture`] not associated with a [`Side`].
///
/// A material, xy trans, xy scale, and lightmap scale.
/// For use in [`crate::generation`] functions.
///
/// Does not impl [`Deref`] into a `Texture`. You must use explicit conversion.
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq)]
pub struct Material<'a>(pub Texture<'a>);

impl<'a> Material<'a> {
    pub const fn new(mat: StrType<'a>) -> Self {
        Texture::new_mat(mat).into_material()
    }

    pub const fn into_texture(self) -> Texture<'a> {
        // SAFETY: safe as `self` is repr(transparent), allows const
        unsafe { std::mem::transmute(self) }
    }

    /// Set the material
    ///
    /// Use [`Material::new()`] if possible as that is const.
    #[allow(clippy::missing_const_for_fn)] // cannot be const due to String drop
    pub fn mat(mut self, material: StrType<'a>) -> Self {
        self.0.material = material;
        self
    }
    /// Set the lightmap scale
    ///
    /// # Panics
    /// Debug asserts that `scale` is not 0
    pub const fn light_scale(mut self, scale: u8) -> Self {
        debug_assert!(scale != 0);
        self.0.light_scale = scale;
        self
    }
    /// Set an arbitrary uvmap
    pub const fn uv(mut self, uaxis: UVAxis<f32>, vaxis: UVAxis<f32>) -> Self {
        self.0.uaxis = uaxis;
        self.0.vaxis = vaxis;
        self
    }
}

impl<'a> Default for Material<'a> {
    fn default() -> Self {
        Self::DEV_PERSON
    }
}
impl<'a> AsRef<Texture<'a>> for Material<'a> {
    fn as_ref(&self) -> &Texture<'a> {
        &self.0
    }
}
impl<'a> AsMut<Texture<'a>> for Material<'a> {
    fn as_mut(&mut self) -> &mut Texture<'a> {
        &mut self.0
    }
}
impl<'a> From<Texture<'a>> for Material<'a> {
    fn from(value: Texture<'a>) -> Self {
        Self(value)
    }
}

impl<'a> Material<'a> {
    // aliases for conveinence. prefer these over og
    // TODO: casing?
    // also: game specific BLOCKBULLETS2 and fog volume
    pub const DEV_FLOOR: Material<'static> = Material::DEV_GRAY;
    pub const DEV_WALL: Material<'static> = Material::DEV_PERSON;
    pub const DEV_128: Material<'static> = Material::DEV_WALL_ORANGE;

    pub const DEV_ORANGE: Material<'static> =
        Material::new(Cow::Borrowed("dev/dev_measuregeneric01"));
    pub const DEV_GRAY: Material<'static> =
        Material::new(Cow::Borrowed("dev/dev_measuregeneric01b"));
    pub const DEV_WALL_ORANGE: Material<'static> =
        Material::new(Cow::Borrowed("dev/dev_measurewall01a"));
    pub const DEV_PERSON: Material<'static> =
        Material::new(Cow::Borrowed("dev/dev_measurewall01c"));
    pub const DEV_WALL_GRAY: Material<'static> =
        Material::new(Cow::Borrowed("dev/dev_measurewall01d"));
    pub const DEV_64: Material<'static> = Material::new(Cow::Borrowed("dev/dev_measurecrate01"));
    pub const DEV_32: Material<'static> = Material::new(Cow::Borrowed("dev/dev_measurecrate02"));

    pub const REFLECT_10: Material<'static> = Material::new(Cow::Borrowed("dev/reflectivity_10"));
    pub const REFLECT_20: Material<'static> = Material::new(Cow::Borrowed("dev/reflectivity_20"));
    pub const REFLECT_30: Material<'static> = Material::new(Cow::Borrowed("dev/reflectivity_30"));
    pub const REFLECT_40: Material<'static> = Material::new(Cow::Borrowed("dev/reflectivity_40"));
    pub const REFLECT_50: Material<'static> = Material::new(Cow::Borrowed("dev/reflectivity_50"));
    pub const REFLECT_60: Material<'static> = Material::new(Cow::Borrowed("dev/reflectivity_60"));
    pub const REFLECT_70: Material<'static> = Material::new(Cow::Borrowed("dev/reflectivity_70"));
    pub const REFLECT_80: Material<'static> = Material::new(Cow::Borrowed("dev/reflectivity_80"));
    pub const REFLECT_90: Material<'static> = Material::new(Cow::Borrowed("dev/reflectivity_90"));

    pub const AREAPORTAL: Material<'static> = Material::new(Cow::Borrowed("tools/toolsareaportal"));
    pub const BLOCKBULLETS: Material<'static> =
        Material::new(Cow::Borrowed("tools/toolsblockbullets"));
    // /// TF2: same as TODO:DOCS: but doesn't cut visleafs
    // pub const BLOCKBULLETS2: Material<'static> = Material::new(Cow::Borrowed("tools/toolsblockbullets2"));
    pub const BLOCKLIGHT: Material<'static> = Material::new(Cow::Borrowed("tools/toolsblocklight"));
    pub const BLOCK_LOS: Material<'static> = Material::new(Cow::Borrowed("tools/toolsblock_los"));
    pub const CLIP: Material<'static> = Material::new(Cow::Borrowed("tools/toolsclip"));
    pub const DOTTED: Material<'static> = Material::new(Cow::Borrowed("tools/toolsdotted"));
    pub const HINT: Material<'static> = Material::new(Cow::Borrowed("tools/toolshint"));
    // pub const INVISIBLEDISPLACEMENT: Material<'static> = Material::new(Cow::Borrowed("tools/toolsinvisibledisplacement"));
    pub const INVISIBLELADDER: Material<'static> =
        Material::new(Cow::Borrowed("tools/toolsinvisibleladder"));
    pub const INVISIBLE: Material<'static> = Material::new(Cow::Borrowed("tools/toolsinvisible"));
    pub const NODRAW: Material<'static> = Material::new(Cow::Borrowed("tools/toolsnodraw"));
    pub const NPCCLIP: Material<'static> = Material::new(Cow::Borrowed("tools/toolsnpcclip"));
    pub const OCCLUDER: Material<'static> = Material::new(Cow::Borrowed("tools/toolsoccluder"));
    pub const PLAYERCLIP: Material<'static> = Material::new(Cow::Borrowed("tools/toolsplayerclip"));
    pub const SKIP: Material<'static> = Material::new(Cow::Borrowed("tools/toolsskip"));
    pub const SKYBOX: Material<'static> = Material::new(Cow::Borrowed("tools/toolsskybox"));
    pub const SKYBOX2D: Material<'static> = Material::new(Cow::Borrowed("tools/toolsskybox2d"));
    pub const TRIGGER: Material<'static> = Material::new(Cow::Borrowed("tools/toolstrigger"));
    //TODO: fog volume
}

/// Texture info for a [`Side`]
///
/// Implements [`Deref`] and other conversions into a [`Material`].
#[derive(Clone, Debug, PartialEq)]
pub struct Texture<'a> {
    pub material: StrType<'a>,
    pub uaxis: UVAxis<f32>,
    pub vaxis: UVAxis<f32>,
    pub light_scale: u8,
}

impl<'a> Texture<'a> {
    pub const fn new(
        material: StrType<'a>, uaxis: UVAxis<f32>, vaxis: UVAxis<f32>, light_scale: u8,
    ) -> Self {
        Self { material, uaxis, vaxis, light_scale }
    }

    pub const fn new_mat(mat: StrType<'a>) -> Self {
        Texture {
            material: mat,
            uaxis: UVAxis::default_top().0,
            vaxis: UVAxis::default_top().1,
            light_scale: LIGHTMAP_SCALE,
        }
    }

    pub const fn into_material(self) -> Material<'a> {
        Material(self)
    }

    pub const fn cube_textures(mats: [&str; 6]) -> [Texture; 6] {
        // let [top, bottom, west, east, south, north] = mats;
        let [east, west, north, south, top, bottom] = mats;

        let east = Texture::new_mat(Cow::Borrowed(east)).east();
        let west = Texture::new_mat(Cow::Borrowed(west)).west();
        let north = Texture::new_mat(Cow::Borrowed(north)).north();
        let south = Texture::new_mat(Cow::Borrowed(south)).south();
        let top = Texture::new_mat(Cow::Borrowed(top)).top();
        let bottom = Texture::new_mat(Cow::Borrowed(bottom)).bottom();

        [east, west, north, south, top, bottom]
    }

    /// Set the uvmap to the default for a side facing east, +X.
    pub const fn east(mut self) -> Self {
        self.uaxis = UVAxis::default_east().0;
        self.vaxis = UVAxis::default_east().1;
        self
    }
    /// Set the uvmap to the default for a side facing west, -X.
    pub const fn west(self) -> Self {
        self.east()
    }
    /// Set the uvmap to the default for a side facing north, +Y.
    pub const fn north(mut self) -> Self {
        self.uaxis = UVAxis::default_north().0;
        self.vaxis = UVAxis::default_north().1;
        self
    }
    /// Set the uvmap to the default for a side facing south -Y.
    pub const fn south(self) -> Self {
        self.north()
    }
    /// Set the uvmap to the default for a side facing upwards +Z.
    pub const fn top(mut self) -> Self {
        self.uaxis = UVAxis::default_top().0;
        self.vaxis = UVAxis::default_top().1;
        self
    }
    /// Set the uvmap to the default for a side facing downwards -Z.
    pub const fn bottom(self) -> Self {
        self.top()
    }
}

impl<'a> Default for Texture<'a> {
    fn default() -> Self {
        Material::DEV_PERSON.into()
    }
}

impl<'a> AsRef<Material<'a>> for Texture<'a> {
    fn as_ref(&self) -> &Material<'a> {
        // SAFETY: safe as repr(transparent)
        unsafe { std::mem::transmute(self) }
    }
}
impl<'a> AsMut<Material<'a>> for Texture<'a> {
    fn as_mut(&mut self) -> &mut Material<'a> {
        // SAFETY: safe as repr(transparent)
        unsafe { std::mem::transmute(self) }
    }
}
impl<'a> Deref for Texture<'a> {
    type Target = Material<'a>;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl<'a> DerefMut for Texture<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
impl<'a> From<Material<'a>> for Texture<'a> {
    fn from(value: Material<'a>) -> Self {
        value.0
    }
}

// // TODO: trash it? and just do default + spread and builder trait or smth
// #[repr(transparent)]
// pub struct TextureBuilder<'a>(Texture<'a>);

// impl<'a> TextureBuilder<'a> {
//     pub const fn new() -> Self {
//         Self(Texture {
//             material: Cow::Borrowed(""),
//             uaxis: UVAxis::default_top().0,
//             vaxis: UVAxis::default_top().1,
//             light_scale: LIGHTMAP_SCALE,
//         })
//     }
//     pub const fn new_mat(material: StrType<'a>) -> Self {
//         Self(Texture {
//             material,
//             uaxis: UVAxis::default_top().0,
//             vaxis: UVAxis::default_top().1,
//             light_scale: LIGHTMAP_SCALE,
//         })
//     }
//     pub const fn build(self) -> Texture<'a> {
//         // SAFETY: safe as `self` is repr(transparent), allows const
//         unsafe { std::mem::transmute(self) }
//     }
//     /// Set the material.
//     /// Preferred to use [`TextureBuilder::new_mat`] instead as it's const.
//     #[allow(clippy::missing_const_for_fn)] // cannot be const due to String drop
//     pub fn mat(mut self, material: StrType<'a>) -> Self {
//         self.0.material = material;
//         self
//     }
//     /// Set the lightmap scale. Cannot be 0. Debug asserted.
//     pub const fn light_scale(mut self, scale: u8) -> Self {
//         debug_assert!(scale != 0);
//         self.0.light_scale = scale;
//         self
//     }
//     /// Set the uvmap to the default for a side facing east, +X.
//     pub const fn east(mut self) -> Self {
//         self.0.uaxis = UVAxis::default_east().0;
//         self.0.vaxis = UVAxis::default_east().1;
//         self
//     }
//     /// Set the uvmap to the default for a side facing west, -X.
//     pub const fn west(self) -> Self {
//         self.east()
//     }
//     /// Set the uvmap to the default for a side facing north, +Y.
//     pub const fn north(mut self) -> Self {
//         self.0.uaxis = UVAxis::default_north().0;
//         self.0.vaxis = UVAxis::default_north().1;
//         self
//     }
//     /// Set the uvmap to the default for a side facing south -Y.
//     pub const fn south(self) -> Self {
//         self.north()
//     }
//     /// Set the uvmap to the default for a side facing upwards +Z.
//     pub const fn top(mut self) -> Self {
//         self.0.uaxis = UVAxis::default_top().0;
//         self.0.vaxis = UVAxis::default_top().1;
//         self
//     }
//     /// Set the uvmap to the default for a side facing downwards -Z.
//     pub const fn bottom(self) -> Self {
//         self.top()
//     }
//     /// Set an arbitrary uvmap.
//     pub const fn uv(mut self, uaxis: UVAxis<f32>, vaxis: UVAxis<f32>) -> Self {
//         self.0.uaxis = uaxis;
//         self.0.vaxis = vaxis;
//         self
//     }
// }

// impl<'a> Default for TextureBuilder<'a> {
//     fn default() -> Self {
//         Self::new()
//     }
// }

/// Texture transformation matrix
///
/// `x` is how much the X axis affects the `UVAxis`, similar for `y` and `z`.
/// `trans` is a translation along the axis.
/// `scale` seems to multiply the output result.
/// <https://developer.valvesoftware.com/wiki/Valve_Map_Format#U.2FV_Axis>
#[derive(Clone, Default, Debug, PartialEq)]
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

impl UVAxis<f32> {
    /// Return the uvmap for a side facing left.
    pub const fn default_east() -> (Self, Self) {
        (
            Self { x: 0.0, y: 1.0, z: 0.0, trans: 0.0, scale: 0.25 },
            Self { x: 0.0, y: 0.0, z: -1.0, trans: 0.0, scale: 0.25 },
        )
    }
    /// Return the uvmap for a side facing right.
    pub const fn default_west() -> (Self, Self) {
        Self::default_east()
    }

    /// Return the uvmap for a side facing backwards.
    pub const fn default_north() -> (Self, Self) {
        (
            Self { x: 1.0, y: 0.0, z: 0.0, trans: 0.0, scale: 0.25 },
            Self { x: 0.0, y: 0.0, z: -1.0, trans: 0.0, scale: 0.25 },
        )
    }
    /// Return the uvmap for a side facing forwards.
    pub const fn default_south() -> (Self, Self) {
        Self::default_north()
    }

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

    /// Gives the uv for the closest axis for a normal of a plane.
    /// `normal` does not need to be normalized.
    /// See: [`Vector3::greatest_axis`]
    pub fn from_norm_align_world(normal: &Vector3<f32>) -> (Self, Self) {
        let normal_abs = normal.clone().abs();
        let axis = normal_abs.greatest_axis();
        match axis {
            Axis3::X => Self::default_east(),
            Axis3::Y => Self::default_north(),
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
        let mut flip_z = Vector3::new(normal.x, normal.y, -normal.z);
        if normal.z.abs() < 1.0 {
            // HACK: fix for non-axis alligned vertical planes where z is 0
            // signs disagree with Wolfram Alpha
            // FIXME: find proper way to get vector perpendicular to normal
            // towards +-Z axis
            // wait, cant we just set it to 0 0 +-1?
            // copysign prob what I want idk
            // if not correct, will just be mirred left/right
            // current: seems to be always not flipped relative to face
            flip_z.z = (flip_z.z + 16.0).copysign(flip_z.z); // arbitrary, maybe need -16
        }
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
