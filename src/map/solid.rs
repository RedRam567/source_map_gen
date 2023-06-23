//! Solids, brushes, peices of map geometry.

use crate::generation2::disp::Displacement;
use crate::generation2::SolidOptions;
use crate::prelude::*;
use crate::utils::Vec2d;
use std::fmt::Display;

/// A peice of map geometry made out of [`Side`]s.
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
///
/// See also: [`Displacement`] and [`DispInfo`]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Side<'a> {
    pub plane: Plane,
    pub texture: Texture<'a>,
    pub disp: Option<Displacement>,
}

impl<'a> Side<'a> {
    pub const fn new(plane: Plane, texture: Texture<'a>) -> Self {
        Self { plane, texture, disp: None }
    }

    pub fn new_verts(
        bl: Vector3<f32>, tl: Vector3<f32>, tr: Vector3<f32>, material: &Material<'a>,
        options: &SolidOptions,
    ) -> Self {
        Plane::new(bl, tl, tr).with_mat_align(material, options.world_align)
    }

    /// Translates each of the [`Plane`]s by a [`Vector3`].
    pub fn translate_mut(&mut self, trans: &Vector3<f32>) -> &mut Self {
        self.plane.translate_mut(trans);
        self
    }
}

// TODO: location?
/// TODO:DOCS:
///
/// See also: <https://developer.valvesoftware.com/wiki/.vmf#Dispinfo>
#[derive(Clone, Debug, PartialEq)]
pub struct DispInfo {
    // not sure if i32 or u32 in the valve impl
    /// > power `(2,3,4)`
    /// Used to calculate the number of rows and columns, only the three given values will compile
    /// properly. If omitted Hammer assumes 4. If the data is built around another power value, the
    /// vertex data will be squished into the starting corner.
    pub power: u32,
    /// > startposition `(vertex)`
    /// The position of the bottom left corner in an actual x y z position.
    pub start_position: Vector3<f32>,
    /// TODO: not in valve dev wiki
    /// See associated constants: TODO:DOCS:
    pub flags: i32,
    /// > elevation `(float)`
    /// A universal displacement in the direction of the vertex's normal added to all of the points.
    pub elevation: f32,
    /// > Subdiv `(bool)`
    /// Marks whether or not the displacement is being sub divided.
    pub is_subdiv: bool,
    /// > The Class defines the normal line for each vertex. This is a line that points outwards
    /// from the face and is used for light shading and vertex placement. Each column contains a
    /// group of 3 values.
    ///
    /// Basically the direction to apply `distances` to from the "ideal" location. (I THINK)
    pub normals: Vec2d<Vector3<f32>>,
    /// > The distance values represent how much the vertex is moved along the normal line. With an
    /// undefined normal line the distance is not applied.
    ///
    /// "0 0 0" is also "undefined".
    pub distances: Vec2d<f32>,
    /// > This Class lists all the default positions for each vertex in a displacement map. The
    /// distances are judged from the points the offsets Class defines. Each column contains a group
    /// of 3 values.
    ///
    /// > The offsets Class defines new default vertex positions relative to the original calculated
    /// positions. In the code sample above just the contents of this Class recreates an identical
    /// displacement map to the previous two Classes combined. Hammer does not modify these points
    /// in any way.
    ///
    /// Basically offsets the "ideal" location before other calculations. Unused by Hammer.
    pub offsets: Vec2d<Vector3<f32>>,
    /// > This Class is almost identical to the normals Class. The data structure is identical except
    /// the name, thus the code sample has been omitted to save space. The only differences lie in
    /// what it does and what it affects. This class defines the default normal lines that the
    /// normals Class is based from. It does not modify the offsets Class. Hammer has no options to
    /// modify these values, it can only set them based upon the faces orientation.
    ///
    /// Basically the face normal repeated like 289 times.
    pub offset_normals: Vec2d<Vector3<f32>>,
    /// > This Class contains a value for each vertex that represents how much of which texture to
    /// shown in blended materials. The difference between the values is merged linearly across the
    /// displacement map.
    pub alphas: Vec2d<f32>,
    /// > This Class contains information specific to each triangle in the displacement, rather than
    /// every vertex. This means that the size and number of its rows will differ. It follows the
    /// pattern of 2^n where n is the power. The row and column numbers are still defined from the
    /// starting point outwards.
    ///
    /// > The value assigned to the triangle represents its orientation. A value of "9" means the
    /// triangles has little or no slope in the z-axis. A value of "1" means it has a significant
    /// slope in the z-axis but a player is still able to walk on it. A value of "0" means it has a
    /// large slope in the z-axis and the player cannot walk up it. Any value is accepted by Hammer,
    /// a value of "2" will even produce green highlights in the Display walkable area view, but any
    /// such values will not compile.
    ///
    /// Basically only for Hammer (I THINK)
    /// - 9 => None or little slope.
    /// - 1 => Large walkable slope.
    /// - 0 => Large unwalkable slope.
    pub triangle_tags: Vec2d<i32>,
    /// > This affects the in-game tesselation of the displacement map, stating which vertices share
    /// an edge with another displacement map but do not share a vertex.
    /// A set of binary flags which corresponds to which vertices are allowed in the displacement
    /// map. A false flag removes the vertex from the compiled map. Note that -1 is all bits set to
    /// true.
    ///
    /// Pretty sure this can just be ignored. For sewing displacements of differing power. Its
    /// either a miniscule optimization to save like 5 triangles or a safety net / UX feature maybe
    ///
    /// All zeros means to remove all verts; vbsp will only remove some near edge and not even all
    /// of them (huh). Also last i32 always seems to be -1 / has no effect.
    pub allowed_verts: [i32; 10],
}

impl DispInfo {
    /// All collision enabled.
    /// Info for flags: <https://developer.valvesoftware.com/wiki/Hammer_Face_Edit_Disps#Attributes>
    pub const NO_FLAGS: i32 = 0;
    /// > Disables any physics objects colliding with the displacement. Useful for snow, mud, etc.
    pub const NO_PHYS_COLLISION: i32 = 2;
    /// > Disables any player or NPC collisions with the displacement.
    pub const NO_HULL_COLLISION: i32 = 4;
    /// > Disables raycasts colliding with the displacement. Gunfire and bullets will not collide with the displacement surface.
    pub const NO_RAY_COLLISION: i32 = 8;
    /// Disable all collision. Bitwise or of other flags.
    pub const NO_COLLISION: i32 =
        DispInfo::NO_PHYS_COLLISION | DispInfo::NO_HULL_COLLISION | DispInfo::NO_RAY_COLLISION;
}

// TODO: VERIFY, IMPORTANT
// The Valve wiki agrees with that / is kinda vague but in practice it seems like the opposite.
/// A flat plane in 3d space.
///
/// When looking directly at the plane, `bottom_left` will be in the bottom left
/// and so on, with the normal being towards you.
/// See also: <https://developer.valvesoftware.com/wiki/Valve_Map_Format#Planes>.
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
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

        let mut texture = material.clone().into_texture();
        texture.uaxis = uaxis;
        texture.vaxis = vaxis;
        Side::new(self, texture)
    }

    pub fn with_mat_align_world<'a>(self, material: &Material<'a>) -> Side<'a> {
        let normal = self.normal_dir();
        let (uaxis, vaxis) = UVAxis::from_norm_align_world(&normal);

        let mut texture = material.clone().into_texture();
        texture.uaxis = uaxis;
        texture.vaxis = vaxis;
        Side::new(self, texture)
    }

    pub fn with_mat_align<'a>(self, material: &Material<'a>, world_align: bool) -> Side<'a> {
        let normal = self.normal_dir();
        let (uaxis, vaxis) = if world_align {
            UVAxis::from_norm_align_world(&normal)
        } else {
            UVAxis::from_norm(&normal)
        };

        let mut texture = material.clone().into_texture();
        texture.uaxis = uaxis;
        texture.vaxis = vaxis;
        Side::new(self, texture)
    }

    /// Calculates the 4th point of the plane as if it is a parallelogram.
    pub fn bottom_right(&self) -> Vector3<f32> {
        //  line BA -> something -> line CD
        let Self { bottom_left: a, top_left: b, top_right: c } = self;
        let slope = b.clone() - a;
        -(slope - c)
    }

    /// Returns 4 points in the order: `bottom_left`, `top_left`, `top_right`, `bottom_right`.
    pub fn four_points(&self) -> [Vector3<f32>; 4] {
        let Self { bottom_left, top_left, top_right } = self.clone();
        [bottom_left, top_left, top_right, self.bottom_right()]
    }

    // TODO: wtf is this?
    pub fn top(z: f32) -> Self {
        const VALUE: f32 = 64.0;
        // from Bounds::top_plane where its Bounds::new(-64,-64,_, 64,64,z)
        Self::new(
            Vector3::new(-VALUE, VALUE, z),
            Vector3::new(VALUE, VALUE, z),
            Vector3::new(VALUE, -VALUE, z),
        )
    }

    // TODO: wtf is this?
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
    fn plane_to_4() {
        let truth = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
        ];
        let input = Plane::new(truth[0].clone(), truth[1].clone(), truth[2].clone());
        let output = input.four_points();
        assert_eq!(truth, output);
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
