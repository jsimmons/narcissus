mod affine2;
mod affine3;
mod mat2;
mod mat3;
mod mat4;
mod point2;
mod point3;
mod quat;
mod vec2;
mod vec3;
mod vec4;

pub use affine2::Affine2;
pub use affine3::Affine3;
pub use mat2::Mat2;
pub use mat3::Mat3;
pub use mat4::Mat4;
pub use point2::Point2;
pub use point3::Point3;
pub use quat::Quat;
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;

/// Unit type for an angle expressed in radians.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
pub struct Rad(f32);

impl Rad {
    pub const fn new(x: f32) -> Self {
        Self(x)
    }

    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

/// Unit type for an angle expressed in degrees.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
pub struct Deg(f32);

impl Deg {
    pub const fn new(x: f32) -> Self {
        Self(x)
    }

    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

impl From<Rad> for Deg {
    #[inline(always)]
    fn from(x: Rad) -> Self {
        Self(x.0.to_degrees())
    }
}

impl From<Deg> for Rad {
    #[inline(always)]
    fn from(x: Deg) -> Self {
        Self(x.0.to_radians())
    }
}

#[inline(always)]
pub fn min(x: f32, y: f32) -> f32 {
    if x < y {
        x
    } else {
        y
    }
}

#[inline(always)]
pub fn max(x: f32, y: f32) -> f32 {
    if x > y {
        x
    } else {
        y
    }
}

#[inline(always)]
pub fn clamp(x: f32, lo: f32, hi: f32) -> f32 {
    debug_assert!(lo <= hi);
    max(min(x, hi), lo)
}

#[macro_export]
macro_rules! impl_shared {
    ($name:ty, $t:ty, $n:expr) => {
        impl $name {
            pub const ZERO: Self = Self::splat(0.0);
            pub const ONE: Self = Self::splat(1.0);
            pub const NAN: Self = Self::splat(0.0 / 0.0);

            #[inline(always)]
            pub const fn splat(value: $t) -> Self {
                // we have to transmute here because we can't make `into()` const.
                unsafe { std::mem::transmute([value; $n]) }
            }

            #[inline]
            pub fn min(a: Self, b: Self) -> Self {
                a.map2(b, |a, b| crate::min(a, b))
            }

            #[inline]
            pub fn max(a: Self, b: Self) -> Self {
                a.map2(b, |a, b| crate::max(a, b))
            }

            #[inline]
            pub fn clamp(x: Self, lo: Self, hi: Self) -> Self {
                Self::max(Self::min(x, hi), lo)
            }

            #[inline(always)]
            pub fn ceil(self) -> Self {
                self.map(|x| x.ceil())
            }

            #[inline(always)]
            pub fn floor(self) -> Self {
                self.map(|x| x.floor())
            }

            #[inline(always)]
            pub fn round(self) -> Self {
                self.map(|x| x.round())
            }
        }

        impl From<[$t; $n]> for $name {
            #[inline(always)]
            fn from(x: [$t; $n]) -> Self {
                unsafe { std::mem::transmute(x) }
            }
        }

        impl From<$name> for [$t; $n] {
            #[inline(always)]
            fn from(x: $name) -> [$t; $n] {
                unsafe { std::mem::transmute(x) }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_affine {
    ($name:ty, $t:ty, $n:expr) => {
        impl $name {
            #[inline]
            pub fn distance(a: Self, b: Self) -> $t {
                (b - a).length()
            }

            #[inline]
            pub fn distance_sq(a: Self, b: Self) -> $t {
                (b - a).length_sq()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_vector {
    ($name:ty, $t:ty, $n:expr) => {
        impl $name {
            #[inline]
            pub fn length(self) -> $t {
                self.length_sq().sqrt()
            }

            #[inline]
            pub fn length_sq(self) -> $t {
                Self::dot(self, self)
            }
        }

        impl std::ops::Neg for $name {
            type Output = $name;
            #[inline(always)]
            fn neg(self) -> Self::Output {
                self.map(|x| -x)
            }
        }

        impl std::ops::Add<$t> for $name {
            type Output = $name;
            #[inline(always)]
            fn add(self, rhs: $t) -> Self::Output {
                self.map(|x| x + rhs)
            }
        }

        impl std::ops::Sub<$t> for $name {
            type Output = $name;
            #[inline(always)]
            fn sub(self, rhs: $t) -> Self::Output {
                self.map(|x| x - rhs)
            }
        }

        impl std::ops::Mul<$t> for $name {
            type Output = $name;
            #[inline(always)]
            fn mul(self, rhs: $t) -> Self::Output {
                self.map(|x| x * rhs)
            }
        }

        impl std::ops::Mul<$name> for $t {
            type Output = $name;
            #[inline(always)]
            fn mul(self, rhs: $name) -> Self::Output {
                rhs.map(|x| self * x)
            }
        }

        impl std::ops::Div<$t> for $name {
            type Output = $name;
            #[inline(always)]
            fn div(self, rhs: $t) -> Self::Output {
                self.map(|x| x / rhs)
            }
        }

        impl std::ops::AddAssign<$t> for $name {
            #[inline(always)]
            fn add_assign(&mut self, rhs: $t) {
                *self = *self + rhs;
            }
        }

        impl std::ops::SubAssign<$t> for $name {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: $t) {
                *self = *self - rhs;
            }
        }

        impl std::ops::MulAssign<$t> for $name {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: $t) {
                *self = *self * rhs;
            }
        }

        impl std::ops::DivAssign<$t> for $name {
            #[inline(always)]
            fn div_assign(&mut self, rhs: $t) {
                *self = *self / rhs;
            }
        }
    };
}
