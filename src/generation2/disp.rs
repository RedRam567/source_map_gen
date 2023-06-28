use crate::map::DispInfo;
use crate::utils::Vec2d;

use super::*;

// not pub
/// > The value assigned to the triangle represents its orientation.
///
/// Basicly the walkablity of the traingle. shown by Hammer with the
/// "Display walkable area" button (DW).
pub enum TriangleTag {
    // about 45 point smth
    /// > A value of "0" means it has a large slope in the z-axis and the player
    /// cannot walk up it.
    Unwalkable = 0,
    /// > A value of "1" means it has a significant slope in the z-axis but a
    /// player is still able to walk on it.
    Steep = 1,
    /// > "9" means the triangles has little or no slope in the z-axis
    Flat = 9,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Displacement {
    /// Number of points in 1 dimension
    pub width: usize,

    pub plane: Plane,
    pub bottom_right: Vector3<f32>,

    pub normals: Vec2d<Vector3<f32>>,
    pub distances: Vec2d<f32>,
    pub alphas: Vec2d<f32>,
}

impl Displacement {
    pub const fn new(corners: [Vector3<f32>; 4], width: usize) -> Self {
        // inlining corners leads to move errors O_o
        let [bl, tl, tr, br] = corners;
        Self {
            width,
            plane: Plane::new(bl, tl, tr),
            bottom_right: br,
            normals: Vec2d::new(Vec2d::strides(width)),
            distances: Vec2d::new(Vec2d::strides(width)),
            alphas: Vec2d::new(Vec2d::strides(width)),
        }
    }

    pub fn new_plane(plane: Plane, width: usize) -> Self {
        let bottom_right = plane.bottom_right();
        Self {
            width,
            plane,
            bottom_right,
            normals: Vec2d::new(Vec2d::strides(width)),
            distances: Vec2d::new(Vec2d::strides(width)),
            alphas: Vec2d::new(Vec2d::strides(width)),
        }
    }

    // TODO: order?
    /// Returns the four corners defining the plane. In the order:
    /// `bottom_left`, `top_left`, `top_right`, `bottom_right`
    #[inline]
    pub const fn corners(&self) -> [&Vector3<f32>; 4] {
        let Plane { bottom_left, top_left, top_right } = &self.plane;
        let bottom_right = &self.bottom_right;
        [bottom_left, top_left, top_right, bottom_right]
    }

    // TODO: fix nameing, size vs width vs len vs num points in 1 axis
    /// Get number of verts in one dimension.
    /// Squared is the total number of verts for the whole [`Displacement`].
    #[inline]
    pub(crate) const fn power_to_len(power: u32) -> usize {
        // 2 -> 5
        // 3 -> 9
        // 4 -> 17
        2_usize.pow(power) + 1
    }

    #[inline]
    pub(crate) const fn len_to_power(size: usize) -> u32 {
        // almost certainly 2,3, or 4
        match size {
            5 => 2,
            9 => 3,
            17 => 4,
            // ilog2() yipee!
            x => (x - 1).ilog2(), // what Wolfram Alpha tells me
        }
    }

    // TODO: rename to reference? default?
    // TODO:FIXME:TODO:DOCS: FIX ORDER / DOCS, BOTTOM RIGHT, to top right, then left
    /// The "default", untranslated position of points on a [`Displacement`]
    /// Lerps between `top_line` and `bottom_line`. Pushes to the [`Vec2d`] in
    /// the order left to right (width), top to bottom (height).
    pub fn ideal_points(&self) -> Vec2d<Vector3<f32>> {
        Self::lerp4(self.corners(), self.width)
    }

    // TODO:DOCS:
    // TODO: move to vec2d? as from_4_points?
    fn lerp4(points: [&Vector3<f32>; 4], num_points: usize) -> Vec2d<Vector3<f32>> {
        let [bl, tl, tr, br] = points;
        let mut points = Vec2d::with_capacity(num_points * num_points, Vec2d::strides(num_points));
        // TODO: unnecessary repeated calc of top and bottom
        for y in 0..num_points {
            for x in 0..num_points {
                // lerp horizontally
                let max = (num_points - 1) as f32;
                let t_x = x as f32 / max;
                // let t_x = 1.0 - t_x; // HACK:
                let top = tl.lerp(tr, t_x);
                let bottom = bl.lerp(br, t_x);
                let (top, bottom) = (bottom, top); // HACK:

                // lerp vetically
                let t_y = y as f32 / max;
                // dbg2!((&top, &bottom));x
                let point = top.lerp(&bottom, t_y);
                // eprintln!("{}\t{}\t{}", t_x, t_y, point);
                // eprintln!("Vector3::new({:.1}, {:.1}, {:.1}),", point.x, point.y, point.z);

                points.inner.push(point);
            }
        }
        points
    }

    /// 0,0,0
    #[inline]
    fn offsets(&self) -> Vec2d<Vector3<f32>> {
        Vec2d::from_parts(vec![Vector3::origin(); self.width * self.width], [self.width, 1])
    }

    /// `self.plane.normal()`
    #[inline]
    fn offset_normals(&self) -> Vec2d<Vector3<f32>> {
        Vec2d::from_parts(vec![self.plane.normal(); self.width * self.width], [self.width, 1])
    }

    // TODO: actually calculate walkablity
    // <https://developer.valvesoftware.com/wiki/.vmf#triangle_tags>
    /// 0, walkable
    fn triangle_tags(&self) -> Vec2d<i32> {
        // 5 -> 4x8
        // 9 -> 8x16
        // 17 -> 16x32
        let height = self.width - 1;
        let width = height * 2; // x2 cuz 2 trianlges in 1 square

        let total = width * height;
        let value = TriangleTag::Unwalkable as i32;

        Vec2d::from_parts(vec![value; total], [width, 1])
    }

    /// All always allowed. As far as I can tell, this is only a small optimization
    /// for two displacments of differing power to save like 5 triangles.
    #[inline]
    const fn allowed_verts() -> [i32; 10] {
        [-1; 10]
    }

    pub fn into_disp_info(self) -> DispInfo {
        DispInfo {
            // annoying move errors solved by order O_o
            offsets: self.offsets(),
            offset_normals: self.offset_normals(),
            triangle_tags: self.triangle_tags(),
            allowed_verts: Self::allowed_verts(),

            power: Displacement::len_to_power(self.width),
            start_position: self.plane.bottom_left,
            flags: DispInfo::NO_FLAGS,
            elevation: 0.0,
            is_subdiv: false, // pretty sure
            normals: self.normals,
            distances: self.distances,
            alphas: self.alphas,
        }
    }

    pub fn from_disp_info(disp_info: DispInfo, plane: Plane) -> Self {
        let len = Self::power_to_len(disp_info.power);
        let bottom_right = plane.bottom_right();
        let normals = disp_info.normals;
        let distances = disp_info.distances;
        let alphas = disp_info.alphas;

        Self { width: len, plane, bottom_right, normals, distances, alphas }
    }

    // fn project_unit_cube_to_sphere(&mut self) {
    //     let mut points = self.ideal_points();

    //     for point in points.inner.iter_mut() {

    //     }
    // }
}

/// `a` when `t` is 0. `b` when `t` is 1. extrapolates not 0..=1 (I THINK).
/// See <https://en.wikipedia.org/wiki/Linear_interpolation>
pub(crate) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    // arbitrary, but 0.01 was too little for sphere, 0.001 also works
    // randomly decided to do smaller
    const EPSILON: f32 = 1e-4;
    // preemptive (in)sanity checks to make sure its definitely exactly a or b
    if t < EPSILON {
        // about 0
        return a;
    }
    if (t - 1.0).abs() < EPSILON {
        // about 1
        return b;
    }

    (1.0 - t) * a + t * b
}

/// Project a point on a unit cube (-1 to 1) a point on a unit sphere.
/// 
/// See also: <http://mathproofs.blogspot.com/2005/07/mapping-cube-to-sphere.html>
pub(crate) fn project_cube_to_sphere(point: &Vector3<f32>) -> Vector3<f32> {
    let Vector3 { x, y, z } = point;
    let x_2 = x * x;
    let y_2 = y * y;
    let z_2 = z * z;
    
    // x * sqrt(1 - y^2/2 - z^2/2 + y^2*z^2/3)
    // x * sqrt(1 - (y^2 - z^2)/2 + y^2*z^2/3)
    // (x * sqrt(6 * (y^2*(2*z^2-3)-3*(z^2-2)))/6 // from TI-92+ lol

    Vector3 {
        x: x * f32::sqrt(1.0 - (y_2 / 2.0) - (z_2 / 2.0) + (y_2 * z_2) / 3.0),
        y: y * f32::sqrt(1.0 - (z_2 / 2.0) - (x_2 / 2.0) + (z_2 * x_2) / 3.0),
        z: z * f32::sqrt(1.0 - (x_2 / 2.0) - (y_2 / 2.0) + (x_2 * y_2) / 3.0),
    }
}

impl Vector3<f32> {
    /// `self` when `t` is 0. `other` when `t` is 1. extrapolates not 0..=1 (I THINK).
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: lerp(self.x, other.x, t),
            y: lerp(self.y, other.y, t),
            z: lerp(self.z, other.z, t),
        }
    }

    /// Returns a unit vector that points from `self` to `other`,
    /// and a distance along that vector where `other` is.
    pub fn dir_and_dist(&self, other: &Self) -> (Vector3<f32>, f32) {
        // dbg!(&self, &other);
        let mut dir = other.clone() - self;
        // dir.x = dir.x.copysign(self.x);
        // dir.y = dir.y.copysign(self.y);
        // dir.z = dir.z.copysign(self.z);
        // TODO:FIXME: massive normal/dir problem, refernece sphere, stretch 3d, also stretch first normal
        // dir.z = -dir.z;
        let dist = dir.magnitude();
        // let dist = self.dist(other);
        // let mut dir = other.clone();
        dir.normalize_mut();
        (dir, dist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideal() {
        // beautiful
        let truth = Vec2d {
            strides: [5, 1],
            inner: vec![
                Vector3::new(0.0, 0.0, 16.0),
                Vector3::new(4.0, 0.0, 16.0),
                Vector3::new(8.0, 0.0, 16.0),
                Vector3::new(12.0, 0.0, 16.0),
                Vector3::new(16.0, 0.0, 16.0),
                Vector3::new(0.0, 0.0, 12.0),
                Vector3::new(4.0, 0.0, 12.0),
                Vector3::new(8.0, 0.0, 12.0),
                Vector3::new(12.0, 0.0, 12.0),
                Vector3::new(16.0, 0.0, 12.0),
                Vector3::new(0.0, 0.0, 8.0),
                Vector3::new(4.0, 0.0, 8.0),
                Vector3::new(8.0, 0.0, 8.0),
                Vector3::new(12.0, 0.0, 8.0),
                Vector3::new(16.0, 0.0, 8.0),
                Vector3::new(0.0, 0.0, 4.0),
                Vector3::new(4.0, 0.0, 4.0),
                Vector3::new(8.0, 0.0, 4.0),
                Vector3::new(12.0, 0.0, 4.0),
                Vector3::new(16.0, 0.0, 4.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(4.0, 0.0, 0.0),
                Vector3::new(8.0, 0.0, 0.0),
                Vector3::new(12.0, 0.0, 0.0),
                Vector3::new(16.0, 0.0, 0.0),
            ],
        };

        let input = Displacement {
            width: 2,
            plane: Plane::new(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 16.0),
                Vector3::new(16.0, 0.0, 16.0),
            ),
            bottom_right: Vector3::new(16.0, 0.0, 0.0),
            normals: Vec2d::new([0, 1]),
            distances: Vec2d::new([0, 1]),
            alphas: Vec2d::new([0, 1]),
        };

        let output = input.ideal_points();

        assert_eq!(truth, output);
    }

    #[test]
    fn normal_dist_from() {
        let input1 = Vector3::new(1.0, 1.0, 1.0);
        let input2 = Vector3::new(2.0, 3.0, 4.0);
        let (dir, dist) = input1.dir_and_dist(&input2);

        let delta = dir * dist;
        assert_eq!(Vector3::new(1.0, 2.0, 3.0), delta);

        let output2 = input1 + &delta;
        assert_eq!(input2, output2);
    }
}
