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
}
