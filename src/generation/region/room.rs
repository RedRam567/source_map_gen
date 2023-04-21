use super::{Bounds, WALL_THICKNESS};
use crate::map::{Map, Solid};

#[derive(Clone, Debug, PartialEq)]
pub struct Room<'a> {
    pub bounds: Bounds<f32>,
    pub connections: Vec<&'a Room<'a>>,
}

impl<'a> Room<'a> {
    pub const fn new(bounds: Bounds<f32>) -> Self {
        Self { bounds, connections: vec![] }
    }

    // TODO: better, mods, door carve
    pub fn construct(&self, map: &mut Map) {
        let mut top = self.bounds.clone();
        top.min.z = top.max.z - WALL_THICKNESS;
        map.add_solid(Map::cube_dev(top));
        let mut bottom = self.bounds.clone();
        bottom.max.z = bottom.min.z + WALL_THICKNESS;
        map.add_solid(Map::cube_dev(bottom));

        let mut front = self.bounds.clone();
        front.min.y = front.max.y - WALL_THICKNESS;
        map.add_solid(Map::cube_dev(front));
        let mut back = self.bounds.clone();
        back.max.y = back.min.y + WALL_THICKNESS;
        map.add_solid(Map::cube_dev(back));

        let mut right = self.bounds.clone();
        right.min.x = right.max.x - WALL_THICKNESS;
        map.add_solid(Map::cube_dev(right));
        let mut left = self.bounds.clone();
        left.max.x = left.min.x + WALL_THICKNESS;
        map.add_solid(Map::cube_dev(left));
    }

    pub(crate) fn construct_sky(&self) -> [Solid<'a>; 6] {
        let mut top = self.bounds.clone();
        top.min.z = top.max.z - WALL_THICKNESS;
        let top = Map::cube_sky(top);
        let mut bottom = self.bounds.clone();
        bottom.max.z = bottom.min.z + WALL_THICKNESS;
        let bottom = Map::cube_sky(bottom);

        let mut front = self.bounds.clone();
        front.min.y = front.max.y - WALL_THICKNESS;
        let front = Map::cube_sky(front);
        let mut back = self.bounds.clone();
        back.max.y = back.min.y + WALL_THICKNESS;
        let back = Map::cube_sky(back);

        let mut right = self.bounds.clone();
        right.min.x = right.max.x - WALL_THICKNESS;
        let right = Map::cube_sky(right);
        let mut left = self.bounds.clone();
        left.max.x = left.min.x + WALL_THICKNESS;
        let left = Map::cube_sky(left);

        [top, bottom, front, back, right, left]
    }

    pub(crate) fn construct_sky_inside(&self) -> [Solid<'a>; 6] {
        let mut top = self.bounds.clone();
        top.min.z = top.max.z;
        top.max.z = top.min.z + WALL_THICKNESS;
        let top = Map::cube_sky(top);
        let mut bottom = self.bounds.clone();
        bottom.max.z = bottom.min.z;
        bottom.min.z = bottom.max.z - WALL_THICKNESS;
        let bottom = Map::cube_sky(bottom);

        let mut front = self.bounds.clone();
        front.min.y = front.max.y;
        front.max.y = front.min.y + WALL_THICKNESS;
        let front = Map::cube_sky(front);
        let mut back = self.bounds.clone();
        back.max.y = back.min.y;
        back.min.y = back.max.y - WALL_THICKNESS;
        let back = Map::cube_sky(back);

        let mut left = self.bounds.clone();
        left.min.x = left.max.x;
        left.max.x = left.min.x + WALL_THICKNESS;
        let left = Map::cube_sky(left);
        let mut right = self.bounds.clone();
        right.max.x = right.min.x;
        right.min.x = right.max.x - WALL_THICKNESS;
        let right = Map::cube_sky(right);

        [top, bottom, front, back, right, left]
    }

    pub(crate) fn construct_dev_inside(&self) -> [Solid<'a>; 6] {
        let mut top = self.bounds.clone();
        top.min.z = top.max.z;
        top.max.z = top.min.z + WALL_THICKNESS;
        let top = Map::cube_dev(top);
        let mut bottom = self.bounds.clone();
        bottom.max.z = bottom.min.z;
        bottom.min.z = bottom.max.z - WALL_THICKNESS;
        let bottom = Map::cube_dev(bottom);

        let mut front = self.bounds.clone();
        front.min.y = front.max.y;
        front.max.y = front.min.y + WALL_THICKNESS;
        let front = Map::cube_dev(front);
        let mut back = self.bounds.clone();
        back.max.y = back.min.y;
        back.min.y = back.max.y - WALL_THICKNESS;
        let back = Map::cube_dev(back);

        let mut left = self.bounds.clone();
        left.min.x = left.max.x;
        left.max.x = left.min.x + WALL_THICKNESS;
        let left = Map::cube_dev(left);
        let mut right = self.bounds.clone();
        right.max.x = right.min.x;
        right.min.x = right.max.x - WALL_THICKNESS;
        let right = Map::cube_dev(right);

        [top, bottom, front, back, right, left]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Vector3;

    #[ignore]
    #[test]
    fn room() {
        let mut map = Map::default();
        let room = Room::new(Bounds {
            min: Vector3::new(-512.0, -512.0, -128.0),
            max: Vector3::new(512.0, 512.0, -64.0),
        });
        room.construct(&mut map);
        dbg!(map);
        panic!()
    }
}
