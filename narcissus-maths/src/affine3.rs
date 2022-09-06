use crate::{Mat3, Vec3};

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Affine3 {
    matrix: Mat3,
    translate: Vec3,
}
