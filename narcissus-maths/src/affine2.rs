use crate::{Mat2, Vec2};

/// Matrix and translation vector which together represent a 2d affine transformation.
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Affine2 {
    matrix: Mat2,
    translate: Vec2,
}
