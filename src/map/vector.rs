use std::{fmt::Display, ops::Neg};

/// A point or translation in 3d space.
#[derive(
    derive_ops::AddAssignRef,
    derive_ops::SubAssignRef,
    derive_ops::MulAssignRef,
    derive_ops::DivAssignRef,
    derive_ops::AddRef,
    derive_ops::SubRef,
    derive_ops::MulRef,
    derive_ops::DivRef,
)]
//
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl Vector3<f32> {
    pub fn abs(&self) -> Self {
        Self { x: self.x.abs(), y: self.y.abs(), z: self.z.abs() }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    // length of
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    // x=1,y=2,z=3, i=x axis, j=y axis, k=z axis (right handed z up)
    // https://en.wikipedia.org/wiki/Cross_product#Matrix_notation
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: -(self.x * other.z - self.z * other.x),
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Axis3 {
    X,
    Y,
    Z,
}

impl<T> Vector3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy> Vector3<T> {
    /// Constant clone. `Point::clone()` isn't const for some reason.
    pub const fn const_clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<T: Display> Display for Vector3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.x, self.y, self.z)
    }
}

impl<T: Neg<Output = T>> Neg for Vector3<T> {
    type Output = Vector3<T>;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y, z: -self.z }
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
}
