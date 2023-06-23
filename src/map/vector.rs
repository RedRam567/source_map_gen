//! A point, direction, or translation in 3d space.

use derive_ops::*;
use std::{fmt::Display, ops::Neg};

/// One of 3 axes, X, Y, Z.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Axis3 {
    X,
    Y,
    Z,
}

// TODO:DOCS: TODO:LOC:
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct Angles {
    /// +down/-up, degrees.
    pub pitch: f64,
    /// +left/-right, degrees.
    pub yaw: f64,
    /// +clockwise/-counterclockwise, degrees.
    pub roll: f64,
}

impl Display for Angles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.pitch, self.yaw, self.roll)
    }
}

/// A point, direction, or translation in 3d space.
#[derive(AddRef, SubRef, MulRef, DivRef)]
#[derive(AddAssignRef, SubAssignRef, MulAssignRef, DivAssignRef)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl Vector3<f32> {
    pub const fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn new_with_round(mut x: f32, mut y: f32, mut z: f32, allow_frac: bool) -> Self {
        if !allow_frac {
            x = x.round();
            y = y.round();
            z = z.round();
        }
        Self::new(x, y, z)
    }

    /// Absolute value of each of the components.
    pub fn abs(self) -> Self {
        Self { x: self.x.abs(), y: self.y.abs(), z: self.z.abs() }
    }

    pub fn round(self) -> Self {
        let mut x = self.x.round();
        let mut y = self.y.round();
        let mut z = self.z.round();
        if x == -0.0 {
            x = 0.0;
        }
        if y == -0.0 {
            y = 0.0;
        }
        if z == -0.0 {
            z = 0.0;
        }
        Self { x, y, z }
    }

    /// The length of the vector.
    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// The dot product.
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// The distance between two 3d points.
    pub fn dist(&self, other: &Self) -> f32 {
        // sqrt(dx^2 + dy^2 + dz^2)
        let x = self.x - other.x;
        let y = self.y - other.y;
        let z = self.z - other.z;
        f32::sqrt(x * x + y * y + z * z)
    }

    // x=1,y=2,z=3, i=x axis, j=y axis, k=z axis (right handed z up)
    /// Calculate the cross product, basically the normal.
    /// <https://en.wikipedia.org/wiki/Cross_product#Computing>
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn normalize(&self) -> Self {
        self.clone() / self.magnitude()
    }

    pub fn normalize_mut(&mut self) -> &mut Self {
        let magnitude = self.clone().magnitude();
        *self /= magnitude;
        self
    }

    /// Gives the greatest axis, tie breaks to Z, then Y, then X.
    ///
    /// If only X is `NaN`, tests Y and Z.
    /// If else if any is `NaN`, returns Z.
    #[inline]
    pub fn greatest_axis(&self) -> Axis3 {
        if self.x > self.y {
            if self.x > self.z {
                Axis3::X
            } else {
                Axis3::Z
            }
        } else if self.y > self.z {
            Axis3::Y
        } else {
            Axis3::Z
        }
    }

    // Returns true if the vector lies on an axis or is the origin.
    #[inline]
    pub fn is_axis_aligned(&self) -> bool {
        // all zeros expect for 1
        let mut num_zeros = 0u8;
        num_zeros += u8::from(self.x == 0.0);
        num_zeros += u8::from(self.y == 0.0);
        num_zeros += u8::from(self.z == 0.0);
        num_zeros >= 2
    }
}

impl<T> Vector3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy> Vector3<T> {
    /// Constant clone. `Vector3::clone()` isn't const for some reason.
    pub const fn const_clone(&self) -> Self {
        Self { ..*self }
    }

    // TODO: From and as
    pub const fn into_vector2(self) -> Vector2<T> {
        Vector2 { x: self.x, y: self.y }
    }

    pub const fn into_vector2_and_z(self) -> (Vector2<T>, T) {
        (Vector2 { x: self.x, y: self.y }, self.z)
    }
}

impl<T> From<[T; 3]> for Vector3<T> {
    fn from(value: [T; 3]) -> Self {
        let [x, y, z] = value;
        Self { x, y, z }
    }
}

impl<T: Display> Display for Vector3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}

impl<T: Neg<Output = T>> Neg for Vector3<T> {
    type Output = Vector3<T>;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y, z: -self.z }
    }
}

/// A point, direction, or translation in 2d space.
#[derive(AddRef, SubRef, MulRef, DivRef)]
#[derive(AddAssignRef, SubAssignRef, MulAssignRef, DivAssignRef)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Vector2<f32> {
    pub const fn origin() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn new_with_round(mut x: f32, mut y: f32, allow_frac: bool) -> Self {
        if !allow_frac {
            x = x.round();
            y = y.round();
        }
        Self::new(x, y)
    }

    /// The distance between two 2d points.
    pub fn dist(&self, other: &Self) -> f32 {
        // sqrt(dx^2 + dy^2)
        let delta = self.clone() - other;
        delta.x.hypot(delta.y)
    }
}

impl<T: Copy> Vector2<T> {
    /// Constant clone. `Vector2::clone()` isn't const for some reason.
    pub const fn const_clone(&self) -> Self {
        Self { ..*self }
    }

    pub const fn with_z(&self, z: T) -> Vector3<T> {
        Vector3 { x: self.x, y: self.y, z }
    }
}

impl<T: Neg<Output = T>> Neg for Vector2<T> {
    type Output = Vector2<T>;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greatest_axis() {
        let nan = std::f32::NAN;
        assert_eq!(Axis3::Z, Vector3::new(0.0, 0.0, 0.0).greatest_axis());
        assert_eq!(Axis3::X, Vector3::new(1.0, 0.0, 0.0).greatest_axis());
        assert_eq!(Axis3::Y, Vector3::new(0.0, 1.0, 0.0).greatest_axis());
        assert_eq!(Axis3::Y, Vector3::new(1.0, 1.0, 0.0).greatest_axis());
        assert_eq!(Axis3::Z, Vector3::new(0.0, 0.0, 1.0).greatest_axis());
        assert_eq!(Axis3::Z, Vector3::new(1.0, 0.0, 1.0).greatest_axis());
        assert_eq!(Axis3::Z, Vector3::new(0.0, 1.0, 1.0).greatest_axis());
        assert_eq!(Axis3::Z, Vector3::new(1.0, 1.0, 1.0).greatest_axis());

        // I desire nan being the "smallest" float (kinda like max())
        // current behavior:
        assert_eq!(Axis3::Z, Vector3::new(nan, 0.0, 0.0).greatest_axis());

        assert_eq!(Axis3::Y, Vector3::new(nan, 1.0, 0.0).greatest_axis());
        assert_eq!(Axis3::Z, Vector3::new(2.0, nan, 0.0).greatest_axis());
        assert_eq!(Axis3::Z, Vector3::new(nan, nan, 0.0).greatest_axis());
        assert_eq!(Axis3::Z, Vector3::new(2.0, 1.0, nan).greatest_axis());
        assert_eq!(Axis3::Z, Vector3::new(nan, 1.0, nan).greatest_axis());
        assert_eq!(Axis3::Z, Vector3::new(2.0, nan, nan).greatest_axis());
        assert_eq!(Axis3::Z, Vector3::new(nan, nan, nan).greatest_axis());
    }

    #[test]
    fn axis_aligned() {
        assert_eq!(true, Vector3::new(-0.0, 0.0, 0.0).is_axis_aligned());
        assert_eq!(true, Vector3::new(1.0, -0.0, -0.0).is_axis_aligned());
        assert_eq!(true, Vector3::new(0.0, 1.0, 0.0).is_axis_aligned());
        assert_eq!(false, Vector3::new(1.0, 1.0, -0.0).is_axis_aligned());
        assert_eq!(true, Vector3::new(0.0, -0.0, 1.0).is_axis_aligned());
        assert_eq!(false, Vector3::new(1.0, -0.0, 1.0).is_axis_aligned());
        assert_eq!(false, Vector3::new(-0.0, 1.0, 1.0).is_axis_aligned());
        assert_eq!(false, Vector3::new(1.0, 1.0, 1.0).is_axis_aligned());
    }
}
