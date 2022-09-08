use crate::{impl_shared, impl_vector};

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl_shared!(Vec4, f32, 4);
impl_vector!(Vec4, f32, 4);

impl Vec4 {
    pub const X: Vec4 = Vec4::new(1.0, 0.0, 0.0, 0.0);
    pub const Y: Vec4 = Vec4::new(0.0, 1.0, 0.0, 0.0);
    pub const Z: Vec4 = Vec4::new(0.0, 0.0, 1.0, 0.0);
    pub const W: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);

    /// Constructs a new [`Vec4`] with the given `x`, `y`, `z` and `w` components.
    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
        Vec4 { x, y, z, w }
    }

    /// Returns a [`Vec4`] with the function `f` applied to each component in order.
    #[inline(always)]
    pub fn map<F>(self, mut f: F) -> Vec4
    where
        F: FnMut(f32) -> f32,
    {
        Self {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
            w: f(self.w),
        }
    }

    /// Returns a new [`Vec4`] with the function `f` applied to each pair of components from `self` and `rhs` in order.
    #[inline(always)]
    pub fn map2<F>(self, rhs: Self, mut f: F) -> Vec4
    where
        F: FnMut(f32, f32) -> f32,
    {
        Vec4 {
            x: f(self.x, rhs.x),
            y: f(self.y, rhs.y),
            z: f(self.z, rhs.z),
            w: f(self.w, rhs.w),
        }
    }

    /// Returns the dot product of `a` and `b`.
    #[inline]
    pub fn dot(a: Vec4, b: Vec4) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
    }
}

#[cfg(target_feature = "sse2")]
impl From<std::arch::x86_64::__m128> for Vec4 {
    #[inline(always)]
    fn from(values: std::arch::x86_64::__m128) -> Self {
        use std::arch::x86_64::_mm_storeu_ps;
        let mut result = Vec4::ZERO;
        unsafe { _mm_storeu_ps(&mut result.x, values) }
        result
    }
}

#[cfg(target_feature = "sse2")]
impl From<Vec4> for std::arch::x86_64::__m128 {
    #[inline(always)]
    fn from(x: Vec4) -> Self {
        unsafe { std::arch::x86_64::_mm_loadu_ps(&x.x) }
    }
}

//
// #[inline(always)]
// pub(crate) fn as_m128(self) -> std::arch::x86_64::__m128 {
//
// }

// #[cfg(target_feature = "sse2")]
// #[inline(always)]
// pub(crate) fn from_m128(values: std::arch::x86_64::__m128) -> Self {
//     use std::arch::x86_64::_mm_storeu_ps;
//     let mut result = Vec4::ZERO;
//     unsafe { _mm_storeu_ps(&mut result.x, values) }
//     result
// }

impl std::ops::Add for Vec4 {
    type Output = Vec4;

    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe {
            use std::arch::x86_64::_mm_add_ps;
            _mm_add_ps(self.into(), rhs.into()).into()
        }
    }
}

impl std::ops::Sub for Vec4 {
    type Output = Vec4;

    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe { std::arch::x86_64::_mm_sub_ps(self.into(), rhs.into()).into() }
    }
}

impl std::ops::Mul for Vec4 {
    type Output = Vec4;

    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe { std::arch::x86_64::_mm_mul_ps(self.into(), rhs.into()).into() }
    }
}

impl std::ops::Div for Vec4 {
    type Output = Vec4;

    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
            w: self.w / rhs.w,
        }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        unsafe { std::arch::x86_64::_mm_div_ps(self.into(), rhs.into()).into() }
    }
}

impl std::ops::AddAssign for Vec4 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::SubAssign for Vec4 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl std::ops::MulAssign for Vec4 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl std::ops::DivAssign for Vec4 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert_eq!(
            Vec4::new(1.0, 2.0, 3.0, 4.0) + Vec4::new(4.0, 3.0, 2.0, 1.0),
            Vec4::splat(5.0)
        );
        assert_eq!(
            Vec4::new(4.0, 3.0, 2.0, 1.0) - Vec4::new(3.0, 2.0, 1.0, 0.0),
            Vec4::splat(1.0)
        );
        assert_eq!(
            Vec4::new(1.0, 2.0, 3.0, 4.0) * Vec4::new(4.0, 3.0, 2.0, 1.0),
            Vec4::new(4.0, 6.0, 6.0, 4.0)
        );
        assert_eq!(
            Vec4::new(1.0, 2.0, 3.0, 4.0) / Vec4::splat(2.0),
            Vec4::new(0.5, 1.0, 1.5, 2.0)
        );

        assert_eq!(Vec4::new(2.0, 2.0, 2.0, 2.0).length_sq(), 16.0);
        assert_eq!(Vec4::new(2.0, 2.0, 2.0, 2.0).length(), 4.0);

        assert_eq!(
            Vec4::clamp(Vec4::ONE * 5.0, Vec4::ZERO, Vec4::ONE),
            Vec4::ONE
        );
    }
}
