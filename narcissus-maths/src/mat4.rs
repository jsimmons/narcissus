use crate::{sin_cos_pi_f32, HalfTurn, Point2, Point3, Rad, Vec2, Vec3, Vec4};

/// 4x4 matrix.
///
/// Supports affine transformations.
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat4(pub [f32; 16]);

impl std::fmt::Debug for Mat4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "Mat4 [")?;
            for row in self.as_rows() {
                writeln!(f, "\t{:?}", row)?;
            }
            writeln!(f, "]")
        } else {
            writeln!(f, "Mat4 {:?}", self.as_rows())
        }
    }
}

impl Mat4 {
    pub const ZERO: Mat4 = Mat4::from_rows([
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ]);

    pub const IDENTITY: Mat4 = Mat4::from_rows([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    #[inline(always)]
    pub fn as_rows(&self) -> &[[f32; 4]; 4] {
        unsafe { std::mem::transmute(&self.0) }
    }

    #[inline(always)]
    pub fn as_rows_mut(&mut self) -> &mut [[f32; 4]; 4] {
        unsafe { std::mem::transmute(&mut self.0) }
    }

    #[inline(always)]
    pub const fn from_rows(rows: [[f32; 4]; 4]) -> Self {
        unsafe { std::mem::transmute(rows) }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn as_m128_array(&self) -> [std::arch::x86_64::__m128; 4] {
        use std::arch::x86_64::_mm_loadu_ps;
        unsafe {
            [
                _mm_loadu_ps(&self.0[0x0]),
                _mm_loadu_ps(&self.0[0x4]),
                _mm_loadu_ps(&self.0[0x8]),
                _mm_loadu_ps(&self.0[0xc]),
            ]
        }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn from_m128_array(values: [std::arch::x86_64::__m128; 4]) -> Self {
        use std::arch::x86_64::_mm_storeu_ps;

        let mut result = Mat4::IDENTITY;
        unsafe {
            _mm_storeu_ps(&mut result.0[0x0], values[0]);
            _mm_storeu_ps(&mut result.0[0x4], values[1]);
            _mm_storeu_ps(&mut result.0[0x8], values[2]);
            _mm_storeu_ps(&mut result.0[0xc], values[3]);
        }
        result
    }

    /// Construct a matrix with the provided `diagonal` and all other values set to `0.0`.
    pub const fn from_diagonal(diagonal: Vec4) -> Mat4 {
        Mat4::from_rows([
            [diagonal.x, 0.0, 0.0, 0.0],
            [0.0, diagonal.y, 0.0, 0.0],
            [0.0, 0.0, diagonal.z, 0.0],
            [0.0, 0.0, 0.0, diagonal.w],
        ])
    }

    /// Construct a transformation matrix which scales along the coordinate axes by the values given in `scale`.
    pub const fn from_scale(scale: Vec3) -> Mat4 {
        Mat4::from_rows([
            [scale.x, 0.0, 0.0, 0.0],
            [0.0, scale.y, 0.0, 0.0],
            [0.0, 0.0, scale.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Construct an affine transformation matrix with the given `translation` along the coordinate axes.
    pub const fn from_translation(translation: Vec3) -> Mat4 {
        Mat4::from_rows([
            [1.0, 0.0, 0.0, translation.x],
            [0.0, 1.0, 0.0, translation.y],
            [0.0, 0.0, 1.0, translation.z],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Constructs a transformation matrix which rotates around the given `axis` by `angle`.
    pub fn from_axis_rotation(axis: Vec3, rotation: HalfTurn) -> Mat4 {
        let (sin, cos) = sin_cos_pi_f32(rotation.as_f32());
        let axis_sin = axis * sin;
        let axis_sq = axis * axis;
        let one_minus_cos = 1.0 - cos;
        let xy = axis.x * axis.y * one_minus_cos;
        let xz = axis.x * axis.z * one_minus_cos;
        let yz = axis.y * axis.z * one_minus_cos;
        Mat4::from_rows([
            [
                axis_sq.x * one_minus_cos + cos,
                xy - axis_sin.z,
                xz + axis_sin.y,
                0.0,
            ],
            [
                xy + axis_sin.z,
                axis_sq.y * one_minus_cos + cos,
                yz - axis_sin.x,
                0.0,
            ],
            [
                xz - axis_sin.y,
                yz + axis_sin.x,
                axis_sq.z * one_minus_cos + cos,
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Constructs a 'look at' transformation from the given `eye` position, look at `center` point, and `up` vector.
    ///
    /// Src coordinate space: right-handed, +y-up.
    /// Dst coordinate space: right-handed, +y-up.
    pub fn look_at(eye: Point3, center: Point3, up: Vec3) -> Mat4 {
        let dir = center - eye;
        let eye = eye.as_vec3();
        let f = dir.normalized();
        let r = Vec3::cross(f, up).normalized();
        let u = Vec3::cross(r, f);
        let r_dot_eye = Vec3::dot(r, eye);
        let u_dot_eye = Vec3::dot(u, eye);
        let f_dot_eye = Vec3::dot(f, eye);
        Mat4::from_rows([
            [r.x, r.y, r.z, -r_dot_eye],
            [u.x, u.y, u.z, -u_dot_eye],
            [-f.x, -f.y, -f.z, f_dot_eye],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Creates an othographic projection matrix with \[0,1\] depth range.
    ///
    /// Destination coordinate space matches native vulkan clip space.
    ///
    /// Src coordinate space: right-handed, +y-up.
    /// Dst coordinate space: right-handed, -y-up, depth range \[0,1\].
    pub fn orthographic_zo(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Mat4 {
        let rml = right - left;
        let rpl = right + left;
        let tmb = top - bottom;
        let tpb = top + bottom;
        let fmn = far - near;
        Mat4::from_rows([
            [2.0 / rml, 0.0, 0.0, -(rpl / rml)],
            [0.0, -2.0 / tmb, 0.0, -(tpb / tmb)],
            [0.0, 0.0, -1.0 / fmn, -(near / fmn)],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Creates a perspective projection matrix with reversed infinite z and \[0,1\] depth range.
    ///
    /// Destination coordinate space matches native vulkan clip space.
    ///
    /// Src coordinate space: right-handed, +y up.
    /// Dst coordinate space: right-handed, -y up, depth range \[0,1\].
    pub fn perspective_rev_inf_zo(vertical_fov: Rad, aspect_ratio: f32, z_near: f32) -> Mat4 {
        let tan = (vertical_fov.as_f32() / 2.0).tan();
        let sy = 1.0 / tan;
        let sx = sy / aspect_ratio;
        Mat4::from_rows([
            [sx, 0.0, 0.0, 0.0],
            [0.0, -sy, 0.0, 0.0],
            [0.0, 0.0, 0.0, z_near],
            [0.0, 0.0, -1.0, 0.0],
        ])
    }

    /// Returns `true` if all elements are finite.
    ///
    /// If any element is `NaN`, positive infinity, or negative infinity, returns `false`.
    pub fn is_finite(&self) -> bool {
        let mut is_finite = true;
        for x in self.0 {
            is_finite &= x.is_finite();
        }
        is_finite
    }

    /// Returns `true` if any element is positive infinity, or negative infinity, and `false` otherwise.
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

    #[allow(dead_code)]
    #[inline(always)]
    fn transpose_base(self) -> Mat4 {
        let m = &self.0;
        Mat4::from_rows([
            [m[0x0], m[0x4], m[0x8], m[0xc]],
            [m[0x1], m[0x5], m[0x9], m[0xd]],
            [m[0x2], m[0x6], m[0xa], m[0xe]],
            [m[0x3], m[0x7], m[0xb], m[0xf]],
        ])
    }

    // Safety: Requires SSE2.
    #[inline]
    #[target_feature(enable = "sse2")]
    unsafe fn transpose_sse2(self) -> Mat4 {
        use std::arch::x86_64::_MM_TRANSPOSE4_PS;
        let [mut row0, mut row1, mut row2, mut row3] = self.as_m128_array();
        _MM_TRANSPOSE4_PS(&mut row0, &mut row1, &mut row2, &mut row3);
        Mat4::from_m128_array([row0, row1, row2, row3])
    }

    /// Returns the transpose of `self`.
    #[must_use]
    #[inline(always)]
    pub fn transpose(self) -> Mat4 {
        #[cfg(not(target_feature = "sse2"))]
        {
            self.transpose_base()
        }
        #[cfg(target_feature = "sse2")]
        unsafe {
            self.transpose_sse2()
        }
    }

    /// Transforms the given [`Vec2`] `vec` by `self`.
    #[must_use]
    #[inline]
    pub fn transform_vec2(&self, vec: Vec2) -> Vec2 {
        let vec = Vec4::new(vec.x, vec.y, 0.0, 0.0);
        let vec = self.transform_vec4(vec);
        Vec2::new(vec.x, vec.y)
    }

    /// Transforms the given [`Point2`] `point` by `self`.
    #[must_use]
    #[inline]
    pub fn transform_point2(&self, point: Point2) -> Point2 {
        let vec = Vec4::new(point.x, point.y, 0.0, 1.0);
        let vec = self.transform_vec4(vec);
        Point2::new(vec.x, vec.y)
    }

    /// Transforms the given [`Vec3`] `vec` by `self`.
    #[must_use]
    #[inline]
    pub fn transform_vec3(&self, vec: Vec3) -> Vec3 {
        let vec = Vec4::new(vec.x, vec.y, vec.z, 0.0);
        let vec = self.transform_vec4(vec);
        [vec.x, vec.y, vec.z].into()
    }

    /// Transforms the given [`Point3`] `point` by `self`.
    #[must_use]
    #[inline]
    pub fn transform_point3(&self, point: Point3) -> Point3 {
        let vec = Vec4::new(point.x, point.y, point.z, 1.0);
        let vec = self.transform_vec4(vec);
        Point3::new(vec.x, vec.y, vec.z)
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn transform_vec4_base(&self, vec: Vec4) -> Vec4 {
        let rows = self.as_rows();
        Vec4::new(
            Vec4::dot(rows[0].into(), vec),
            Vec4::dot(rows[1].into(), vec),
            Vec4::dot(rows[2].into(), vec),
            Vec4::dot(rows[3].into(), vec),
        )
    }

    // Safety: Requires SSE4.1.
    #[allow(dead_code)]
    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn transform_vec4_sse41(&self, vec: Vec4) -> Vec4 {
        use std::arch::x86_64::{_mm_hadd_ps, _mm_mul_ps};

        let vec = vec.into();
        let rows = self.as_m128_array();

        let values = _mm_hadd_ps(
            _mm_hadd_ps(_mm_mul_ps(rows[0], vec), _mm_mul_ps(rows[1], vec)),
            _mm_hadd_ps(_mm_mul_ps(rows[2], vec), _mm_mul_ps(rows[3], vec)),
        );

        values.into()
    }

    /// Transforms the given [`Vec4`] `vec` by `self`.
    #[must_use]
    #[inline(always)]
    pub fn transform_vec4(&self, vec: Vec4) -> Vec4 {
        #[cfg(not(target_feature = "sse4.1"))]
        {
            self.transform_vec4_base(vec)
        }

        #[cfg(target_feature = "sse4.1")]
        unsafe {
            self.transform_vec4_sse41(vec)
        }
    }
}

#[allow(dead_code)]
#[inline(always)]
fn mul_mat4_base(lhs: Mat4, rhs: Mat4) -> Mat4 {
    let mut result = Mat4::IDENTITY;
    {
        let result = result.as_rows_mut();
        let lhs = lhs.as_rows();
        let rhs = rhs.as_rows();
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = lhs[i][0] * rhs[0][j]
                    + lhs[i][1] * rhs[1][j]
                    + lhs[i][2] * rhs[2][j]
                    + lhs[i][3] * rhs[3][j];
            }
        }
    }
    result
}

// Safety: Requires SSE2.
#[allow(dead_code)]
#[inline]
#[target_feature(enable = "sse2")]
unsafe fn mul_mat4_sse2(lhs: Mat4, rhs: Mat4) -> Mat4 {
    use std::arch::x86_64::{__m128, _mm_add_ps, _mm_mul_ps, _mm_shuffle_ps};

    #[inline(always)]
    fn linear_combine(a: __m128, mat: &[__m128; 4]) -> __m128 {
        unsafe {
            let r = _mm_mul_ps(_mm_shuffle_ps(a, a, 0x00), mat[0]);
            let r = _mm_add_ps(r, _mm_mul_ps(_mm_shuffle_ps(a, a, 0x55), mat[1]));
            let r = _mm_add_ps(r, _mm_mul_ps(_mm_shuffle_ps(a, a, 0xaa), mat[2]));
            _mm_add_ps(r, _mm_mul_ps(_mm_shuffle_ps(a, a, 0xff), mat[3]))
        }
    }

    let lhs = lhs.as_m128_array();
    let rhs = rhs.as_m128_array();

    let x0 = linear_combine(lhs[0], &rhs);
    let x1 = linear_combine(lhs[1], &rhs);
    let x2 = linear_combine(lhs[2], &rhs);
    let x3 = linear_combine(lhs[3], &rhs);

    Mat4::from_m128_array([x0, x1, x2, x3])
}

// Safety: Requires AVX2.
#[allow(dead_code)]
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn mul_mat4_avx2(lhs: Mat4, rhs: Mat4) -> Mat4 {
    use std::arch::x86_64::{
        __m128, __m256, _mm256_add_ps, _mm256_broadcast_ps, _mm256_loadu_ps, _mm256_mul_ps,
        _mm256_shuffle_ps, _mm256_storeu_ps, _mm256_zeroupper,
    };

    #[inline(always)]
    unsafe fn two_linear_combine(a: __m256, m: &[__m128; 4]) -> __m256 {
        let m0 = _mm256_broadcast_ps(&m[0]);
        let m1 = _mm256_broadcast_ps(&m[1]);
        let m2 = _mm256_broadcast_ps(&m[2]);
        let m3 = _mm256_broadcast_ps(&m[3]);
        let r = _mm256_mul_ps(_mm256_shuffle_ps(a, a, 0x00), m0);
        let r = _mm256_add_ps(r, _mm256_mul_ps(_mm256_shuffle_ps(a, a, 0x55), m1));
        let r = _mm256_add_ps(r, _mm256_mul_ps(_mm256_shuffle_ps(a, a, 0xaa), m2));
        _mm256_add_ps(r, _mm256_mul_ps(_mm256_shuffle_ps(a, a, 0xff), m3))
    }

    _mm256_zeroupper();

    let a0 = _mm256_loadu_ps(&lhs.0[0]);
    let a1 = _mm256_loadu_ps(&lhs.0[8]);
    let rhs = rhs.as_m128_array();

    let x0 = two_linear_combine(a0, &rhs);
    let x1 = two_linear_combine(a1, &rhs);

    let mut result = Mat4::IDENTITY;
    _mm256_storeu_ps(&mut result.0[0], x0);
    _mm256_storeu_ps(&mut result.0[8], x1);
    result
}

impl std::ops::Mul for Mat4 {
    type Output = Mat4;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        #[cfg(not(target_feature = "sse2"))]
        {
            mul_mat4_base(self, rhs)
        }
        #[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
        unsafe {
            mul_mat4_sse2(self, rhs)
        }
        #[cfg(target_feature = "avx2")]
        unsafe {
            mul_mat4_avx2(self, rhs)
        }
    }
}

impl std::ops::MulAssign for Mat4 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl std::ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;

    #[inline(always)]
    fn mul(self, rhs: Vec4) -> Self::Output {
        self.transform_vec4(rhs)
    }
}

impl std::ops::Mul<Vec3> for Mat4 {
    type Output = Vec3;

    #[inline(always)]
    fn mul(self, rhs: Vec3) -> Self::Output {
        self.transform_vec3(rhs)
    }
}

impl std::ops::Mul<Point3> for Mat4 {
    type Output = Point3;

    #[inline(always)]
    fn mul(self, rhs: Point3) -> Self::Output {
        self.transform_point3(rhs)
    }
}

impl std::ops::Mul<Vec2> for Mat4 {
    type Output = Vec2;

    #[inline(always)]
    fn mul(self, rhs: Vec2) -> Self::Output {
        self.transform_vec2(rhs)
    }
}

impl std::ops::Mul<Point2> for Mat4 {
    type Output = Point2;

    #[inline(always)]
    fn mul(self, rhs: Point2) -> Self::Output {
        self.transform_point2(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const I: Mat4 = Mat4::IDENTITY;
    const M: Mat4 = Mat4::from_rows([
        [1.0, 2.0, 3.0, 4.0],
        [5.0, 6.0, 7.0, 8.0],
        [9.0, 10.0, 11.0, 12.0],
        [13.0, 14.0, 15.0, 16.0],
    ]);
    const T: Mat4 = Mat4::from_rows([
        [1.0, 5.0, 9.0, 13.0],
        [2.0, 6.0, 10.0, 14.0],
        [3.0, 7.0, 11.0, 15.0],
        [4.0, 8.0, 12.0, 16.0],
    ]);

    const SCALE: Mat4 = Mat4::from_scale(Vec3::splat(2.0));
    const TRANSLATE: Mat4 = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));

    const V2: Vec2 = Vec2::new(1.0, 2.0);
    const V3: Vec3 = Vec3::new(1.0, 2.0, 3.0);
    const V4: Vec4 = Vec4::new(1.0, 2.0, 3.0, 4.0);
    const P2: Point2 = Point2::new(1.0, 2.0);
    const P3: Point3 = Point3::new(1.0, 2.0, 3.0);

    #[test]
    fn transpose() {
        assert_eq!(I.transpose(), I);
        assert_eq!(M.transpose(), T);
        assert_eq!(M.transpose().transpose(), M);
        assert_eq!(T.transpose().transpose(), T);
    }

    #[test]
    fn axis_angle() {
        let rot_180_x = Mat4::from_axis_rotation(Vec3::X, HalfTurn::new(1.0));
        assert_eq!(rot_180_x * Vec3::X, Vec3::X);
        assert_eq!(rot_180_x * Vec3::Y, -Vec3::Y);
        assert_eq!(rot_180_x * Vec3::Z, -Vec3::Z);
        assert_eq!(rot_180_x * Vec2::X, Vec2::X);
        assert_eq!(rot_180_x * Vec2::Y, -Vec2::Y);
        let rot_180_y = Mat4::from_axis_rotation(Vec3::Y, HalfTurn::new(1.0));
        assert_eq!(rot_180_y * Vec3::X, -Vec3::X);
        assert_eq!(rot_180_y * Vec3::Y, Vec3::Y);
        assert_eq!(rot_180_y * Vec3::Z, -Vec3::Z);
        assert_eq!(rot_180_y * Vec2::X, -Vec2::X);
        assert_eq!(rot_180_y * Vec2::Y, Vec2::Y);
        let rot_180_z = Mat4::from_axis_rotation(Vec3::Z, HalfTurn::new(1.0));
        assert_eq!(rot_180_z * Vec3::X, -Vec3::X);
        assert_eq!(rot_180_z * Vec3::Y, -Vec3::Y);
        assert_eq!(rot_180_z * Vec3::Z, Vec3::Z);
        assert_eq!(rot_180_z * Vec2::X, -Vec2::X);
        assert_eq!(rot_180_z * Vec2::Y, -Vec2::Y);
    }

    #[test]
    fn mul() {
        assert_eq!(I * I, I);
        assert_eq!(SCALE * I, SCALE);
        assert_eq!(M * I, M);

        assert_eq!(mul_mat4_base(I, I), I);
        assert_eq!(mul_mat4_base(SCALE, I), SCALE);
        assert_eq!(mul_mat4_base(M, I), M);

        if std::is_x86_feature_detected!("sse2") {
            assert_eq!(unsafe { mul_mat4_sse2(I, I) }, I);
            assert_eq!(unsafe { mul_mat4_sse2(SCALE, I) }, SCALE);
            assert_eq!(unsafe { mul_mat4_sse2(M, I) }, M);
        }

        if std::is_x86_feature_detected!("avx2") {
            assert_eq!(unsafe { mul_mat4_avx2(I, I) }, I);
            assert_eq!(unsafe { mul_mat4_avx2(SCALE, I) }, SCALE);
            assert_eq!(unsafe { mul_mat4_avx2(M, I) }, M);
        }
    }

    #[test]
    fn mul_vec2() {
        assert_eq!(I * Vec2::ZERO, Vec2::ZERO);
        assert_eq!(I * V2, V2);
        assert_eq!(SCALE * Vec2::ZERO, Vec2::ZERO);
        assert_eq!(SCALE * Vec2::ONE, Vec2::splat(2.0));
        assert_eq!(TRANSLATE * Vec2::ZERO, Vec2::ZERO);
    }

    #[test]
    fn mul_point2() {
        assert_eq!(I * Point2::ZERO, Point2::ZERO);
        assert_eq!(I * P2, P2);
        assert_eq!(SCALE * Point2::ZERO, Point2::ZERO);
        assert_eq!(SCALE * Point2::ONE, Point2::splat(2.0));
        assert_eq!(TRANSLATE * Point2::ZERO, P2);
    }

    #[test]
    fn mul_vec3() {
        assert_eq!(I * Vec3::ZERO, Vec3::ZERO);
        assert_eq!(I * V3, V3);
        assert_eq!(SCALE * Vec3::ZERO, Vec3::ZERO);
        assert_eq!(SCALE * Vec3::ONE, Vec3::splat(2.0));
        assert_eq!(TRANSLATE * Vec3::ZERO, Vec3::ZERO);
    }

    #[test]
    fn mul_point3() {
        assert_eq!(I * Point3::ZERO, Point3::ZERO);
        assert_eq!(I * P3, P3);
        assert_eq!(SCALE * Point3::ZERO, Point3::ZERO);
        assert_eq!(SCALE * Point3::ONE, Point3::splat(2.0));
        assert_eq!(TRANSLATE * Point3::ZERO, P3);
    }

    #[test]
    fn mul_vec4() {
        assert_eq!(I * Vec4::ZERO, Vec4::ZERO);
        assert_eq!(I * V4, V4);
        assert_eq!(SCALE * Vec4::ZERO, Vec4::ZERO);
        assert_eq!(SCALE * Vec4::ONE, Vec4::new(2.0, 2.0, 2.0, 1.0));
        assert_eq!(
            TRANSLATE * Vec4::new(0.0, 0.0, 0.0, 1.0),
            Vec4::new(1.0, 2.0, 3.0, 1.0)
        );

        if std::is_x86_feature_detected!("sse4.1") {
            unsafe {
                assert_eq!(I.transform_vec4_sse41(Vec4::ZERO), Vec4::ZERO);
                assert_eq!(I.transform_vec4_sse41(V4), V4);
                assert_eq!(SCALE.transform_vec4_sse41(Vec4::ZERO), Vec4::ZERO);
                assert_eq!(
                    SCALE.transform_vec4_sse41(Vec4::ONE),
                    Vec4::new(2.0, 2.0, 2.0, 1.0)
                );
                assert_eq!(
                    TRANSLATE.transform_vec4_sse41(Vec4::new(0.0, 0.0, 0.0, 1.0)),
                    Vec4::new(1.0, 2.0, 3.0, 1.0)
                );
            }
        }
    }
}
