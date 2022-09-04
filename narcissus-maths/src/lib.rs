#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::splat(0.0);

    pub const X: Self = Self::new(1.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0);

    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub const fn splat(value: f32) -> Self {
        Self { x: value, y: value }
    }

    #[inline(always)]
    pub fn as_array(self) -> [f32; 2] {
        unsafe { std::mem::transmute(self) }
    }

    #[inline(always)]
    pub fn from_array(values: [f32; 2]) -> Self {
        unsafe { std::mem::transmute(values) }
    }

    #[inline]
    pub fn distance(a: Self, b: Self) -> f32 {
        (a - b).length()
    }

    #[inline]
    pub fn distance_sq(a: Self, b: Self) -> f32 {
        (a - b).length_sq()
    }

    #[inline]
    pub fn dot(a: Self, b: Self) -> f32 {
        a.x * b.x + a.y * b.y
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        Self::dot(self, self)
    }

    #[inline]
    pub fn ceil(self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
        }
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    #[inline]
    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul for Vec2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl std::ops::Div for Vec2 {
    type Output = Vec2;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl std::ops::AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::MulAssign for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl std::ops::DivAssign for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::splat(0.0);

    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline(always)]
    pub const fn splat(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }

    #[inline(always)]
    pub fn as_array(self) -> [f32; 3] {
        unsafe { std::mem::transmute(self) }
    }

    #[inline(always)]
    pub fn from_array(values: [f32; 3]) -> Self {
        unsafe { std::mem::transmute(values) }
    }

    #[inline]
    pub fn distance(a: Self, b: Self) -> f32 {
        (a - b).length()
    }

    #[inline]
    pub fn distance_sq(a: Self, b: Self) -> f32 {
        (a - b).length_sq()
    }

    #[inline]
    pub fn dot(a: Self, b: Self) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        Self::dot(self, self)
    }

    #[inline]
    pub fn ceil(self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
            z: self.z.ceil(),
        }
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor(),
        }
    }

    #[inline]
    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
            z: self.z.round(),
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Mul for Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl std::ops::Div for Vec3 {
    type Output = Vec3;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl std::ops::AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl std::ops::MulAssign for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl std::ops::DivAssign for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Self = Self::splat(0.0);

    pub const X: Self = Self::new(1.0, 0.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0, 0.0);
    pub const W: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline(always)]
    pub const fn splat(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }

    #[inline(always)]
    pub fn as_array(self) -> [f32; 4] {
        unsafe { std::mem::transmute(self) }
    }

    #[inline(always)]
    pub fn from_array(values: [f32; 4]) -> Self {
        unsafe { std::mem::transmute(values) }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn as_m128(self) -> std::arch::x86_64::__m128 {
        unsafe { std::arch::x86_64::_mm_loadu_ps(&self.x) }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn from_m128(values: std::arch::x86_64::__m128) -> Self {
        use std::arch::x86_64::_mm_storeu_ps;
        let mut result = Vec4::ZERO;
        unsafe { _mm_storeu_ps(&mut result.x, values) }
        result
    }

    #[inline]
    pub fn distance(a: Self, b: Self) -> f32 {
        (a - b).length()
    }

    #[inline]
    pub fn distance_sq(a: Self, b: Self) -> f32 {
        (a - b).length_sq()
    }

    #[inline]
    pub fn dot(a: Self, b: Self) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        Self::dot(self, self)
    }

    #[inline]
    pub fn ceil(self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
            z: self.z.ceil(),
            w: self.w.ceil(),
        }
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor(),
            w: self.w.floor(),
        }
    }

    #[inline]
    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
            z: self.z.round(),
            w: self.w.round(),
        }
    }
}

impl std::ops::Add for Vec4 {
    type Output = Vec4;

    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe {
            use std::arch::x86_64::_mm_add_ps;
            Vec4::from_m128(_mm_add_ps(self.as_m128(), rhs.as_m128()))
        }
    }
}

impl std::ops::Sub for Vec4 {
    type Output = Vec4;

    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe {
            use std::arch::x86_64::_mm_sub_ps;
            Vec4::from_m128(_mm_sub_ps(self.as_m128(), rhs.as_m128()))
        }
    }
}

impl std::ops::Mul for Vec4 {
    type Output = Vec4;

    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe {
            use std::arch::x86_64::_mm_mul_ps;
            Vec4::from_m128(_mm_mul_ps(self.as_m128(), rhs.as_m128()))
        }
    }
}

impl std::ops::Div for Vec4 {
    type Output = Vec4;

    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
            w: self.w / rhs.w,
        }
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        unsafe {
            use std::arch::x86_64::_mm_div_ps;
            Vec4::from_m128(_mm_div_ps(self.as_m128(), rhs.as_m128()))
        }
    }
}

impl std::ops::AddAssign for Vec4 {
    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        use std::arch::x86_64::_mm_add_ps;
        unsafe {
            *self = Vec4::from_m128(_mm_add_ps(self.as_m128(), rhs.as_m128()));
        }
    }
}

impl std::ops::SubAssign for Vec4 {
    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        unsafe {
            *self = Vec4::from_m128(std::arch::x86_64::_mm_sub_ps(self.as_m128(), rhs.as_m128()));
        }
    }
}

impl std::ops::MulAssign for Vec4 {
    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
        self.w *= rhs.w;
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        unsafe {
            *self = Vec4::from_m128(std::arch::x86_64::_mm_mul_ps(self.as_m128(), rhs.as_m128()));
        }
    }
}

impl std::ops::DivAssign for Vec4 {
    #[cfg(not(target_feature = "sse2"))]
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
        self.w /= rhs.w;
    }

    #[cfg(target_feature = "sse2")]
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        unsafe {
            *self = Vec4::from_m128(std::arch::x86_64::_mm_div_ps(self.as_m128(), rhs.as_m128()));
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Quat {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat4(pub [f32; 16]);

impl std::fmt::Debug for Mat4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Mat4 [")?;
        if f.alternate() {
            writeln!(f)?;
            for row in self.as_rows_array() {
                f.write_str("\t")?;
                for value in row {
                    f.write_fmt(format_args!("{value}, "))?;
                }
                f.write_str("\n")?;
            }
        } else {
            for value in &self.0[..15] {
                f.write_fmt(format_args!("{value}, "))?;
            }
            f.write_fmt(format_args!("{}", self.0[15]))?;
        }
        f.write_str("]")
    }
}

impl Mat4 {
    pub const ZERO: Mat4 = Mat4::from_rows_array([
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ]);

    pub const IDENTITY: Mat4 = Mat4::from_rows_array([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    #[inline(always)]
    pub fn as_rows_array(&self) -> &[[f32; 4]; 4] {
        unsafe { std::mem::transmute(&self.0) }
    }

    #[inline(always)]
    pub fn as_rows_array_mut(&mut self) -> &mut [[f32; 4]; 4] {
        unsafe { std::mem::transmute(&mut self.0) }
    }

    #[inline(always)]
    pub const fn from_rows_array(rows: [[f32; 4]; 4]) -> Self {
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

    pub fn from_scale(scale: Vec3) -> Mat4 {
        Mat4::from_rows_array([
            [scale.x, 0.0, 0.0, 0.0],
            [0.0, scale.y, 0.0, 0.0],
            [0.0, 0.0, scale.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn transpose_base(self) -> Mat4 {
        let m = &self.0;
        Mat4::from_rows_array([
            [m[0x0], m[0x4], m[0x8], m[0xc]],
            [m[0x1], m[0x5], m[0x9], m[0xd]],
            [m[0x2], m[0x6], m[0xa], m[0xe]],
            [m[0x3], m[0x7], m[0xb], m[0xf]],
        ])
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    unsafe fn transpose_sse2(self) -> Mat4 {
        use std::arch::x86_64::_MM_TRANSPOSE4_PS;
        let [mut row0, mut row1, mut row2, mut row3] = self.as_m128_array();
        _MM_TRANSPOSE4_PS(&mut row0, &mut row1, &mut row2, &mut row3);
        Mat4::from_m128_array([row0, row1, row2, row3])
    }

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

    #[must_use]
    #[inline]
    pub fn mul_vec3(&self, vec: Vec3) -> Vec3 {
        let vec = Vec4::new(vec.x, vec.y, vec.z, 1.0);
        let vec = self.mul_vec4(vec);
        Vec3::new(vec.x, vec.y, vec.z)
    }

    #[inline(always)]
    fn mul_vec4_base(&self, vec: Vec4) -> Vec4 {
        let rows = self.as_rows_array();
        Vec4::new(
            Vec4::dot(Vec4::from_array(rows[0]), vec),
            Vec4::dot(Vec4::from_array(rows[1]), vec),
            Vec4::dot(Vec4::from_array(rows[2]), vec),
            Vec4::dot(Vec4::from_array(rows[3]), vec),
        )
    }

    #[allow(dead_code)]
    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn mul_vec4_sse41(&self, vec: Vec4) -> Vec4 {
        use std::arch::x86_64::{_mm_hadd_ps, _mm_mul_ps};

        let vec = vec.as_m128();
        let rows = self.as_m128_array();

        let values = _mm_hadd_ps(
            _mm_hadd_ps(_mm_mul_ps(rows[0], vec), _mm_mul_ps(rows[1], vec)),
            _mm_hadd_ps(_mm_mul_ps(rows[2], vec), _mm_mul_ps(rows[3], vec)),
        );

        Vec4::from_m128(values)
    }

    #[must_use]
    #[inline(always)]
    pub fn mul_vec4(&self, vec: Vec4) -> Vec4 {
        #[cfg(not(target_feature = "sse4.1"))]
        {
            self.mul_vec4_base(vec)
        }

        #[cfg(target_feature = "sse4.1")]
        unsafe {
            self.mul_vec4_sse41(vec)
        }
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn mul_mat4_base(self: &Mat4, rhs: Mat4) -> Mat4 {
        let mut result = Mat4::IDENTITY;
        {
            let result = result.as_rows_array_mut();
            let lhs = self.as_rows_array();
            let rhs = rhs.as_rows_array();
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

    #[allow(dead_code)]
    #[inline]
    #[target_feature(enable = "sse2")]
    unsafe fn mul_mat4_sse2(&self, rhs: Mat4) -> Mat4 {
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

        let lhs = self.as_m128_array();
        let rhs = rhs.as_m128_array();

        let x0 = linear_combine(lhs[0], &rhs);
        let x1 = linear_combine(lhs[1], &rhs);
        let x2 = linear_combine(lhs[2], &rhs);
        let x3 = linear_combine(lhs[3], &rhs);

        Mat4::from_m128_array([x0, x1, x2, x3])
    }

    #[allow(dead_code)]
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn mul_mat4_avx2(&self, rhs: Mat4) -> Mat4 {
        use std::arch::x86_64::{
            __m128, __m256, _mm256_add_ps, _mm256_broadcast_ps, _mm256_loadu_ps, _mm256_mul_ps,
            _mm256_shuffle_ps, _mm256_storeu_ps, _mm256_zeroupper,
        };

        #[inline(always)]
        unsafe fn two_linear_combine(a: __m256, mat: &[__m128; 4]) -> __m256 {
            let r = _mm256_mul_ps(_mm256_shuffle_ps(a, a, 0x00), _mm256_broadcast_ps(&mat[0]));
            let r = _mm256_add_ps(
                r,
                _mm256_mul_ps(_mm256_shuffle_ps(a, a, 0x55), _mm256_broadcast_ps(&mat[1])),
            );
            let r = _mm256_add_ps(
                r,
                _mm256_mul_ps(_mm256_shuffle_ps(a, a, 0xaa), _mm256_broadcast_ps(&mat[2])),
            );
            _mm256_add_ps(
                r,
                _mm256_mul_ps(_mm256_shuffle_ps(a, a, 0xff), _mm256_broadcast_ps(&mat[3])),
            )
        }

        _mm256_zeroupper();

        let a0 = _mm256_loadu_ps(&self.0[0]);
        let a1 = _mm256_loadu_ps(&self.0[8]);
        let rhs = rhs.as_m128_array();

        let x0 = two_linear_combine(a0, &rhs);
        let x1 = two_linear_combine(a1, &rhs);

        let mut result = Mat4::IDENTITY;
        _mm256_storeu_ps(&mut result.0[0], x0);
        _mm256_storeu_ps(&mut result.0[8], x1);
        result
    }
}

impl std::ops::Mul for Mat4 {
    type Output = Mat4;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        #[cfg(not(target_feature = "sse2"))]
        {
            self.mul_mat4_base(rhs)
        }
        #[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
        unsafe {
            self.mul_mat4_sse2(rhs)
        }
        #[cfg(target_feature = "avx2")]
        unsafe {
            self.mul_mat4_avx2(rhs)
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
        self.mul_vec4(rhs)
    }
}

impl std::ops::Mul<Vec3> for Mat4 {
    type Output = Vec3;

    #[inline(always)]
    fn mul(self, rhs: Vec3) -> Self::Output {
        self.mul_vec3(rhs)
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat43(pub [f32; 12]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_vec4() {
        assert_eq!(
            Vec4::new(1.0, 2.0, 3.0, 4.0) + Vec4::new(4.0, 3.0, 2.0, 1.0),
            Vec4::splat(5.0)
        );
        assert_eq!(
            Vec4::new(4.0, 3.0, 2.0, 1.0) - Vec4::new(3.0, 2.0, 1.0, 0.0),
            Vec4::splat(1.0)
        );
        assert_eq!(
            Vec4::new(1.0, 2.0, 3.0, 4.0) * Vec4::new(4.0, 3.0, 2.0, 1.0),
            Vec4::new(4.0, 6.0, 6.0, 4.0)
        );
        assert_eq!(
            Vec4::new(1.0, 2.0, 3.0, 4.0) / Vec4::splat(2.0),
            Vec4::new(0.5, 1.0, 1.5, 2.0)
        );

        assert_eq!(Vec4::new(2.0, 2.0, 2.0, 2.0).length_sq(), 16.0);
        assert_eq!(Vec4::new(2.0, 2.0, 2.0, 2.0).length(), 4.0);

        assert_eq!(Vec4::distance_sq(Vec4::splat(-1.0), Vec4::splat(1.0)), 16.0);
        assert_eq!(Vec4::distance(Vec4::splat(-1.0), Vec4::splat(1.0)), 4.0);
    }

    #[test]
    fn mat4_basics() {
        assert_eq!(Mat4::IDENTITY.transpose(), Mat4::IDENTITY);
        #[rustfmt::skip]
        let x = Mat4([
            1.0,  2.0,  3.0,  4.0,
            5.0,  6.0,  7.0,  8.0,
            9.0,  10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, 16.0,
        ]);
        #[rustfmt::skip]
        let y = Mat4([
            1.0, 5.0,  9.0, 13.0,
            2.0, 6.0, 10.0, 14.0,
            3.0, 7.0, 11.0, 15.0,
            4.0, 8.0, 12.0, 16.0,
        ]);
        assert_eq!(x.transpose(), y);
        assert_eq!(x.transpose().transpose(), x);
    }

    #[test]
    fn mat4_multiply() {
        assert_eq!(Mat4::IDENTITY.mul_mat4_base(Mat4::IDENTITY), Mat4::IDENTITY);

        if std::is_x86_feature_detected!("sse2") {
            assert_eq!(
                unsafe { Mat4::IDENTITY.mul_mat4_sse2(Mat4::IDENTITY) },
                Mat4::IDENTITY
            );
        }

        if std::is_x86_feature_detected!("avx2") {
            assert_eq!(
                unsafe { Mat4::IDENTITY.mul_mat4_avx2(Mat4::IDENTITY) },
                Mat4::IDENTITY
            );
        }

        assert_eq!(Mat4::IDENTITY * Mat4::IDENTITY, Mat4::IDENTITY);

        let scale = Mat4::from_scale(Vec3::splat(2.0));
        assert_eq!(scale * Mat4::IDENTITY, scale);
    }

    #[test]
    fn mat4_mul_vec4() {
        assert_eq!(Mat4::IDENTITY * Vec4::ZERO, Vec4::ZERO);

        assert_eq!(
            Mat4::IDENTITY.mul_vec4_base(Vec4::new(1.0, 2.0, 3.0, 4.0)),
            Vec4::new(1.0, 2.0, 3.0, 4.0)
        );

        if std::is_x86_feature_detected!("sse4.1") {
            assert_eq!(
                unsafe { Mat4::IDENTITY.mul_vec4_sse41(Vec4::new(1.0, 2.0, 3.0, 4.0)) },
                Vec4::new(1.0, 2.0, 3.0, 4.0)
            );
        }

        let scale = Mat4::from_scale(Vec3::splat(2.0));
        assert_eq!(scale.mul_vec3(Vec3::splat(1.0)), Vec3::splat(2.0));
    }
}
