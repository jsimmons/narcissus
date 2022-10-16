use crate::{sin_cos_pi_f32, HalfTurn, Point2, Point3, Vec2, Vec3};

/// 3x3 matrix.
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat3(pub [f32; 9]);

impl std::fmt::Debug for Mat3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "Mat3 [")?;
            for row in self.as_rows() {
                writeln!(f, "\t{:?}", row)?;
            }
            writeln!(f, "]")
        } else {
            writeln!(f, "Mat3 {:?}", self.as_rows())
        }
    }
}

impl Mat3 {
    pub const ZERO: Mat3 = Mat3::from_rows([[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]);
    pub const IDENTITY: Mat3 = Mat3::from_rows([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);

    #[inline(always)]
    pub fn as_rows(&self) -> &[[f32; 3]; 3] {
        unsafe { std::mem::transmute(&self.0) }
    }

    #[inline(always)]
    pub fn as_rows_mut(&mut self) -> &mut [[f32; 3]; 3] {
        unsafe { std::mem::transmute(&mut self.0) }
    }

    #[inline(always)]
    pub const fn from_rows(rows: [[f32; 3]; 3]) -> Self {
        unsafe { std::mem::transmute(rows) }
    }

    /// Construct a matrix with the provided `diagonal` and all other values set to `0.0`.
    pub const fn from_diagonal(diagonal: Vec3) -> Mat3 {
        Mat3::from_rows([
            [diagonal.x, 0.0, 0.0],
            [0.0, diagonal.y, 0.0],
            [0.0, 0.0, diagonal.z],
        ])
    }

    /// Construct a transformation matrix which scales along the coordinate axis by the values given in `scale`.
    pub const fn from_scale(scale: Vec3) -> Mat3 {
        Mat3::from_diagonal(scale)
    }

    /// Constructs a transformation matrix which rotates around the given `axis` by `angle`.
    pub fn from_axis_rotation(axis: Vec3, rotation: HalfTurn) -> Mat3 {
        let (sin, cos) = sin_cos_pi_f32(rotation.as_f32());
        let axis_sin = axis * sin;
        let axis_sq = axis * axis;
        let one_minus_cos = 1.0 - cos;
        let xy = axis.x * axis.y * one_minus_cos;
        let xz = axis.x * axis.z * one_minus_cos;
        let yz = axis.y * axis.z * one_minus_cos;
        Mat3::from_rows([
            [
                axis_sq.x * one_minus_cos + cos,
                xy - axis_sin.z,
                xz + axis_sin.y,
            ],
            [
                xy + axis_sin.z,
                axis_sq.y * one_minus_cos + cos,
                yz - axis_sin.x,
            ],
            [
                xz - axis_sin.y,
                yz + axis_sin.x,
                axis_sq.z * one_minus_cos + cos,
            ],
        ])
    }

    /// Returns the transpose of `self`.
    #[must_use]
    #[inline(always)]
    pub fn transpose(self) -> Mat3 {
        let m = &self.0;
        Mat3::from_rows([[m[0], m[3], m[6]], [m[1], m[4], m[7]], [m[2], m[5], m[8]]])
    }

    #[must_use]
    pub fn mul_mat3(self: &Mat3, rhs: Mat3) -> Mat3 {
        let mut result = Mat3::IDENTITY;
        {
            let result = result.as_rows_mut();
            let lhs = self.as_rows();
            let rhs = rhs.as_rows();
            for i in 0..3 {
                for j in 0..3 {
                    result[i][j] =
                        lhs[i][0] * rhs[0][j] + lhs[i][1] * rhs[1][j] + lhs[i][2] * rhs[2][j];
                }
            }
        }
        result
    }

    #[must_use]
    #[inline]
    pub fn mul_point2(self: &Mat3, point: Point2) -> Point2 {
        let vec = Vec3::new(point.x, point.y, 1.0);
        let rows = self.as_rows();
        Point2::new(
            Vec3::dot(rows[0].into(), vec),
            Vec3::dot(rows[1].into(), vec),
        )
    }

    #[must_use]
    #[inline]
    pub fn mul_vec2(self: &Mat3, vec: Vec2) -> Vec2 {
        let vec = Vec3::new(vec.x, vec.y, 0.0);
        let rows = self.as_rows();
        Vec2::new(
            Vec3::dot(rows[0].into(), vec),
            Vec3::dot(rows[1].into(), vec),
        )
    }

    #[must_use]
    #[inline]
    pub fn mul_point3(self: &Mat3, point: Point3) -> Point3 {
        self.mul_vec3(point.as_vec3()).as_point3()
    }

    #[must_use]
    #[inline]
    pub fn mul_vec3(self: &Mat3, vec: Vec3) -> Vec3 {
        let rows = self.as_rows();
        Vec3::new(
            Vec3::dot(rows[0].into(), vec),
            Vec3::dot(rows[1].into(), vec),
            Vec3::dot(rows[2].into(), vec),
        )
    }
}

impl std::ops::Mul for Mat3 {
    type Output = Mat3;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_mat3(rhs)
    }
}

impl std::ops::MulAssign for Mat3 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl std::ops::Mul<Vec3> for Mat3 {
    type Output = Vec3;

    #[inline(always)]
    fn mul(self, rhs: Vec3) -> Self::Output {
        self.mul_vec3(rhs)
    }
}

impl std::ops::Mul<Point3> for Mat3 {
    type Output = Point3;

    #[inline(always)]
    fn mul(self, rhs: Point3) -> Self::Output {
        self.mul_point3(rhs)
    }
}

impl std::ops::Mul<Vec2> for Mat3 {
    type Output = Vec2;

    #[inline(always)]
    fn mul(self, rhs: Vec2) -> Self::Output {
        self.mul_vec2(rhs)
    }
}

impl std::ops::Mul<Point2> for Mat3 {
    type Output = Point2;

    #[inline(always)]
    fn mul(self, rhs: Point2) -> Self::Output {
        self.mul_point2(rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::{HalfTurn, Mat3, Vec2, Vec3};

    const I: Mat3 = Mat3::IDENTITY;
    const M: Mat3 = Mat3::from_rows([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
    const T: Mat3 = Mat3::from_rows([[1.0, 4.0, 7.0], [2.0, 5.0, 8.0], [3.0, 6.0, 9.0]]);

    #[test]
    fn transpose() {
        assert_eq!(I.transpose(), I);
        assert_eq!(M.transpose(), T);
        assert_eq!(M.transpose().transpose(), M);
        assert_eq!(T.transpose().transpose(), T);
    }

    #[test]
    fn axis_angle() {
        let rot_180_x = Mat3::from_axis_rotation(Vec3::X, HalfTurn::new(1.0));
        assert_eq!(rot_180_x * Vec3::X, Vec3::X);
        assert_eq!(rot_180_x * Vec3::Y, -Vec3::Y);
        assert_eq!(rot_180_x * Vec3::Z, -Vec3::Z);
        assert_eq!(rot_180_x * Vec2::X, Vec2::X);
        assert_eq!(rot_180_x * Vec2::Y, -Vec2::Y);
        let rot_180_y = Mat3::from_axis_rotation(Vec3::Y, HalfTurn::new(1.0));
        assert_eq!(rot_180_y * Vec3::X, -Vec3::X);
        assert_eq!(rot_180_y * Vec3::Y, Vec3::Y);
        assert_eq!(rot_180_y * Vec3::Z, -Vec3::Z);
        assert_eq!(rot_180_y * Vec2::X, -Vec2::X);
        assert_eq!(rot_180_y * Vec2::Y, Vec2::Y);
        let rot_180_z = Mat3::from_axis_rotation(Vec3::Z, HalfTurn::new(1.0));
        assert_eq!(rot_180_z * Vec3::X, -Vec3::X);
        assert_eq!(rot_180_z * Vec3::Y, -Vec3::Y);
        assert_eq!(rot_180_z * Vec3::Z, Vec3::Z);
        assert_eq!(rot_180_z * Vec2::X, -Vec2::X);
        assert_eq!(rot_180_z * Vec2::Y, -Vec2::Y);
    }

    #[test]
    fn mul() {
        assert_eq!(I * I, I);
        assert_eq!(M * I, M);
        assert_eq!(I * M, M);
    }
}
