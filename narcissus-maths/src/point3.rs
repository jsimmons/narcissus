use crate::{impl_affine, impl_shared, Vec3};

/// Type representing a point in a 3d affine space.
#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl_shared!(Point3, f32, 3);
impl_affine!(Point3, f32, 3);

impl Point3 {
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    /// Creates a new point in 3d space with the given `x`, `y` and `z` coordinates.
    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Returns a new point in 3d space with the function `f` applied to each coordinate in order.
    #[inline(always)]
    pub fn map<F>(self, mut f: F) -> Self
    where
        F: FnMut(f32) -> f32,
    {
        Self {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
        }
    }
}

impl std::ops::Sub for Point3 {
    type Output = Vec3;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Add<Vec3> for Point3 {
    type Output = Point3;
    #[inline]
    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub<Vec3> for Point3 {
    type Output = Point3;
    #[inline]
    fn sub(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::AddAssign<Vec3> for Point3 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::SubAssign<Vec3> for Point3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn distance() {
        // nice triangle
        assert_eq!(
            Point3::distance_sq(Point3::ZERO, Point3::new(5.0, 12.0, 0.0)),
            169.0
        );
        assert_eq!(
            Point3::distance(Point3::ZERO, Point3::new(5.0, 12.0, 0.0)),
            13.0
        );
    }
}
