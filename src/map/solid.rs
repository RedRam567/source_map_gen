use super::{vector::Vector3, Axis3};
use crate::generation::{LIGHTMAP_SCALE, MAT_SCALE};
use crate::StrType;
use std::{borrow::Cow, fmt::Display, hint::unreachable_unchecked};

/// A peice of map geometry. Ex: a cube, cylinder
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Solid<'a> {
    pub sides: Vec<Side<'a>>,
}

/// A side of a [`Solid`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Side<'a> {
    pub plane: Plane,
    pub texture: Texture<'a>,
}

/// A flat geometric plane.
/// When looking directly at the plane, `bottom_left` will be in the bottom left
/// and so on, with the normal being towards you.
/// <https://developer.valvesoftware.com/wiki/Valve_Map_Format#Planes>
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plane {
    pub bottom_left: Vector3<f32>,
    pub top_left: Vector3<f32>,
    pub top_right: Vector3<f32>,
}

/// Infomation about a texture on a [`Plane`]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Texture<'a> {
    pub material: StrType<'a>,
    pub uaxis: UVAxis<f32>,
    pub vaxis: UVAxis<f32>,
    pub light_scale: u8,
}

#[repr(transparent)]
pub struct TextureBuilder<'a>(Texture<'a>);

/// Texture coords.
/// <https://developer.valvesoftware.com/wiki/Valve_Map_Format#U.2FV_Axis>
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UVAxis<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub trans: T,
    pub scale: T,
}

impl<'a> Solid<'a> {
    pub const fn new(sides: Vec<Side<'a>>) -> Self {
        Self { sides }
    }

    pub fn translate_mut(&mut self, trans: &Vector3<f32>) -> &mut Self {
        for side in self.sides.iter_mut() {
            side.translate_mut(trans);
        }
        self
    }
}
// TODO: normal, slope, wtf is default

impl<'a> Side<'a> {
    pub const fn new(plane: Plane, texture: Texture<'a>) -> Self {
        Self { plane, texture }
    }

    pub fn translate_mut(&mut self, trans: &Vector3<f32>) -> &mut Self {
        self.plane.translate_mut(trans);
        self
    }
}

impl Plane {
    pub const fn new(
        bottom_left: Vector3<f32>,
        top_left: Vector3<f32>,
        top_right: Vector3<f32>,
    ) -> Self {
        Self { bottom_left, top_left, top_right }
    }

    pub fn translate_mut(&mut self, trans: &Vector3<f32>) -> &mut Self {
        self.bottom_left += trans;
        self.top_left += trans;
        self.top_right += trans;
        self
    }

    // TODO: explanation in new or struct, link
    /// Returns the non-normalized normal.
    pub fn normal_dir(&self) -> Vector3<f32> {
        // reverse order, outwards from a cube
        let a = &self.top_right;
        let b = &self.top_left;
        let c = &self.bottom_left;

        (b.clone() - a).cross(&(c.clone() - a))
    }

    /// Returns the normalized normal.
    pub fn normal(&self) -> Vector3<f32> {
        self.normal_dir().normalize()
    }
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
    // /// Set the material.
    // #[allow(clippy::missing_const_for_fn)] // cannot be const due to String drop
    // pub fn mat(mut self, material: StrType<'a>) -> Self {
    //     self.0.material = material;
    //     self
    // }
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

    /// Gives the uv for the normal of a plane.
    /// `normal` does not need to be normalized.
    pub fn from_norm(normal: &Vector3<f32>) -> (Self, Self) {
        eprintln!("normal\t{normal}");

        // TODO: FIXME: axis aligned = nans
        // get the uaxis by getting vector perpendiuclar to normal
        // and normal mirrored around xy plane
        let flip_z = Vector3::new(normal.x, normal.y, -normal.z);
        let mut uaxis = normal.cross(&flip_z).normalize();
        eprintln!("perp2\t{uaxis}");

        // get vaxis by getting vecotr perpendicular to uaxis and normal
        let mut vaxis = -normal.cross(&uaxis).normalize();
        eprintln!("perp3\t{vaxis}");

        // uh magic fix for signs makes textures "face upwards"
        // invert sign if z is negative
        uaxis.x *= normal.z.signum();
        uaxis.y *= normal.z.signum();

        vaxis.x *= normal.z.signum();
        vaxis.y *= normal.z.signum();
        vaxis.z *= normal.z.signum();

        let uaxis = UVAxis::new(uaxis.x, uaxis.y, uaxis.z, 0.0, MAT_SCALE);
        let vaxis = UVAxis::new(vaxis.x, vaxis.y, vaxis.z, 0.0, MAT_SCALE);

        (uaxis, vaxis)
    }
}

// let rx = Vector3::new(x, z, -y); // RX
// eprintln!("rx {rx:?}");
// let ry = Vector3::new(-z, y, x); // RY
// eprintln!("ry {ry:?}");
// let rz = Vector3::new(y, -x, z); // RZ
// eprintln!("rz {rz:?}");

impl Display for Plane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.bottom_left, self.top_left, self.top_right)
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

#[allow(clippy::approx_constant)]
#[cfg(test)]
mod tests {
    use approx::{abs_diff_eq, assert_abs_diff_eq};

    use super::*;
    use crate::{
        generation::{Bounds, MAT_SCALE},
        map::Map,
    };

    #[test]
    fn normal() {
        let truth = [
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ];
        let bounds = Bounds::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        let normals: Vec<_> =
            Map::cube_dev(bounds).sides.iter().map(|s| s.plane.normal()).collect();

        for (truth, normal) in truth.iter().zip(normals) {
            assert_eq!(truth, &normal);
        }
    }

    #[test]
    fn uv_align() {
        let truth = [
            UVAxis::default_top(),
            UVAxis::default_top(),
            UVAxis::default_left(),
            UVAxis::default_left(),
            UVAxis::default_back(),
            UVAxis::default_back(),
        ];
        let bounds = Bounds::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        let normals: Vec<_> =
            Map::cube_dev(bounds).sides.iter().map(|s| s.plane.normal()).collect();
        let uv_axes: Vec<_> = normals.iter().map(UVAxis::from_norm_align_world).collect();
        for (truth, uv_axes) in truth.iter().zip(uv_axes) {
            assert_eq!(truth, &uv_axes);
        }
    }

    #[test]
    #[rustfmt::skip]
    fn uv_normal() {
        // let truth = [
        //     UVAxis::default_top(),
        //     UVAxis::default_top(),
        //     UVAxis::default_left(),
        //     UVAxis::default_left(),
        //     UVAxis::default_back(),
        //     UVAxis::default_back(),
        // ];
        // let bounds = Bounds::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        // let normals: Vec<_> =
        //     Map::cube_dev(bounds).sides.iter().map(|s| s.plane.normal()).collect();
        // for (truth, normal) in truth.iter().zip(normals) {
        //     let uv_axes = UVAxis::from_dir(&normal);
        //     eprintln!("truth   {truth:?}");
        //     eprintln!("uv_axes {uv_axes:?}");
        //     assert_eq!(truth, &uv_axes);
        //     eprintln!();
        // }
        eprintln!("END OF EASY\n");

        use std::f32::consts::FRAC_1_SQRT_2;
        let truth = [
            (UVAxis::new(-0.707107, 0.707107, 0.0, 0.0, 0.25), UVAxis::new(0.408248, 0.408248, -0.816497, 0.0, 0.25)),
            (UVAxis::new(1.0, 0.0, 0.0, 0.0, MAT_SCALE), UVAxis::new(0.0, -FRAC_1_SQRT_2, -FRAC_1_SQRT_2, 0.0, MAT_SCALE)),
            (UVAxis::new(1.0, 0.0, 0.0, 0.0, MAT_SCALE), UVAxis::new(0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2, 0.0, MAT_SCALE)),
        ];
        // wedge with slope facing back, up
        let input = [Vector3::new(1.0, 1.0, 1.0).normalize(), Vector3::new(0.0, 1.0, 1.0).normalize(), Vector3::new(0.0, -1.0, -1.0).normalize()];

        // for (truth, normal) in truth.iter().zip(input) {
        //     let uv_axes = UVAxis::from_dir(&normal);
        //     eprintln!("truth   {truth:?}");
        //     eprintln!("uv_axes {uv_axes:?}");
        //     if let Err(s) = about_equal(truth, &uv_axes) {
        //         eprintln!("\x1b[0;31mFAIL:    {s}\x1b[0m");
        //     } else {
        //         eprintln!("\x1b[0;32mPASS\x1b[0m");
        //     }
        //     eprintln!();
        // }
        // panic!()
        fn about_equal(lhs: &(UVAxis<f32>, UVAxis<f32>), rhs: &(UVAxis<f32>, UVAxis<f32>)) -> Result<(), String> {
            let mut failed_msg = String::new();
            (!abs_diff_eq!(lhs.0.x, rhs.0.x, epsilon = 0.01)).then(|| failed_msg += "u.x, ");
            (!abs_diff_eq!(lhs.0.y, rhs.0.y, epsilon = 0.01)).then(|| failed_msg += "u.y, ");
            (!abs_diff_eq!(lhs.0.z, rhs.0.z, epsilon = 0.01)).then(|| failed_msg += "u.z, ");
            (!abs_diff_eq!(lhs.0.trans, rhs.0.trans, epsilon = 0.01)).then(|| failed_msg += "u.trans, ");
            (!abs_diff_eq!(lhs.0.scale, rhs.0.scale, epsilon = 0.01)).then(|| failed_msg += "u.scale, ");
            if !failed_msg.is_empty() {
                failed_msg += "\r\x1b[48C";
            }
            (!abs_diff_eq!(lhs.1.x, rhs.1.x, epsilon = 0.01)).then(|| failed_msg += "v.x, ");
            (!abs_diff_eq!(lhs.1.y, rhs.1.y, epsilon = 0.01)).then(|| failed_msg += "v.y, ");
            (!abs_diff_eq!(lhs.1.z, rhs.1.z, epsilon = 0.01)).then(|| failed_msg += "v.z, ");
            (!abs_diff_eq!(lhs.1.trans, rhs.1.trans, epsilon = 0.01)).then(|| failed_msg += "v.trans, ");
            (!abs_diff_eq!(lhs.1.scale, rhs.1.scale, epsilon = 0.01)).then(|| failed_msg += "v.scale, ");

            if failed_msg.is_empty() {
                Ok(())
            } else {
                Err(failed_msg)
            }
        }

        // rip fingers :'(
        // TODO: scalene tetrahedron
        // TODO: axis aligned
        // AWK TO THIS
        let planes = [
            // triangles
            Plane::new(Vector3::new(0.0, 0.0, 128.0), Vector3::new(0.0, 128.0, 0.0), Vector3::new(128.0, 0.0, 0.0)),
            Plane::new(Vector3::new(0.0, 0.0, 128.0), Vector3::new(128.0, 0.0, 0.0), Vector3::new(0.0, -128.0, 0.0)),
            Plane::new(Vector3::new(0.0, 0.0, 128.0), Vector3::new(0.0, -128.0, 0.0), Vector3::new(-128.0, 0.0, 0.0)),
            Plane::new(Vector3::new(0.0, 0.0, 128.0), Vector3::new(-128.0, 0.0, 0.0), Vector3::new(0.0, 128.0, 0.0)),
            Plane::new(Vector3::new(0.0, 0.0, -128.0), Vector3::new(0.0, -128.0, 0.0), Vector3::new(128.0, 0.0, 0.0)),
            Plane::new(Vector3::new(0.0, 0.0, -128.0), Vector3::new(128.0, 0.0, 0.0), Vector3::new(0.0, 128.0, 0.0)),
            Plane::new(Vector3::new(0.0, 0.0, -128.0), Vector3::new(-128.0, 0.0, 0.0), Vector3::new(0.0, -128.0, 0.0)),
            Plane::new(Vector3::new(-128.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -128.0), Vector3::new(0.0, 128.0, 0.0)),
            // wedges
            Plane::new(Vector3::new(64.0, 0.0, 0.0), Vector3::new(64.0, -64.0, 0.0), Vector3::new(0.0, -64.0, 64.0)),
            Plane::new(Vector3::new(0.0, -64.0, 0.0), Vector3::new(-64.0, -64.0, 0.0), Vector3::new(-64.0, 0.0, 64.0)),
            Plane::new(Vector3::new(-64.0, 0.0, 0.0), Vector3::new(-64.0, 64.0, 0.0), Vector3::new(0.0, 64.0, 64.0)),
            Plane::new(Vector3::new(0.0, 64.0, 0.0), Vector3::new(64.0, 64.0, 0.0), Vector3::new(64.0, 0.0, 64.0)),
            Plane::new(Vector3::new(0.0, 0.0, -64.0), Vector3::new(0.0, -64.0, -64.0), Vector3::new(64.0, -64.0, 0.0)),
            Plane::new(Vector3::new(0.0, 0.0, -64.0), Vector3::new(-64.0, 0.0, -64.0), Vector3::new(-64.0, -64.0, 0.0)),
            Plane::new(Vector3::new(0.0, 0.0, -64.0), Vector3::new(0.0, 64.0, -64.0), Vector3::new(-64.0, 64.0, 0.0)),
            Plane::new(Vector3::new(0.0, 0.0, -64.0), Vector3::new(64.0, 0.0, -64.0), Vector3::new(64.0, 64.0, 0.0)),
        ];

        let truth = [
            // slope
            (UVAxis::new(-0.707107, 0.707107, 0.0, 0.0, 0.25), UVAxis::new(0.408248, 0.408248, -0.816497, 0.0, 0.25)),
            (UVAxis::new(0.707107, 0.707107, 0.0, 0.0, 0.25), UVAxis::new(0.408248, -0.408248, -0.816497, 0.0, 0.25)),
            (UVAxis::new(0.707107, -0.707107, 0.0, 0.0, 0.25), UVAxis::new(-0.408248, -0.408248, -0.816497, 0.0, 0.25)),
            (UVAxis::new(-0.707107, -0.707107, 0.0, 0.0, 0.25), UVAxis::new(-0.408248, 0.408248, -0.816497, 0.0, 0.25)),
            (UVAxis::new(0.707107, 0.707107, 0.0, 0.0, 0.25), UVAxis::new(-0.408248, 0.408248, -0.816497, 0.0, 0.25)),
            (UVAxis::new(-0.707107, 0.707107, 0.0, 0.0, 0.25), UVAxis::new(-0.408248, -0.408248, -0.816497, 0.0, 0.25)),
            (UVAxis::new(0.707107, -0.707107, 0.0, 0.0, 0.25), UVAxis::new(0.408248, 0.408248, -0.816497, 0.0, 0.25)),
            (UVAxis::new(-0.707107, -0.707107, 0.0, 0.0, 0.25), UVAxis::new(0.408248, -0.408248, -0.816497, 0.0, 0.25)),
            // wedges
            (UVAxis::new(0.0, 1.0, 0.0, 0.0, 0.25), UVAxis::new(0.707107, 0.0, -0.707107, 0.0, 0.25)),
            (UVAxis::new(1.0, 0.0, 0.0, 0.0, 0.25), UVAxis::new(0.0, -0.707107, -0.707107, 0.0, 0.25)),
            (UVAxis::new(0.0, -1.0, 0.0, 0.0, 0.25), UVAxis::new(-0.707107, 0.0, -0.707107, 0.0, 0.25)),
            (UVAxis::new(-1.0, 0.0, 0.0, 0.0, 0.25), UVAxis::new(0.0, 0.707107, -0.707107, 0.0, 0.25)),
            (UVAxis::new(0.0, 1.0, 0.0, 0.0, 0.25), UVAxis::new(-0.707107, 0.0, -0.707107, 0.0, 0.25)),
            (UVAxis::new(1.0, 0.0, 0.0, 0.0, 0.25), UVAxis::new(0.0, 0.707107, -0.707107, 0.0, 0.25)),
            (UVAxis::new(0.0, -1.0, 0.0, 0.0, 0.25), UVAxis::new(0.707107, 0.0, -0.707107, 0.0, 0.25)),
            (UVAxis::new(-1.0, 0.0, 0.0, 0.0, 0.25), UVAxis::new(0.0, -0.707107, -0.707107, 0.0, 0.25)),
        ];
        let mut any_failed = false;
        for (truth, plane) in truth.iter().zip(planes.iter()) {
            let normal = plane.normal_dir();
            let uv = UVAxis::from_norm(&normal);
            eprintln!("truth\t(u {},\tv {})", truth.0, truth.1);
            eprintln!("result\t(u {},\tv {})", uv.0, uv.1);
            if let Err(s) = about_equal(truth, &uv) {
                any_failed = true;
                eprintln!("\x1b[0;31mFAIL:    {s}\x1b[0m");
            } else {
                eprintln!("\x1b[0;32mPASS\x1b[0m");
            }
            eprintln!();
        }
        if any_failed {
            panic!()
        }
    }
}
