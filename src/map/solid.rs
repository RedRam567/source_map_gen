//! Solids, brushes, peices of map geometry.

use crate::generation2::SolidOptions;
use crate::prelude::*;
use std::fmt::Display;

/// A peice of map geometry made out of sides. Ex: a cube, cylinder.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Solid<'a> {
    pub sides: Vec<Side<'a>>,
}

impl<'a> Solid<'a> {
    pub const fn new(sides: Vec<Side<'a>>) -> Self {
        Self { sides }
    }

    /// Translates the inner [`Plane`]. TODO: also translate texture.
    pub fn translate_mut(&mut self, trans: &Vector3<f32>) -> &mut Self {
        for side in self.sides.iter_mut() {
            side.translate_mut(trans);
        }
        self
    }

    /// Translates the inner [`Plane`]. Does not maintain allignment of textures relative to the [`Solid`].
    pub fn translate_mut_ignore_texture(&mut self, trans: &Vector3<f32>) -> &mut Self {
        for side in self.sides.iter_mut() {
            side.translate_mut(trans);
        }
        self
    }
}

/// A side of a [`Solid`].
#[derive(Clone, Debug, PartialEq)]
pub struct Side<'a> {
    pub plane: Plane,
    pub texture: Texture<'a>,
}

// TODO: normal, slope, wtf is default

impl<'a> Side<'a> {
    pub const fn new(plane: Plane, texture: Texture<'a>) -> Self {
        Self { plane, texture }
    }

    pub fn new_verts(
        bl: Vector3<f32>, tl: Vector3<f32>, tr: Vector3<f32>, material: &Material<'a>,
        options: &SolidOptions,
    ) -> Self {
        Plane::new(bl, tl, tr)
            .with_mat_align(material, options.world_align)
    }

    /// Translates each of the [`Plane`]s by a [`Vector3`].
    pub fn translate_mut(&mut self, trans: &Vector3<f32>) -> &mut Self {
        self.plane.translate_mut(trans);
        self
    }
}

// The Valve wiki agrees with that / is kinda vague but in practice it seems like the opposite.
/// A flat geometric plane.
/// When looking directly at the plane, `bottom_left` will be in the bottom left
/// and so on, with the normal being towards you.
/// See <https://developer.valvesoftware.com/wiki/Valve_Map_Format#Planes>.
#[derive(Clone, Debug, PartialEq)]
pub struct Plane {
    pub bottom_left: Vector3<f32>,
    pub top_left: Vector3<f32>,
    pub top_right: Vector3<f32>,
}

impl Plane {
    pub const fn new(
        bottom_left: Vector3<f32>, top_left: Vector3<f32>, top_right: Vector3<f32>,
    ) -> Self {
        Self { bottom_left, top_left, top_right }
    }

    pub fn new_with_round(
        mut bl: Vector3<f32>, mut tl: Vector3<f32>, mut tr: Vector3<f32>, allow_frac: bool,
    ) -> Self {
        if !allow_frac {
            bl = bl.round();
            tl = tl.round();
            tr = tr.round();
        }
        Self::new(bl, tl, tr)
    }

    /// Translates by adding a [`Vector3`] to each of the points.
    pub fn translate_mut(&mut self, trans: &Vector3<f32>) -> &mut Self {
        self.bottom_left += trans;
        self.top_left += trans;
        self.top_right += trans;
        self
    }

    /// Returns the non-normalized normal.
    /// See [`Plane`].
    pub fn normal_dir(&self) -> Vector3<f32> {
        // reverse order, outwards from a cube
        let a = &self.top_right;
        let b = &self.top_left;
        let c = &self.bottom_left;

        (b.clone() - a).cross(&(c.clone() - a))
    }

    /// Returns the normalized normal.
    /// See [`Plane`].
    pub fn normal(&self) -> Vector3<f32> {
        self.normal_dir().normalize()
    }

    pub fn with_texture<'a>(self, texture: &Texture<'a>) -> Side<'a> {
        Side::new(self, texture.clone())
    }

    pub fn with_mat<'a>(self, material: &Material<'a>) -> Side<'a> {
        let normal = self.normal_dir();
        let (uaxis, vaxis) = UVAxis::from_norm(&normal);
        let Material { material, light_scale } = material.clone();
        Side::new(self, Texture::new(material, uaxis, vaxis, light_scale))
    }

    pub fn with_mat_align_world<'a>(self, material: &Material<'a>) -> Side<'a> {
        let normal = self.normal_dir();
        let (uaxis, vaxis) = UVAxis::from_norm_align_world(&normal);
        let Material { material, light_scale } = material.clone();
        Side::new(self, Texture::new(material, uaxis, vaxis, light_scale))
    }

    pub fn with_mat_align<'a>(self, material: &Material<'a>, world_align: bool) -> Side<'a> {
        let normal = self.normal_dir();
        let (uaxis, vaxis) = if world_align {
            UVAxis::from_norm_align_world(&normal)
        } else {
            UVAxis::from_norm(&normal)
        };
        let Material { material, light_scale } = material.clone();
        Side::new(self, Texture::new(material, uaxis, vaxis, light_scale))
    }

    pub fn top(z: f32) -> Self {
        const VALUE: f32 = 64.0;
        // from Bounds::top_plane where its Bounds::new(-64,-64,_, 64,64,z)
        Self::new(
            Vector3::new(-VALUE, VALUE, z),
            Vector3::new(VALUE, VALUE, z),
            Vector3::new(VALUE, -VALUE, z),
        )
    }

    pub fn bottom(z: f32) -> Self {
        const VALUE: f32 = 64.0;
        // from Bounds::bottom_plane where its Bounds::new(-64,-64,z, 64,64,_)
        Self::new(
            Vector3::new(VALUE, VALUE, z),
            Vector3::new(-VALUE, VALUE, z),
            Vector3::new(-VALUE, -VALUE, z),
        )
    }
}

impl Display for Plane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) ({}) ({})", self.bottom_left, self.top_left, self.top_right)
    }
}

#[allow(clippy::approx_constant)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{generation::Bounds, map::Map};
    use approx::abs_diff_eq;

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
            UVAxis::default_east(),
            UVAxis::default_east(),
            UVAxis::default_north(),
            UVAxis::default_north(),
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
    #[rustfmt::skip] // lol no
    fn uv_normal() {
        // axis aligned
        let truth = [
            UVAxis::default_top(),
            UVAxis::default_top(),
            UVAxis::default_east(),
            UVAxis::default_east(),
            UVAxis::default_north(),
            UVAxis::default_north(),
        ];
        let bounds = Bounds::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        let normals: Vec<_> =
            Map::cube_dev(bounds).sides.iter().map(|s| s.plane.normal()).collect();
        for (truth, normal) in truth.iter().zip(normals) {
            let uv_axes = UVAxis::from_norm(&normal);
            assert_eq!(truth, &uv_axes);
        }

        // rip fingers :'(
        // TODO: scalene tetrahedron
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
            // triangles
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
    }
}
