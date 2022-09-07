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
                (a - b).length()
            }

            #[inline]
            pub fn distance_sq(a: Self, b: Self) -> $t {
                (a - b).length_sq()
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
    };
}
