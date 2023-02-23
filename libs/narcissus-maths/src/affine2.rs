use crate::{Mat2, Point2, Vec2};

/// Matrix and translation vector which together represent a 2d affine transformation.
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Affine2 {
    pub matrix: Mat2,
    pub translate: Vec2,
}

impl Affine2 {
    pub const ZERO: Affine2 = Affine2 {
        matrix: Mat2::ZERO,
        translate: Vec2::ZERO,
    };

    pub const IDENTITY: Affine2 = Affine2 {
        matrix: Mat2::IDENTITY,
        translate: Vec2::ZERO,
    };

    pub fn mul_affine2(&self, rhs: Affine2) -> Affine2 {
        Self {
            matrix: self.matrix * rhs.matrix,
            translate: self.translate + rhs.translate,
        }
    }

    pub fn transform_vec2(&self, vec: Vec2) -> Vec2 {
        self.matrix * vec
    }

    pub fn transform_point2(&self, point: Point2) -> Point2 {
        self.matrix * point + self.translate
    }
}

impl std::ops::Mul for Affine2 {
    type Output = Affine2;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_affine2(rhs)
    }
}

impl std::ops::MulAssign for Affine2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl std::ops::Mul<Vec2> for Affine2 {
    type Output = Vec2;

    #[inline(always)]
    fn mul(self, rhs: Vec2) -> Self::Output {
        self.transform_vec2(rhs)
    }
}

impl std::ops::Mul<Point2> for Affine2 {
    type Output = Point2;

    #[inline(always)]
    fn mul(self, rhs: Point2) -> Self::Output {
        self.transform_point2(rhs)
    }
}
