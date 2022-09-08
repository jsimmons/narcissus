use crate::{impl_affine, impl_shared, Vec2};

/// Type representing a point in a 2d affine space.
#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl_shared!(Point2, f32, 2);
impl_affine!(Point2, f32, 2);

impl Point2 {
    /// Constructs a new [`Point2`] with the given `x` and `y` coordinates.
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns a new [`Point2`] with the function `f` applied to each coordinate of `self` in order.
    #[inline(always)]
    pub fn map<F>(self, mut f: F) -> Self
    where
        F: FnMut(f32) -> f32,
    {
        Self {
            x: f(self.x),
            y: f(self.y),
        }
    }

    /// Returns a new [`Point2`] with the function `f` applied to each pair of components from `self` and `rhs` in order.
    #[inline(always)]
    pub fn map2<F>(self, rhs: Self, mut f: F) -> Self
    where
        F: FnMut(f32, f32) -> f32,
    {
        Self {
            x: f(self.x, rhs.x),
            y: f(self.y, rhs.y),
        }
    }
}

impl std::ops::Sub for Point2 {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add<Vec2> for Point2 {
    type Output = Point2;
    #[inline]
    fn add(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Vec2> for Point2 {
    type Output = Point2;
    #[inline]
    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::AddAssign<Vec2> for Point2 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::SubAssign<Vec2> for Point2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
