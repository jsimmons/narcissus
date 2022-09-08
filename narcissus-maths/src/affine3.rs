use crate::{Mat3, Vec3};

/// Matrix and translation vector which together represent a 3d affine transformation.
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Affine3 {
    matrix: Mat3,
    translate: Vec3,
}
