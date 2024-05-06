use crate::{impl_shared, impl_vector, Point2};

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl_shared!(Vec2, f32, 2);
impl_vector!(Vec2, f32, 2);

impl Vec2 {
    pub const X: Vec2 = Vec2::new(1.0, 0.0);
    pub const Y: Vec2 = Vec2::new(0.0, 1.0);

    /// Constructs a new [`Vec2`] with the given `x` and `y` components.
    #[inline(always)]
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Converts this point to the equivalent point.
    #[inline(always)]
    #[must_use]
    pub const fn as_point2(self) -> Point2 {
        Point2::new(self.x, self.y)
    }

    /// Returns a [`Vec2`] with the function `f` applied to each component in order.
    #[inline(always)]
    #[must_use]
    pub fn map<F>(self, mut f: F) -> Vec2
    where
        F: FnMut(f32) -> f32,
    {
        Vec2 {
            x: f(self.x),
            y: f(self.y),
        }
    }

    /// Returns a new [`Vec2`] with the function `f` applied to each pair of
    /// components from `self` and `rhs` in order.
    #[inline(always)]
    #[must_use]
    pub fn map2<F>(self, rhs: Vec2, mut f: F) -> Vec2
    where
        F: FnMut(f32, f32) -> f32,
    {
        Vec2 {
            x: f(self.x, rhs.x),
            y: f(self.y, rhs.y),
        }
    }

    /// Returns the dot product of `a` and `b`.
    #[inline(always)]
    #[must_use]
    pub fn dot(a: Self, b: Self) -> f32 {
        a.x * b.x + a.y * b.y
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul for Vec2 {
    type Output = Vec2;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl std::ops::Div for Vec2 {
    type Output = Vec2;
    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl std::ops::AddAssign for Vec2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::SubAssign for Vec2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::MulAssign for Vec2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl std::ops::DivAssign for Vec2 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}
