use crate::{Point2, Vec2};

/// 2x2 matrix.
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat2(pub [f32; 4]);

impl std::fmt::Debug for Mat2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "Mat2 [")?;
            for row in self.as_rows() {
                writeln!(f, "\t{:?}", row)?;
            }
            writeln!(f, "]")
        } else {
            writeln!(f, "Mat2 {:?}", self.as_rows())
        }
    }
}

impl Mat2 {
    pub const ZERO: Mat2 = Mat2::from_rows([[0.0, 0.0], [0.0, 0.0]]);
    pub const IDENTITY: Mat2 = Mat2::from_rows([[1.0, 0.0], [0.0, 1.0]]);

    #[inline(always)]
    pub fn as_rows(&self) -> &[[f32; 2]; 2] {
        unsafe { std::mem::transmute(&self.0) }
    }

    #[inline(always)]
    pub fn as_rows_mut(&mut self) -> &mut [[f32; 2]; 2] {
        unsafe { std::mem::transmute(&mut self.0) }
    }

    #[inline(always)]
    pub const fn from_rows(rows: [[f32; 2]; 2]) -> Self {
        unsafe { std::mem::transmute(rows) }
    }

    /// Construct a matrix with the provided `diagonal` and all other values set to `0.0`.
    pub const fn from_diagonal(diagonal: Vec2) -> Mat2 {
        Mat2::from_rows([[diagonal.x, 0.0], [0.0, diagonal.y]])
    }

    /// Construct a transformation matrix which scales along the coordinate axis by the values given in `scale`.
    pub const fn from_scale(scale: Vec2) -> Mat2 {
        Mat2::from_diagonal(scale)
    }

    /// Returns the transpose of `self`.
    #[must_use]
    #[inline(always)]
    pub fn transpose(self) -> Mat2 {
        let m = &self.0;
        Mat2::from_rows([[m[0], m[2]], [m[1], m[3]]])
    }

    #[must_use]
    pub fn mul_mat2(self: &Mat2, rhs: Mat2) -> Mat2 {
        let mut result = Mat2::IDENTITY;
        {
            let result = result.as_rows_mut();
            let lhs = self.as_rows();
            let rhs = rhs.as_rows();
            for i in 0..2 {
                for j in 0..2 {
                    result[i][j] = lhs[i][0] * rhs[0][j] + lhs[i][1] * rhs[1][j];
                }
            }
        }
        result
    }

    #[must_use]
    #[inline]
    pub fn mul_point2(self: &Mat2, point: Point2) -> Point2 {
        self.mul_vec2(point.as_vec2()).as_point2()
    }

    #[must_use]
    #[inline]
    pub fn mul_vec2(self: &Mat2, vec: Vec2) -> Vec2 {
        let vec = Vec2::new(vec.x, vec.y);
        let rows = self.as_rows();
        Vec2::new(
            Vec2::dot(rows[0].into(), vec),
            Vec2::dot(rows[1].into(), vec),
        )
    }
}

impl std::ops::Mul for Mat2 {
    type Output = Mat2;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_mat2(rhs)
    }
}

impl std::ops::MulAssign for Mat2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl std::ops::Mul<Vec2> for Mat2 {
    type Output = Vec2;

    #[inline(always)]
    fn mul(self, rhs: Vec2) -> Self::Output {
        self.mul_vec2(rhs)
    }
}

impl std::ops::Mul<Point2> for Mat2 {
    type Output = Point2;

    #[inline(always)]
    fn mul(self, rhs: Point2) -> Self::Output {
        self.mul_point2(rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::Mat2;

    const I: Mat2 = Mat2::IDENTITY;
    const M: Mat2 = Mat2::from_rows([[1.0, 2.0], [3.0, 4.0]]);
    const T: Mat2 = Mat2::from_rows([[1.0, 3.0], [2.0, 4.0]]);

    #[test]
    fn transpose() {
        assert_eq!(I.transpose(), I);
        assert_eq!(M.transpose(), T);
        assert_eq!(M.transpose().transpose(), M);
        assert_eq!(T.transpose().transpose(), T);
    }

    #[test]
    fn mul() {
        assert_eq!(I * I, I);
        assert_eq!(M * I, M);
        assert_eq!(I * M, M);
    }
}
