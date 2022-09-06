use crate::{Mat2, Vec2};

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Affine2 {
    matrix: Mat2,
    translate: Vec2,
}
