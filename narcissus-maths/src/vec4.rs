#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Self = Self::splat(0.0);

    pub const X: Self = Self::new(1.0, 0.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0, 0.0);
    pub const W: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline(always)]
    pub const fn splat(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }

    #[inline(always)]
    pub fn as_array(self) -> [f32; 4] {
        unsafe { std::mem::transmute(self) }
    }

    #[inline(always)]
    pub fn from_array(values: [f32; 4]) -> Self {
        unsafe { std::mem::transmute(values) }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    pub(crate) fn as_m128(self) -> std::arch::x86_64::__m128 {
        unsafe { std::arch::x86_64::_mm_loadu_ps(&self.x) }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    pub(crate) fn from_m128(values: std::arch::x86_64::__m128) -> Self {
        use std::arch::x86_64::_mm_storeu_ps;
        let mut result = Vec4::ZERO;
        unsafe { _mm_storeu_ps(&mut result.x, values) }
        result
    }

    #[inline]
    pub fn distance(a: Self, b: Self) -> f32 {
        (a - b).length()
    }

    #[inline]
    pub fn distance_sq(a: Self, b: Self) -> f32 {
        (a - b).length_sq()
    }

    #[inline]
    pub fn dot(a: Self, b: Self) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        Self::dot(self, self)
    }

    #[inline]
    pub fn ceil(self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
            z: self.z.ceil(),
            w: self.w.ceil(),
        }
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor(),
            w: self.w.floor(),
        }
    }

    #[inline]
    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
            z: self.z.round(),
            w: self.w.round(),
        }
    }
}

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
            Vec4::from_m128(_mm_add_ps(self.as_m128(), rhs.as_m128()))
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
        unsafe {
            use std::arch::x86_64::_mm_sub_ps;
            Vec4::from_m128(_mm_sub_ps(self.as_m128(), rhs.as_m128()))
        }
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
        unsafe {
            use std::arch::x86_64::_mm_mul_ps;
            Vec4::from_m128(_mm_mul_ps(self.as_m128(), rhs.as_m128()))
        }
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
        unsafe {
            use std::arch::x86_64::_mm_div_ps;
            Vec4::from_m128(_mm_div_ps(self.as_m128(), rhs.as_m128()))
        }
    }
}

impl std::ops::AddAssign for Vec4 {
    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        use std::arch::x86_64::_mm_add_ps;
        unsafe {
            *self = Vec4::from_m128(_mm_add_ps(self.as_m128(), rhs.as_m128()));
        }
    }
}

impl std::ops::SubAssign for Vec4 {
    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        unsafe {
            *self = Vec4::from_m128(std::arch::x86_64::_mm_sub_ps(self.as_m128(), rhs.as_m128()));
        }
    }
}

impl std::ops::MulAssign for Vec4 {
    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
        self.w *= rhs.w;
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        unsafe {
            *self = Vec4::from_m128(std::arch::x86_64::_mm_mul_ps(self.as_m128(), rhs.as_m128()));
        }
    }
}

impl std::ops::DivAssign for Vec4 {
    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
        self.w /= rhs.w;
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        unsafe {
            *self = Vec4::from_m128(std::arch::x86_64::_mm_div_ps(self.as_m128(), rhs.as_m128()));
        }
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

        assert_eq!(Vec4::distance_sq(Vec4::splat(-1.0), Vec4::splat(1.0)), 16.0);
        assert_eq!(Vec4::distance(Vec4::splat(-1.0), Vec4::splat(1.0)), 4.0);
    }
}
