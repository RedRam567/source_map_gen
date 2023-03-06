//! Module handles generating a map

pub mod region;

use region::*;

pub const MAT_DEV_WALL: &str = "DEV/DEV_MEASUREWALL01C";
pub const MAT_DEV_FLOOR: &str = "DEV/DEV_MEASUREGENERIC01B";
pub const MAT_3D_SKY: &str = "TOOLS/TOOLSSKYBOX";
pub const LIGHTMAP_SCALE: u8 = 16;
pub const MAT_SCALE: f32 = 0.25;

pub static DEV_TOP: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_DEV_FLOOR)).top().build();
pub static DEV_BOTTOM: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_DEV_FLOOR)).bottom().build();
pub static DEV_LEFT: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_DEV_WALL)).left().build();
pub static DEV_RIGHT: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_DEV_WALL)).right().build();
pub static DEV_BACK: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_DEV_WALL)).back().build();
pub static DEV_FRONT: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_DEV_WALL)).front().build();

pub static SKY_TOP: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_3D_SKY)).top().build();
pub static SKY_BOTTOM: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_3D_SKY)).bottom().build();
pub static SKY_LEFT: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_3D_SKY)).left().build();
pub static SKY_RIGHT: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_3D_SKY)).right().build();
pub static SKY_BACK: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_3D_SKY)).back().build();
pub static SKY_FRONT: Texture<'static> =
    TextureBuilder::new_mat(Cow::Borrowed(MAT_3D_SKY)).front().build();

use crate::map::{Map, Plane, Side, Solid, Texture, TextureBuilder, Vector3};
use std::borrow::Cow;

/// Bounds in 3d space.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bounds<T> {
    pub min: Vector3<T>,
    pub max: Vector3<T>,
}

impl Bounds<f32> {
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

    // if `other` collides with `self` return largest sub-bounds of `other`
    // that doesn't collide, else Ok
    pub fn collides(&self, other: &Self) -> bool {
        // you can just extend line collision over n dimensions by checking axes
        if lines_collides(self.min.x, self.max.x, other.min.x, other.max.x) {
            return true;
        };
        if lines_collides(self.min.y, self.max.y, other.min.y, other.max.y) {
            return true;
        };
        if lines_collides(self.min.z, self.max.z, other.min.z, other.max.z) {
            return true;
        };
        false
    }

    /// returns the 8 vertexes of the bounds in the order, relative to a view from the back, looking at the front:
    ///
    /// `left back bottom`, `left front bottom`, `right front bottom`, `right back bottom`
    ///
    /// `left back top `, `left front top `, `right front top `, `right back top `
    pub const fn verts(&self) -> [Vector3<f32>; 8] {
        // relative to back view looking front, points go counter clockwise from top looking down
        [
            Vector3 { ..self.min },                               // 0 left  back  bottom
            Vector3 { y: self.max.y, ..self.min },                // 1 left  front bottom
            Vector3 { x: self.max.x, y: self.max.y, ..self.min }, // 2 right front bottom
            Vector3 { x: self.max.x, ..self.min },                // 3 right back  bottom
            Vector3 { x: self.min.x, y: self.min.y, ..self.max }, // 4 left  back  top
            Vector3 { x: self.min.x, ..self.max },                // 5 left  front top
            Vector3 { ..self.max },                               // 6 right front top
            Vector3 { y: self.min.y, ..self.max },                // 7 right back  top
        ]
    }

    /// Returns the six planes of bounds in the order:
    ///
    /// `top`, `bottom`, `left`, `right`, `back`, `front`
    #[rustfmt::skip]
    pub const fn planes(&self) -> [Plane; 6] {
        // hammer seems to use plane points that start at different vertexes
        // example back face starts in "bottom right"
        // mine always starts in "bottom left" expect for maybe bottom idk
        // shouldn't matter tho, any 3 points on the same plane should workd
        // (in clockwise order, also 90 degrees stuff??)
        let verts = self.verts();
        let top = Plane::new(verts[4].const_clone(), verts[5].const_clone(), verts[6].const_clone());
        let bottom = Plane::new(verts[2].const_clone(), verts[1].const_clone(), verts[0].const_clone());
        let left = Plane::new(verts[1].const_clone(), verts[5].const_clone(), verts[4].const_clone());
        let right = Plane::new(verts[3].const_clone(), verts[7].const_clone(), verts[6].const_clone());
        let back = Plane::new(verts[0].const_clone(), verts[4].const_clone(), verts[7].const_clone());
        let front = Plane::new(verts[2].const_clone(), verts[6].const_clone(), verts[5].const_clone());
        [top, bottom, left, right, back, front]
    }
}

impl<'a> Map<'a> {
    // pub fn cube_dev(bounds: Bounds<f32>) -> Solid {
    //     let [top, bottom, left, right, back, front] = bounds.planes();

    //     let sides = vec![
    //         Side { plane: top, texture: TextureBuilder::new().top().mat(MAT_DEV_FLOOR).build() },
    //         Side { plane: bottom, texture: TextureBuilder::new().bottom().build() },
    //         Side { plane: left, texture: TextureBuilder::new().left().build() },
    //         Side { plane: right, texture: TextureBuilder::new().right().build() },
    //         Side { plane: back, texture: TextureBuilder::new().back().build() },
    //         Side {
    //             plane: front,
    //             texture: TextureBuilder::new().front().mat(MAT_DEV_FLOOR).build(),
    //         },
    //     ];
    //     Solid { sides }
    // }

    // TODO: use best
    pub fn cube_dev(bounds: Bounds<f32>) -> Solid<'a> {
        Self::cube_dev2(bounds)
    }

    pub fn cube_sky(bounds: Bounds<f32>) -> Solid<'a> {
        let textures = [
            SKY_TOP.clone(),
            SKY_BOTTOM.clone(),
            SKY_LEFT.clone(),
            SKY_RIGHT.clone(),
            SKY_BACK.clone(),
            SKY_FRONT.clone(),
        ];
        Self::cube_owned(bounds, textures)
    }

    pub fn cube_dev1(bounds: Bounds<f32>) -> Solid<'a> {
        let textures = [&DEV_TOP, &DEV_BOTTOM, &DEV_LEFT, &DEV_RIGHT, &DEV_BACK, &DEV_FRONT];
        Self::cube(bounds, textures)
    }

    pub fn cube_dev3(bounds: Bounds<f32>) -> Solid<'a> {
        let textures = Texture::cube_textures([
            MAT_DEV_FLOOR,
            MAT_DEV_FLOOR,
            MAT_DEV_WALL,
            MAT_DEV_WALL,
            MAT_DEV_WALL,
            MAT_DEV_WALL,
        ]);
        Self::cube_owned(bounds, textures)
    }

    pub fn cube_dev4(bounds: Bounds<f32>) -> Solid<'a> {
        let textures = [
            DEV_TOP.clone(),
            DEV_BOTTOM.clone(),
            DEV_LEFT.clone(),
            DEV_RIGHT.clone(),
            DEV_BACK.clone(),
            DEV_FRONT.clone(),
        ];
        Self::cube_owned(bounds, textures)
    }

    pub fn cube_dev2(bounds: Bounds<f32>) -> Solid<'a> {
        Self::cube_str(
            bounds,
            [MAT_DEV_FLOOR, MAT_DEV_FLOOR, MAT_DEV_WALL, MAT_DEV_WALL, MAT_DEV_WALL, MAT_DEV_WALL],
        )
    }

    #[inline]
    pub fn cube(bounds: Bounds<f32>, textures: [&Texture<'a>; 6]) -> Solid<'a> {
        Solid::new(
            bounds
                .planes()
                .into_iter()
                // this clone makes this function 10 times smaller
                .zip(textures.into_iter())
                .map(|(p, t)| Side::new(p, t.clone()))
                .collect(),
        )
    }

    #[inline]
    pub fn cube_owned(bounds: Bounds<f32>, textures: [Texture<'a>; 6]) -> Solid<'a> {
        Solid::new(
            bounds
                .planes()
                .into_iter()
                // this clone makes this function 10 times smaller
                .zip(textures.into_iter())
                .map(|(p, t)| Side::new(p, t))
                .collect(),
        )
    }

    #[inline]
    pub fn cube_str(bounds: Bounds<f32>, textures: [&str; 6]) -> Solid {
        let [top, bottom, left, right, back, front] = textures;

        let top = TextureBuilder::new_mat(Cow::Borrowed(top)).top().build();
        let bottom = TextureBuilder::new_mat(Cow::Borrowed(bottom)).bottom().build();
        let left = TextureBuilder::new_mat(Cow::Borrowed(left)).left().build();
        let right = TextureBuilder::new_mat(Cow::Borrowed(right)).right().build();
        let back = TextureBuilder::new_mat(Cow::Borrowed(back)).back().build();
        let front = TextureBuilder::new_mat(Cow::Borrowed(front)).front().build();

        let textures = [top, bottom, left, right, back, front];
        Solid::new(
            bounds
                .planes()
                .into_iter()
                // this clone makes this function 10 times smaller
                .zip(textures.into_iter())
                .map(|(p, t)| Side::new(p, t))
                .collect(),
        )
    }

    pub fn cube_ref(bounds: Bounds<f32>, textures: [&Texture<'a>; 6]) -> Solid<'a> {
        Solid::new(
            bounds
                .planes()
                .into_iter()
                // this clone makes this function 10 times smaller
                .zip(textures.into_iter())
                .map(|(p, t)| Side::new(p, t.clone()))
                .collect(),
        )
    }
}

/// Tests if two line segments overlap at all. "Barely touching" doesn't count.
#[inline]
fn lines_collides(x1: f32, x2: f32, y1: f32, y2: f32) -> bool {
    debug_assert!(x2 >= x1, "end must be greater or equal");
    debug_assert!(y2 >= y1, "end must be greater or equal");
    // end of first after start of other
    // start of first before end of other
    x2 > y1 && x1 < y2
}

/// Returns the amount two line segments are overlapping.
/// Will return zero if touching boundries and negatives if not overlapping.
/// Can sometimes return zero even if colliding (if start equals end).
#[inline]
fn lines_overlap(x1: f32, x2: f32, y1: f32, y2: f32) -> f32 {
    debug_assert!(x2 >= x1, "end must be greater or equal");
    debug_assert!(y2 >= y1, "end must be greater or equal");
    x2.min(y2) - x1.max(y1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{map::IdInfo, vmf::ToBlock};

    #[test]
    fn cube() {
        let bounds = Bounds::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        // let bounds = Bounds::new(Vector3::new(-128.0, -192.0, -64.0), Vector3::new(384f32, 320.0, 0.0));
        let cube = Map::cube_dev1(bounds);
        let mut state = IdInfo::default();
        let _vmf = cube.to_block(&mut state);
        // println!("{vmf}");
        // eprintln!("{vmf}");
        // dbg!(cube);
        // panic!()
    }

    #[test]
    fn collide() {
        // --2345--
        assert_1d(-1, 2, 5, 0, 1); // 01------
        assert_1d(0, 2, 5, 0, 2); // 012-----
        assert_1d(1, 2, 5, 1, 3); // -123----
        assert_1d(2, 2, 5, 2, 4); // --234---
        assert_1d(3, 2, 5, 2, 5); // --2345--
        assert_1d(2, 2, 5, 3, 5); // ---345--
        assert_1d(1, 2, 5, 4, 6); // ----456-
        assert_1d(0, 2, 5, 5, 7); // -----567
        assert_1d(-1, 2, 5, 6, 7); // ------67
        assert_1d(3, 2, 5, 0, 7); // 01234567

        assert_collide(true, 2.0, 5.0, 3.0, 3.0);
        // O_O collides but with length of zero O_O
        assert_length(0.0, 2.0, 5.0, 3.0, 3.0);

        fn assert_1d(truth: i32, x1: i32, x2: i32, y1: i32, y2: i32) {
            let truth_bool = truth > 0;
            let truth = truth as f32;
            let x1 = x1 as f32;
            let x2 = x2 as f32;
            let y1 = y1 as f32;
            let y2 = y2 as f32;
            eprintln!("truth: {truth: <5} input: {x1}, {x2}, {y1}, {y2}");

            assert_collide(truth_bool, x1, x2, y1, y2);
            assert_length(truth, x1, x2, y1, y2);
        }

        fn assert_collide(truth: bool, x1: f32, x2: f32, y1: f32, y2: f32) {
            assert_eq!(truth, lines_collides(x1, x2, y1, y2));
            assert_eq!(truth, lines_collides(y1, y2, x1, x2));
            // reverse order as end must be greater
            assert_eq!(truth, lines_collides(-x2, -x1, -y2, -y1));
            assert_eq!(truth, lines_collides(-y2, -y1, -x2, -x1));
        }

        fn assert_length(truth: f32, x1: f32, x2: f32, y1: f32, y2: f32) {
            assert_eq!(truth, lines_overlap(x1, x2, y1, y2));
            assert_eq!(truth, lines_overlap(y1, y2, x1, x2));
            // reverse order as end must be greater
            assert_eq!(truth, lines_overlap(-x2, -x1, -y2, -y1));
            assert_eq!(truth, lines_overlap(-y2, -y1, -x2, -x1));
        }

        fn eprint_return<T: std::fmt::Display>(value: T) -> T {
            eprintln!("value {value}");
            value
        }
    }
}
