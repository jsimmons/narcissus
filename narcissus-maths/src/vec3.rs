use crate::{impl_shared, impl_vector, Point3};

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl_shared!(Vec3, f32, 3);
impl_vector!(Vec3, f32, 3);

impl Vec3 {
    pub const X: Vec3 = Vec3::new(1.0, 0.0, 0.0);
    pub const Y: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    pub const Z: Vec3 = Vec3::new(0.0, 0.0, 1.0);

    /// Constructs a new [`Vec3`] with the given `x`, `y` and `z` components.
    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    /// Converts this point to the equivalent point.
    #[inline(always)]
    pub const fn as_point3(self) -> Point3 {
        Point3::new(self.x, self.y, self.z)
    }

    /// Returns a [`Vec3`] with the function `f` applied to each component in order.
    #[inline(always)]
    pub fn map<F>(self, mut f: F) -> Vec3
    where
        F: FnMut(f32) -> f32,
    {
        Vec3 {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
        }
    }

    /// Returns a new [`Vec3`] with the function `f` applied to each pair of components from `self` and `rhs` in order.
    #[inline(always)]
    pub fn map2<F>(self, rhs: Self, mut f: F) -> Vec3
    where
        F: FnMut(f32, f32) -> f32,
    {
        Vec3 {
            x: f(self.x, rhs.x),
            y: f(self.y, rhs.y),
            z: f(self.z, rhs.z),
        }
    }

    /// Returns the dot product of `a` and `b`.
    #[inline]
    #[must_use]
    pub fn dot(a: Vec3, b: Vec3) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    /// Returns the cross product of `a` and `b`.
    #[inline]
    #[must_use]
    pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
        [
            a.y * b.z - a.z * b.y,
            a.z * b.x - a.x * b.z,
            a.x * b.y - a.y * b.x,
        ]
        .into()
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for Vec3 {
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

impl std::ops::Mul for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl std::ops::Div for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl std::ops::AddAssign for Vec3 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::SubAssign for Vec3 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl std::ops::MulAssign for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl std::ops::DivAssign for Vec3 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_product() {
        assert_eq!(
            Vec3::cross(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)),
            Vec3::new(-3.0, 6.0, -3.0)
        );
        assert_eq!(
            Vec3::cross(Vec3::new(2.0, 1.0, 2.0), Vec3::new(3.0, 4.0, 3.0)),
            Vec3::new(-5.0, 0.0, 5.0)
        );
    }
}
