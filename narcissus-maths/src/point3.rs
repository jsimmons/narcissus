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
    /// Constructs a new [`Point3`] with the given `x`, `y`, and `z` coordinates.
    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Point3 {
        Point3 { x, y, z }
    }

    /// Converts this point to the equivalent vector.
    #[inline(always)]
    pub const fn as_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// Returns a new [`Point3`] with the function `f` applied to each coordinate of `self` in order.
    #[inline(always)]
    pub fn map<F>(self, mut f: F) -> Point3
    where
        F: FnMut(f32) -> f32,
    {
        Point3 {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
        }
    }

    /// Returns a new [`Point3`] with the function `f` applied to each pair of components from `self` and `rhs` in order.
    #[inline(always)]
    pub fn map2<F>(self, rhs: Point3, mut f: F) -> Point3
    where
        F: FnMut(f32, f32) -> f32,
    {
        Point3 {
            x: f(self.x, rhs.x),
            y: f(self.y, rhs.y),
            z: f(self.z, rhs.z),
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
