//! Module handles generating a map

// mod proc

// pub fn cube
// pub fn cube text
// pub fn cube texts

pub const MAT_DEV_WALL: &str = "DEV/DEV_MEASUREWALL01C";
pub const MAT_DEV_FLOOR: &str = "DEV/DEV_MEASUREGENERIC01B";
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

use crate::map::{Map, Plane, Vector3, Side, Solid, Texture, TextureBuilder};
use std::borrow::Cow;

/// Bounds in 3d space.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bounds<T> {
    pub min: Vector3<T>,
    pub max: Vector3<T>,
}

impl<T> Bounds<T> {}

impl Bounds<f32> {
    /// Create bounds from any two points in 3d space.
    pub fn new(point1: Vector3<f32>, point2: Vector3<f32>) -> Self {
        let max = Vector3 {
            x: point1.x.max(point2.x),
            y: point1.y.max(point2.y),
            z: point1.z.max(point2.z),
        };
        Self {
            min: Vector3 {
                x: point1.x.min(point2.x),
                y: point1.y.min(point2.y),
                z: point1.z.min(point2.z),
            },
            max,
        }
    }
}

impl Bounds<f32> {
    /// returns the 8 vertexes of the bounds in the order, relative to a view from the back, looking at the front:
    ///
    /// `bottom back left`, `bottom front left`, `bottom front right`, `bottom back right`, `top back left`, `top front left`, `top front right`, `top back right`,
    pub const fn verts(&self) -> [Vector3<f32>; 8] {
        // relative to back view looking front, points go counter clockwise from top looking down
        [
            Vector3 { ..self.min },                               // 0 bottom back  left
            Vector3 { y: self.max.y, ..self.min },                // 1 bottom front left
            Vector3 { x: self.max.x, y: self.max.y, ..self.min }, // 2 bottom front right
            Vector3 { x: self.max.x, ..self.min },                // 3 bottom back  right
            Vector3 { x: self.min.x, y: self.min.y, ..self.max }, // 4 top    back  left
            Vector3 { x: self.min.x, ..self.max },                // 5 top    front left
            Vector3 { ..self.max },                             // 6 top    front right
            Vector3 { y: self.min.y, ..self.max },                // 7 top    back  right
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vmf::{ToBlock, VmfState};

    #[test]
    fn cube() {
        let bounds = Bounds::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        // let bounds = Bounds::new(Vector3::new(-128.0, -192.0, -64.0), Vector3::new(384f32, 320.0, 0.0));
        let cube = Map::cube_dev1(bounds);
        let mut state = VmfState::default();
        let vmf = cube.to_block(&mut state);
        // println!("{vmf}");
        // eprintln!("{vmf}");
        // dbg!(cube);
        // panic!()
    }

    #[test]
    fn test() {
    }
}
