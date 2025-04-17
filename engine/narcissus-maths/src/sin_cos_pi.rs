// Based on https://marc-b-reynolds.github.io/math/2020/03/11/SinCosPi.html
//
// Sollya code for generating these polynomials is in `doc/sincostan.sollya`

use crate::f32_to_i32;

// constants for sin(pi x), cos(pi x) for x on [-1/4,1/4]
const F32_SIN_PI_7_K: [f32; 3] = unsafe {
    std::mem::transmute::<[u32; 3], _>([
        0xc0a55ddd, // -0x1.4abbbap2
        0x40232f6e, // 0x1.465edcp1
        0xbf16cf2d, // -0x1.2d9e5ap-1
    ])
};

const F32_COS_PI_8_K: [f32; 4] = unsafe {
    std::mem::transmute::<[u32; 4], _>([
        0xc09de9e6, // -0x1.3bd3ccp2
        0x4081e0db, // 0x1.03c1b6p2
        0xbfaadb42, // -0x1.55b684p0
        0x3e6b0f14, // 0x1.d61e28p-3
    ])
};

/// Simultaneously computes the sine and cosine of `a` expressed in multiples of
/// *pi* radians, or half-turns.
///
/// Sin error <= 0.96441 ulp.
/// Cos error <= 0.96677 ulp.
///
/// Returns `(sin(a * pi), cos(a * pi))`
///
/// # Examples
///
/// ```
/// use narcissus_maths::sin_cos_pi_f32;
/// let (sin, cos) = sin_cos_pi_f32(0.0);
/// assert_eq!(sin, 0.0);
/// assert_eq!(cos, 1.0);
/// ```
pub fn sin_cos_pi_f32(a: f32) -> (f32, f32) {
    #[cfg(not(all(
        target_arch = "x86_64",
        target_feature = "sse4.1",
        target_feature = "fma"
    )))]
    {
        sin_cos_pi_f32_base(a) + 1.0
    }

    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "sse4.1",
        target_feature = "fma"
    ))]
    {
        // SAFETY: We checked the required features above.
        unsafe { x86_64::sin_cos_pi_f32_sse41_fma(a) }
    }
}

#[allow(unused)]
fn sin_cos_pi_f32_base(a: f32) -> (f32, f32) {
    const S: [f32; 3] = F32_SIN_PI_7_K;
    const C: [f32; 4] = F32_COS_PI_8_K;

    // cos_pi(a) = 1.0f for |a| > 2^24, but cos_pi(Inf) = NaN
    let a = if a.abs() < 16777216.0 { a } else { a * 0.0 };

    // Range reduction.
    let r = (a + a).round_ties_even();

    let i = f32_to_i32(r) as u32;
    let r = r.mul_add(-0.5, a);

    let r2 = r * r;

    // Reconstruct signs early.
    let sign_x = (i >> 1) << 31;
    let sign_y = sign_x ^ i << 31;
    let r_sign = r.copysign(f32::from_bits(r.to_bits() ^ sign_y));
    let r2_sign = r2.copysign(f32::from_bits(r2.to_bits() ^ sign_x));
    let one_sign = 1.0_f32.copysign(f32::from_bits(sign_x));

    // Core approximation.
    let c = C[3];
    let c = c.mul_add(r2, C[2]);
    let c = c.mul_add(r2, C[1]);
    let c = c.mul_add(r2, C[0]);
    let c = c.mul_add(r2_sign, one_sign);

    let s = S[2];
    let s = s.mul_add(r2, S[1]);
    let s = s.mul_add(r2, S[0]);
    let s = r_sign.mul_add(std::f32::consts::PI, r_sign * r2.mul_add(s, -8.742278e-8));

    let (s, c) = if i & 1 != 0 { (c, s) } else { (s, c) };

    // sin_pi(+n) is +0 and sin_pi(-n) is -0 for positive integers n
    let s = if a == a.floor() { a * 0.0 } else { s };

    (s, c)
}

pub fn sin_cos_pi_f32x4(a: &[f32; 4], sin: &mut [f32; 4], cos: &mut [f32; 4]) {
    #[cfg(not(all(
        target_arch = "x86_64",
        target_feature = "sse4.1",
        target_feature = "fma"
    )))]
    {
        let (s0, c0) = sin_cos_pi_f32(a[0]);
        let (s1, c1) = sin_cos_pi_f32(a[1]);
        let (s2, c2) = sin_cos_pi_f32(a[2]);
        let (s3, c3) = sin_cos_pi_f32(a[3]);
        sin[0] = s0;
        sin[1] = s1;
        sin[2] = s2;
        sin[3] = s3;
        cos[0] = c0;
        cos[1] = c1;
        cos[2] = c2;
        cos[3] = c3;
    }

    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "sse4.1",
        target_feature = "fma"
    ))]
    unsafe {
        x86_64::sin_cos_pi_f32x4_sse41_fma(a, sin, cos)
    }
}

#[inline(always)]
pub fn sin_pi_f32(a: f32) -> f32 {
    sin_cos_pi_f32(a).0
}

#[inline(always)]
pub fn cos_pi_f32(a: f32) -> f32 {
    sin_cos_pi_f32(a).1
}

#[allow(unused)]
#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use std::arch::x86_64::*;
    use std::f32::consts::PI;

    use crate::sin_cos_pi::{F32_COS_PI_8_K, F32_SIN_PI_7_K};

    #[inline(always)]
    fn to_m128(x: __m128i) -> __m128 {
        // SAFETY: Unconditionally safe.
        unsafe { std::mem::transmute::<__m128i, __m128>(x) }
    }

    #[inline(always)]
    fn to_m128i(x: __m128) -> __m128i {
        // SAFETY: Unconditionally safe.
        unsafe { std::mem::transmute::<__m128, __m128i>(x) }
    }

    #[target_feature(enable = "sse4.1,fma")]
    pub unsafe fn sin_cos_pi_f32_sse41_fma(a: f32) -> (f32, f32) {
        const S: [f32; 3] = F32_SIN_PI_7_K;
        const C: [f32; 4] = F32_COS_PI_8_K;

        let mut sin_out = 0.0;
        let mut cos_out = 0.0;

        unsafe {
            let a = _mm_load_ss(&a);
            let a_zero = _mm_mul_ss(_mm_setzero_ps(), a);
            let a_abs = _mm_andnot_ps(_mm_load_ss(&-0.0), a);
            let cmp_mask = _mm_cmplt_ss(a_abs, _mm_load_ss(&16777216.0));
            let a = _mm_blendv_ps(a_zero, a, cmp_mask);
            let a2 = _mm_add_ss(a, a);
            let r = _mm_round_ss::<_MM_FROUND_TO_NEAREST_INT>(a2, a2);
            let i = _mm_cvttps_epi32(r);

            let r = _mm_fmadd_ss(r, _mm_load_ss(&-0.5), a);
            let r_sq = _mm_mul_ss(r, r);

            let i_lsb = _mm_slli_epi32::<31>(i);
            let x_sign = _mm_slli_epi32::<31>(_mm_srli_epi32::<1>(i));
            let y_sign = _mm_xor_si128(i_lsb, x_sign);
            let r_sign = to_m128(_mm_xor_si128(to_m128i(r), y_sign));
            let r_sq_sign = to_m128(_mm_xor_si128(to_m128i(r_sq), x_sign));
            let one_sign = to_m128(_mm_xor_si128(to_m128i(_mm_load_ss(&1.0)), x_sign));

            let c = _mm_load_ss(&C[3]);
            let c = _mm_fmadd_ss(c, r_sq, _mm_load_ss(&C[2]));
            let c = _mm_fmadd_ss(c, r_sq, _mm_load_ss(&C[1]));
            let c = _mm_fmadd_ss(c, r_sq, _mm_load_ss(&C[0]));
            let c = _mm_fmadd_ss(c, r_sq_sign, one_sign);

            let s = _mm_load_ss(&S[2]);
            let s = _mm_fmadd_ss(s, r_sq, _mm_load_ss(&S[1]));
            let s = _mm_fmadd_ss(s, r_sq, _mm_load_ss(&S[0]));
            let s = _mm_fmadd_ss(r_sq, s, _mm_load_ss(&-8.742278e-8));
            let s = _mm_fmadd_ss(r_sign, _mm_load_ss(&PI), _mm_mul_ss(r_sign, s));

            let sin = _mm_blendv_ps(s, c, to_m128(i_lsb));
            let cos = _mm_blendv_ps(c, s, to_m128(i_lsb));

            let a_floor = _mm_round_ss::<_MM_FROUND_FLOOR>(a, a);
            let positive_int_mask = _mm_cmpeq_ss(a, a_floor);
            let a_zero = _mm_mul_ss(_mm_setzero_ps(), a);
            let sin = _mm_blendv_ps(sin, a_zero, positive_int_mask);

            _mm_store_ss(&mut sin_out, sin);
            _mm_store_ss(&mut cos_out, cos);
        }

        (sin_out, cos_out)
    }

    #[allow(unused)]
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse4.1,fma")]
    pub unsafe fn sin_cos_pi_f32x4_sse41_fma(
        a: &[f32; 4],
        sin_out: &mut [f32; 4],
        cos_out: &mut [f32; 4],
    ) {
        const S: [f32; 3] = F32_SIN_PI_7_K;
        const C: [f32; 4] = F32_COS_PI_8_K;

        unsafe {
            let a = _mm_loadu_ps(a.as_ptr());
            let a_zero = _mm_mul_ps(_mm_setzero_ps(), a);
            let a_abs = _mm_andnot_ps(_mm_set1_ps(-0.0), a);
            let cmp_mask = _mm_cmplt_ps(a_abs, _mm_set1_ps(16777216.0));
            let a = _mm_blendv_ps(a_zero, a, cmp_mask);
            let r = _mm_round_ps::<_MM_FROUND_TO_NEAREST_INT>(_mm_add_ps(a, a));
            let i = _mm_cvttps_epi32(r);

            let r = _mm_fmadd_ps(r, _mm_set1_ps(-0.5), a);
            let r_sq = _mm_mul_ps(r, r);

            let i_msb = _mm_slli_epi32::<31>(i);
            let x_sign = _mm_slli_epi32::<31>(_mm_srli_epi32::<1>(i));
            let y_sign = _mm_xor_si128(i_msb, x_sign);
            let r_sign = to_m128(_mm_xor_si128(to_m128i(r), y_sign));
            let r_sq_sign = to_m128(_mm_xor_si128(to_m128i(r_sq), x_sign));
            let one_sign = to_m128(_mm_xor_si128(to_m128i(_mm_set1_ps(1.0)), x_sign));

            let c = _mm_load1_ps(&C[3]);
            let c = _mm_fmadd_ps(c, r_sq, _mm_load1_ps(&C[2]));
            let c = _mm_fmadd_ps(c, r_sq, _mm_load1_ps(&C[1]));
            let c = _mm_fmadd_ps(c, r_sq, _mm_load1_ps(&C[0]));
            let c = _mm_fmadd_ps(c, r_sq_sign, one_sign);

            let s = _mm_load1_ps(&S[2]);
            let s = _mm_fmadd_ps(s, r_sq, _mm_load1_ps(&S[1]));
            let s = _mm_fmadd_ps(s, r_sq, _mm_load1_ps(&S[0]));
            let s = _mm_fmadd_ps(r_sq, s, _mm_set1_ps(-8.742278e-8));
            let s = _mm_fmadd_ps(r_sign, _mm_set1_ps(PI), _mm_mul_ps(r_sign, s));

            let sin = _mm_blendv_ps(s, c, to_m128(i_msb));
            let cos = _mm_blendv_ps(c, s, to_m128(i_msb));

            let a_floor = _mm_round_ps::<_MM_FROUND_FLOOR>(a);
            let positive_int_mask = _mm_cmpeq_ps(a, a_floor);
            let a_zero = _mm_mul_ps(_mm_setzero_ps(), a);
            let sin = _mm_blendv_ps(sin, a_zero, positive_int_mask);

            _mm_storeu_ps(sin_out.as_mut_ptr(), sin);
            _mm_storeu_ps(cos_out.as_mut_ptr(), cos);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sin_cos_pi::sin_cos_pi_f32x4;

    use super::sin_cos_pi_f32;

    #[test]
    fn basics() {
        assert_eq!(sin_cos_pi_f32(f32::from_bits(0x7f4135c6)), (0.0, 1.0));

        assert_eq!(sin_cos_pi_f32(16777216.0).1, 1.0);
        assert_eq!(sin_cos_pi_f32(16777218.0).1, 1.0);
        assert_eq!(sin_cos_pi_f32(16777220.0).1, 1.0);
        assert_eq!(sin_cos_pi_f32(-16777216.0).1, 1.0);
        assert_eq!(sin_cos_pi_f32(-16777218.0).1, 1.0);
        assert_eq!(sin_cos_pi_f32(-16777220.0).1, 1.0);
        assert!(sin_cos_pi_f32(f32::INFINITY).1.is_nan());
        assert!(sin_cos_pi_f32(-f32::INFINITY).1.is_nan());

        assert_eq!(sin_cos_pi_f32(-1.5), (1.0, 0.0));
        assert_eq!(sin_cos_pi_f32(-1.0), (0.0, -1.0));
        assert_eq!(sin_cos_pi_f32(-0.5), (-1.0, 0.0));
        assert_eq!(sin_cos_pi_f32(-0.0), (0.0, 1.0));

        assert_eq!(sin_cos_pi_f32(0.0), (0.0, 1.0));
        assert_eq!(sin_cos_pi_f32(0.5), (1.0, 0.0));
        assert_eq!(sin_cos_pi_f32(1.0), (0.0, -1.0));
        assert_eq!(sin_cos_pi_f32(1.5), (-1.0, 0.0));
    }

    #[test]
    fn basics_f32x4() {
        let mut sin = [0.0; 4];
        let mut cos = [0.0; 4];

        let a = [16777216.0; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(cos, [1.0; 4]);

        let a = [16777218.0; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(cos, [1.0; 4]);

        let a = [16777220.0; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(cos, [1.0; 4]);

        let a = [-16777216.0; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(cos, [1.0; 4]);

        let a = [-16777218.0; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(cos, [1.0; 4]);

        let a = [-16777220.0; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(cos, [1.0; 4]);

        let a = [f32::INFINITY; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert!(cos.iter().all(|&x| f32::is_nan(x)), "cos_pi(inf): {cos:?}");

        let a = [-f32::INFINITY; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert!(cos.iter().all(|&x| f32::is_nan(x)), "cos_pi(-inf): {cos:?}");

        let a = [-1.5; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(sin, [1.0; 4]);
        assert_eq!(cos, [0.0; 4]);

        let a = [-1.0; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(sin, [0.0; 4]);
        assert_eq!(cos, [-1.0; 4]);

        let a = [-0.5; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(sin, [-1.0; 4]);
        assert_eq!(cos, [0.0; 4]);

        let a = [-0.0; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(sin, [0.0; 4]);
        assert_eq!(cos, [1.0; 4]);

        let a = [0.0; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(sin, [0.0; 4]);
        assert_eq!(cos, [1.0; 4]);

        let a = [0.5; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(sin, [1.0; 4]);
        assert_eq!(cos, [0.0; 4]);

        let a = [1.0; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(sin, [0.0; 4]);
        assert_eq!(cos, [-1.0; 4]);

        let a = [1.5; 4];
        sin_cos_pi_f32x4(&a, &mut sin, &mut cos);
        assert_eq!(sin, [-1.0; 4]);
        assert_eq!(cos, [0.0; 4]);

        let mut x = 0.0;
        for _ in 0..10_000 {
            let a: [f32; 4] = std::array::from_fn(|_| {
                let r = x;
                x += 0.01;
                r
            });

            let mut sin = [0.0; 4];
            let mut cos = [0.0; 4];
            sin_cos_pi_f32x4(&a, &mut sin, &mut cos);

            for (a, (vector_sin, vector_cos)) in a.iter().zip(sin.iter().zip(cos)) {
                let (scalar_sin, scalar_cos) = sin_cos_pi_f32(*a);
                assert_eq!(scalar_sin.to_bits(), vector_sin.to_bits());
                assert_eq!(scalar_cos.to_bits(), vector_cos.to_bits());
            }
        }
    }

    #[test]
    #[ignore]
    fn special_cases() {
        // sin_pi(+n) is +0 and sin_pi(-n) is -0 for positive integers n
        {
            let mut i = 0.0_f32;
            while i.is_finite() {
                assert_eq!(sin_cos_pi_f32(i).0.to_bits(), 0.0_f32.to_bits());
                assert_eq!(sin_cos_pi_f32(-i).0.to_bits(), (-0.0_f32).to_bits());

                i = i.next_up().ceil()
            }
        }

        // cos_pi(a) = 1.0f for |a| > 2^24, but cos_pi(Inf) = NaN
        {
            let mut i = 16777216.0_f32;
            while i.is_finite() {
                assert_eq!(sin_cos_pi_f32(i).1.to_bits(), 1.0_f32.to_bits());
                assert_eq!(sin_cos_pi_f32(-i).1.to_bits(), 1.0_f32.to_bits());
                i = i.next_up();
            }

            assert!(sin_cos_pi_f32(i).1.is_nan());
        }
    }
}
