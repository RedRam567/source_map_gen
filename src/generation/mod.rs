//! Module handles generating a map

// mod proc

// pub fn cube
// pub fn cube text
// pub fn cube texts

use crate::{map::{Map, Plane, Point, Side, Solid, Texture, TextureBuilder}, MAT_DEV_FLOOR};

/// Bounds in 3d space.
pub struct Bounds<T> {
    pub min: Point<T>,
    pub max: Point<T>,
}

impl<T> Bounds<T> {}

impl Bounds<f32> {
    /// Create bounds from any two points in 3d space.
    pub fn new(point1: Point<f32>, point2: Point<f32>) -> Self {
        let max = Point {
            x: point1.x.max(point2.x),
            y: point1.y.max(point2.y),
            z: point1.z.max(point2.z),
        };
        Self {
            min: Point {
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
    pub fn verts(&self) -> [Point<f32>; 8] {
        // relative to back view looking front, points go counter clockwise from top looking down
        [
            self.min.clone(),                                   // 0 bottom back  left
            Point { y: self.max.y, ..self.min },                // 1 bottom front left
            Point { x: self.max.x, y: self.max.y, ..self.min }, // 2 bottom front right
            Point { x: self.max.x, ..self.min },                // 3 bottom back  right
            Point { x: self.min.x, y: self.min.y, ..self.max }, // 4 top    back  left
            Point { x: self.min.x, ..self.max },                // 5 top    front left
            self.max.clone(),                                   // 6 top    front right
            Point { y: self.min.y, ..self.max },                // 7 top    back  right
        ]
    }

    /// Returns the six planes of bounds in the order:
    ///
    /// `top`, `bottom`, `left`, `right`, `back`, `front`
    pub fn planes(&self) -> [Plane; 6] {
        let verts = self.verts();
        let top = Plane::new(verts[4].clone(), verts[5].clone(), verts[6].clone());
        let bottom = Plane::new(verts[2].clone(), verts[1].clone(), verts[0].clone());
        let left = Plane::new(verts[1].clone(), verts[5].clone(), verts[4].clone());
        let right = Plane::new(verts[3].clone(), verts[7].clone(), verts[6].clone());
        let back = Plane::new(verts[0].clone(), verts[4].clone(), verts[7].clone());
        let front = Plane::new(verts[2].clone(), verts[6].clone(), verts[5].clone());
        [top, bottom, left, right, back, front]
    }
}

impl Map {
    pub fn cube_dev(bounds: Bounds<f32>) -> Solid {
        // let verts = bounds.verts();
        // dbg!(&verts);

        // hammer seems to use plane points that start at different vertexes
        // example back face starts in "bottom right"
        // mine always starts in "bottom left" expect for maybe bottom idk
        // shouldn't matter tho, any 3 points on the same plane should workd
        // (in clockwise order, also 90 degrees stuff??)

        let planes = bounds.planes();
        let [top, bottom, left, right, back, front] = planes;

        let sides = vec![
            Side { plane: top, texture: TextureBuilder::new().top().mat(MAT_DEV_FLOOR).build() },
            Side { plane: bottom, texture: TextureBuilder::new().bottom().build() },
            Side { plane: left, texture: TextureBuilder::new().left().build() },
            Side { plane: right, texture: TextureBuilder::new().right().build() },
            Side { plane: back, texture: TextureBuilder::new().back().build() },
            Side { plane: front, texture: TextureBuilder::new().front().mat(MAT_DEV_FLOOR).build() },
        ];
        Solid { sides }
    }

    pub fn cube(bounds: Bounds<f32>, textures: &[Texture; 6]) -> Solid {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::vmf::{ToBlock, VmfState};

    use super::*;

    #[test]
    fn cube() {
        let bounds = Bounds::new(Point::new(-128.0, -192.0, -64.0), Point::new(384f32, 320.0, 0.0));
        let cube = Map::cube_dev(bounds);
        let mut state = VmfState::default();
        let vmf = cube.to_block(&mut state);
        println!("{vmf}");
        // eprintln!("{vmf}");
        // dbg!(cube);
        panic!()
    }
}
