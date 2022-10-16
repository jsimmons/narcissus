use crate::{Mat2, Point2, Vec2};

/// Matrix and translation vector which together represent a 2d affine transformation.
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Affine2 {
    matrix: Mat2,
    translate: Vec2,
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

    pub fn mul_vec2(&self, vec: Vec2) -> Vec2 {
        self.matrix * vec + self.translate
    }

    pub fn mul_point2(&self, point: Point2) -> Point2 {
        self.matrix * point + self.translate
    }
}
