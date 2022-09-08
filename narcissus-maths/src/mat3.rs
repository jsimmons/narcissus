use crate::{Rad, Vec3};

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
    pub fn from_axis_angle(axis: Vec3, angle: Rad) -> Mat3 {
        let (sin, cos) = angle.as_f32().sin_cos();
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
}
