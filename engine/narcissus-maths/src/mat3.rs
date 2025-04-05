use crate::{HalfTurn, Point2, Point3, Vec2, Vec3, sin_cos_pi_f32};

/// 3x3 matrix.
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat3(pub [f32; 9]);

impl std::fmt::Debug for Mat3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "Mat3 [")?;
            for row in self.as_rows() {
                writeln!(f, "\t{row:?}")?;
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

    /// Construct a matrix with the provided `diagonal` and all other values set to
    /// `0.0`.
    pub const fn from_diagonal(diagonal: Vec3) -> Mat3 {
        Mat3::from_rows([
            [diagonal.x, 0.0, 0.0],
            [0.0, diagonal.y, 0.0],
            [0.0, 0.0, diagonal.z],
        ])
    }

    /// Construct a transformation matrix which scales along the coordinate axis by
    /// the values given in `scale`.
    pub const fn from_scale(scale: Vec3) -> Mat3 {
        Mat3::from_diagonal(scale)
    }

    /// Constructs a transformation matrix which rotates around the given `axis` by
    /// `angle`.
    ///
    /// In a right-handed coordinate system, positive angles rotate
    /// counter-clockwise around `axis` where `axis` is pointing toward the
    /// observer.
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

    /// Returns `true` if all elements are finite.
    ///
    /// If any element is `NaN`, positive infinity, or negative infinity, returns
    /// `false`.
    pub fn is_finite(&self) -> bool {
        let mut is_finite = true;
        for x in self.0 {
            is_finite &= x.is_finite();
        }
        is_finite
    }

    /// Returns `true` if any element is positive infinity, or negative infinity,
    /// and `false` otherwise.
    pub fn is_infinite(&self) -> bool {
        let mut is_infinite = false;
        for x in self.0 {
            is_infinite |= x.is_infinite();
        }
        is_infinite
    }

    /// Returns `true` if any element is `NaN`, and `false` otherwise.
    pub fn is_nan(&self) -> bool {
        let mut is_nan = false;
        for x in self.0 {
            is_nan |= x.is_nan();
        }
        is_nan
    }

    /// Returns the transpose of `self`.
    #[must_use]
    #[inline(always)]
    pub fn transpose(self) -> Mat3 {
        let m = &self.0;
        Mat3::from_rows([[m[0], m[3], m[6]], [m[1], m[4], m[7]], [m[2], m[5], m[8]]])
    }

    #[must_use]
    #[inline]
    pub fn transform_point2(self: &Mat3, point: Point2) -> Point2 {
        let vec = Vec3::new(point.x, point.y, 1.0);
        let rows = self.as_rows();
        Point2::new(
            Vec3::dot(rows[0].into(), vec),
            Vec3::dot(rows[1].into(), vec),
        )
    }

    #[must_use]
    #[inline]
    pub fn transform_vec2(self: &Mat3, vec: Vec2) -> Vec2 {
        let vec = Vec3::new(vec.x, vec.y, 0.0);
        let rows = self.as_rows();
        Vec2::new(
            Vec3::dot(rows[0].into(), vec),
            Vec3::dot(rows[1].into(), vec),
        )
    }

    #[must_use]
    #[inline]
    pub fn transform_point3(self: &Mat3, point: Point3) -> Point3 {
        self.transform_vec3(point.as_vec3()).as_point3()
    }

    #[must_use]
    #[inline]
    pub fn transform_vec3(self: &Mat3, vec: Vec3) -> Vec3 {
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
        let lhs = self.as_rows();
        let rhs = rhs.as_rows();
        Mat3::from_rows([
            [
                lhs[0][0] * rhs[0][0] + lhs[0][1] * rhs[1][0] + lhs[0][2] * rhs[2][0],
                lhs[0][0] * rhs[0][1] + lhs[0][1] * rhs[1][1] + lhs[0][2] * rhs[2][1],
                lhs[0][0] * rhs[0][2] + lhs[0][1] * rhs[1][2] + lhs[0][2] * rhs[2][2],
            ],
            [
                lhs[1][0] * rhs[0][0] + lhs[1][1] * rhs[1][0] + lhs[1][2] * rhs[2][0],
                lhs[1][0] * rhs[0][1] + lhs[1][1] * rhs[1][1] + lhs[1][2] * rhs[2][1],
                lhs[1][0] * rhs[0][2] + lhs[1][1] * rhs[1][2] + lhs[1][2] * rhs[2][2],
            ],
            [
                lhs[2][0] * rhs[0][0] + lhs[2][1] * rhs[1][0] + lhs[2][2] * rhs[2][0],
                lhs[2][0] * rhs[0][1] + lhs[2][1] * rhs[1][1] + lhs[2][2] * rhs[2][1],
                lhs[2][0] * rhs[0][2] + lhs[2][1] * rhs[1][2] + lhs[2][2] * rhs[2][2],
            ],
        ])
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
        self.transform_vec3(rhs)
    }
}

impl std::ops::Mul<Point3> for Mat3 {
    type Output = Point3;

    #[inline(always)]
    fn mul(self, rhs: Point3) -> Self::Output {
        self.transform_point3(rhs)
    }
}

impl std::ops::Mul<Vec2> for Mat3 {
    type Output = Vec2;

    #[inline(always)]
    fn mul(self, rhs: Vec2) -> Self::Output {
        self.transform_vec2(rhs)
    }
}

impl std::ops::Mul<Point2> for Mat3 {
    type Output = Point2;

    #[inline(always)]
    fn mul(self, rhs: Point2) -> Self::Output {
        self.transform_point2(rhs)
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

        // Check we're rotating the right way, counter-clockwise.
        let rot_90_y = Mat3::from_axis_rotation(Vec3::Y, HalfTurn::new(0.5));
        assert_eq!(rot_90_y * Vec3::Z, Vec3::X);
        assert_eq!(rot_90_y * -Vec3::Z, -Vec3::X);
        assert_eq!(rot_90_y * Vec3::X, -Vec3::Z);
        assert_eq!(rot_90_y * -Vec3::X, Vec3::Z);
    }

    #[test]
    fn mul() {
        assert_eq!(I * I, I);
        assert_eq!(M * I, M);
        assert_eq!(I * M, M);
    }
}
