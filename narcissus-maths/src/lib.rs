mod affine2;
mod affine3;
mod mat2;
mod mat3;
mod mat4;
mod next_after_f32;
mod point2;
mod point3;
mod quat;
mod sin_cos_pi;
mod tan_pi;
mod vec2;
mod vec3;
mod vec4;

pub use affine2::Affine2;
pub use affine3::Affine3;
pub use mat2::Mat2;
pub use mat3::Mat3;
pub use mat4::Mat4;
pub use next_after_f32::next_after_f32;
pub use point2::Point2;
pub use point3::Point3;
pub use quat::Quat;
pub use sin_cos_pi::sin_cos_pi_f32;
pub use tan_pi::tan_pi_f32;
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;

/// Unit type for an angle expressed in radians.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
pub struct Rad(f32);

impl Rad {
    #[inline(always)]
    pub const fn new(x: f32) -> Self {
        Self(x)
    }

    #[inline(always)]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

/// Unit type for an angle expressed in degrees.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
pub struct Deg(f32);

impl Deg {
    #[inline(always)]
    pub const fn new(x: f32) -> Self {
        Self(x)
    }

    #[inline(always)]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

/// Unit type for an angle expressed in half-turns.
///
/// A turn represents a 360 degree rotation, a half-turn represents a 180 degree rotation. A
/// half-turn is implicitly scaled by pi.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
pub struct HalfTurn(f32);

impl HalfTurn {
    #[inline(always)]
    pub const fn new(x: f32) -> Self {
        Self(x)
    }

    #[inline(always)]
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

impl From<HalfTurn> for Deg {
    fn from(x: HalfTurn) -> Self {
        Self(x.0 * 180.0)
    }
}

impl From<Deg> for Rad {
    #[inline(always)]
    fn from(x: Deg) -> Self {
        Self(x.0.to_radians())
    }
}

impl From<HalfTurn> for Rad {
    #[inline(always)]
    fn from(x: HalfTurn) -> Self {
        Self(x.0 * std::f32::consts::PI)
    }
}

impl From<Rad> for HalfTurn {
    #[inline(always)]
    fn from(x: Rad) -> Self {
        Self(x.0 / std::f32::consts::PI)
    }
}

impl From<Deg> for HalfTurn {
    #[inline(always)]
    fn from(x: Deg) -> Self {
        Self(x.0 / 180.0)
    }
}

/// Returns the minimum of `x` and `y`.
///
/// This function returns a platform dependent value if either of its inputs are `NaN`.
///
/// Platform Specific Behavior
/// ---
/// On `x86` If either input is `NaN`, returns the value of `y`. Other platforms follow IEEE754-2008 semantics, where if
/// either input is `NaN` the other input is returned. `NaN` propagates when both inputs are `NaN`.
#[inline(always)]
pub fn min(x: f32, y: f32) -> f32 {
    #[cfg(target_arch = "x86_64")]
    if x < y {
        x
    } else {
        y
    }
    #[cfg(not(target_arch = "x86_64"))]
    x.min(y)
}

/// Returns the maximum of `x` and `y`.
///
/// This function returns a platform dependent value if either of its inputs are `NaN`.
///
/// # Platform Specific Behavior
/// On `x86` If either input is `NaN`, returns the value of `y`. Other platforms follow IEEE754-2008 semantics, where if
/// either input is `NaN` the other input is returned. `NaN` propagates when both inputs are `NaN`.
#[inline(always)]
pub fn max(x: f32, y: f32) -> f32 {
    #[cfg(target_arch = "x86_64")]
    if x > y {
        x
    } else {
        y
    }
    #[cfg(not(target_arch = "x86_64"))]
    x.max(y)
}

/// Returns the value `x` clamped between `lo` and `hi`.
///
/// This function returns an unspecified, platform dependent value if any of its inputs are `NaN`.
///
/// # Panics
///
/// Panics if `lo` is greater than `hi`.
#[inline(always)]
pub fn clamp(x: f32, lo: f32, hi: f32) -> f32 {
    debug_assert!(lo <= hi);
    max(min(x, hi), lo)
}

/// Returns the nearest integer to a given `f32`.
///
/// Respects IEEE-754 tiesToEven
#[inline(always)]
fn round_ties_to_even(x: f32) -> f32 {
    #[cfg(target_feature = "sse4.1")]
    unsafe {
        use std::arch::x86_64::{
            _mm_load_ss, _mm_round_ss, _MM_FROUND_NO_EXC, _MM_FROUND_TO_NEAREST_INT,
        };
        const ROUNDING: i32 = (_MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC) as i32;
        let x = _mm_load_ss(&x);
        let x = _mm_round_ss::<ROUNDING>(x, x);
        std::arch::x86_64::_mm_cvtss_f32(x)
    }
}

#[macro_export]
macro_rules! impl_shared {
    ($name:ty, $t:ty, $n:expr) => {
        impl $name {
            #[doc = concat!("[`", stringify!($name), "`] with all elements initialized to `0.0`.")]
            pub const ZERO: $name = Self::splat(0.0);
            #[doc = concat!("[`", stringify!($name), "`] with all elements initialized to `1.0`.")]
            pub const ONE: $name = Self::splat(1.0);
            #[allow(clippy::zero_divided_by_zero)]
            #[doc = concat!("[`", stringify!($name), "`] with all elements initialized to `NaN`.")]
            pub const NAN: $name = Self::splat(0.0 / 0.0);

            #[doc = concat!("Constructs a new [`", stringify!($name), "`] where each element is initialized with the given `value`.")]
            #[inline(always)]
            pub const fn splat(value: $t) -> $name {
                // we have to transmute here because we can't make `into()` const.
                // Safety: $name is repr(C) struct with $n elements of type $t, so the transmute is always valid.
                unsafe { std::mem::transmute([value; $n]) }
            }

            #[doc = concat!("Returns a [`", stringify!($name), "`] where the `i`th element is initialized with the minimum of the corresponding elements `a[i]` and `b[i]`.\n\nThis function returns a platform dependent value if either input is `NaN`. See [`crate::min`] for exact details.")]
            #[inline]
            pub fn min(a: $name, b: $name) -> $name {
                a.map2(b, |a, b| $crate::min(a, b))
            }

            #[doc = concat!("Returns a [`", stringify!($name), "`] where the `i`th element is initialized with the maximum of the corresponding elements `a[i]` and `b[i]`.\n\nThis function returns a platform dependent value if either input is `NaN`. See [`crate::max`] for exact details.")]
            #[inline]
            pub fn max(a: $name, b: $name) -> $name {
                a.map2(b, |a, b| $crate::max(a, b))
            }

            #[doc = concat!("Returns a [`", stringify!($name), "`] where the `i`th element `x[i]` is clamped between the corresponding elements `lo[i]` and `hi[i]`.\n\n# Panics\n\nPanics if any element of `lo` is greater than its corresponding element in `hi`.")]
            #[inline]
            pub fn clamp(x: $name, lo: $name, hi: $name) -> $name {
                Self::max(Self::min(x, hi), lo)
            }

            #[doc = concat!("Returns a [`", stringify!($name), "`] where each element is the absolute value of the corresponding element in `self`.")]
            #[inline]
            pub fn abs(self) -> $name {
                self.map(|x| x.abs())
            }

            #[doc = concat!("Returns a [`", stringify!($name), "`] where each element is the smallest integer value greater than or equal to the corresponding element in `self`.")]
            #[inline(always)]
            pub fn ceil(self) -> $name {
                self.map(|x| x.ceil())
            }

            #[doc = concat!("Returns a [`", stringify!($name), "`] where each element is the largest integer value less than or equal to the corresponding element in `self`.")]
            #[inline(always)]
            pub fn floor(self) -> $name {
                self.map(|x| x.floor())
            }

            #[doc = concat!("Returns a [`", stringify!($name), "`] where each element is the nearest integer value to the corresponding element in `self`. Rounds half-way cases away from `0.0`.")]
            #[inline(always)]
            pub fn round(self) -> $name {
                self.map(|x| x.round())
            }
        }

        impl From<[$t; $n]> for $name {
            #[inline(always)]
            fn from(x: [$t; $n]) -> $name {
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
            /// Calculates the euclidean distance between the two points `a` and `b`.
            #[inline]
            pub fn distance(a: $name, b: $name) -> $t {
                (b - a).length()
            }

            /// Calculates the squared euclidean distance between the two points `a` and `b`.
            /// Avoids an expensive `sqrt` operation.
            #[inline]
            pub fn distance_sq(a: $name, b: $name) -> $t {
                (b - a).length_sq()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_vector {
    ($name:ty, $t:ty, $n:expr) => {
        impl $name {
            /// Calculates the length of the vector `self`.
            #[inline]
            pub fn length(self) -> $t {
                self.length_sq().sqrt()
            }

            /// Calculate the squared length of the vector `self`.
            /// Avoids an expensive `sqrt` operation.
            #[inline]
            pub fn length_sq(self) -> $t {
                Self::dot(self, self)
            }

            /// Returns a vector with the same direction as `self` but with unit (1.0) length.
            #[must_use]
            #[inline]
            pub fn normalized(self) -> $name {
                self / self.length()
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
