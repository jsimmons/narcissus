use crate::{Mat3, Point3, Vec3};

/// Matrix and translation vector which together represent a 3d affine
/// transformation.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Affine3 {
    pub matrix: Mat3,
    pub translation: Vec3,
}

impl Affine3 {
    pub const ZERO: Affine3 = Affine3 {
        matrix: Mat3::ZERO,
        translation: Vec3::ZERO,
    };

    pub const IDENTITY: Affine3 = Affine3 {
        matrix: Mat3::IDENTITY,
        translation: Vec3::ZERO,
    };

    pub fn new(matrix: Mat3, translation: Vec3) -> Self {
        Self {
            matrix,
            translation,
        }
    }

    pub fn mul_affine3(&self, rhs: Affine3) -> Affine3 {
        Self {
            matrix: self.matrix * rhs.matrix,
            translation: self.translation + rhs.translation,
        }
    }

    pub fn transform_vec3(&self, vec: Vec3) -> Vec3 {
        self.matrix * vec
    }

    pub fn transform_point3(&self, point: Point3) -> Point3 {
        self.matrix * point + self.translation
    }
}

impl std::ops::Mul for Affine3 {
    type Output = Affine3;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_affine3(rhs)
    }
}

impl std::ops::MulAssign for Affine3 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl std::ops::Mul<Vec3> for Affine3 {
    type Output = Vec3;

    #[inline(always)]
    fn mul(self, rhs: Vec3) -> Self::Output {
        self.transform_vec3(rhs)
    }
}

impl std::ops::Mul<Point3> for Affine3 {
    type Output = Point3;

    #[inline(always)]
    fn mul(self, rhs: Point3) -> Self::Output {
        self.transform_point3(rhs)
    }
}
