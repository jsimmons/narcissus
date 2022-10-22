// Based on https://marc-b-reynolds.github.io/math/2020/03/11/SinCosPi.html
//
// Sollya code for generating these polynomials is in `doc/sincostan.sollya`

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
        0x4081e0dc, // 0x1.03c1b8p2
        0xbfaadbe7, // -0x1.55b7cep0
        0x3e6b4255, // 0x1.d684aap-3
    ])
};

#[inline(always)]
fn mulsign_f32(x: f32, s: u32) -> f32 {
    f32::from_bits(x.to_bits() ^ s)
}

/// Simultaneously computes the sine and cosine of `a` expressed in multiples of *pi* radians, or half-turns.
///
/// Sin error <= 0.96563 ulp.
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
    const S: [f32; 3] = F32_SIN_PI_7_K;
    const C: [f32; 4] = F32_COS_PI_8_K;

    // cos_pi(a) = 1.0f for |a| > 2^24, but cos_pi(Inf) = NaN
    let a = if a.abs() < 16777216.0 { a } else { a * 0.0 };

    // Range reduction.
    let r = (a + a).round();
    let i = unsafe { r.to_int_unchecked::<i32>() } as u32;
    let r = r.mul_add(-0.5, a);

    let sx = (i >> 1) << 31;
    let sy = (i << 31) ^ sx;

    // Core approximation.
    let r2 = r * r;
    let r = mulsign_f32(r, sy);

    let c = C[3];
    let c = c.mul_add(r2, C[2]);
    let c = c.mul_add(r2, C[1]);
    let c = c.mul_add(r2, C[0]);
    let c = c.mul_add(r2, 1.0);
    let c = mulsign_f32(c, sx);

    let s = S[2];
    let s = s.mul_add(r2, S[1]);
    let s = s.mul_add(r2, S[0]);
    let s = r.mul_add(std::f32::consts::PI, r * r2.mul_add(s, -8.742278e-8));

    let (s, c) = if i & 1 != 0 { (c, s) } else { (s, c) };

    // IEEE-754: sin_pi(+n) is +0 and sin_pi(-n) is -0 for positive integers n
    let s = if a == a.floor() { a * 0.0 } else { s };

    (s, c)
}

#[cfg(test)]
mod tests {
    use crate::sin_cos_pi_f32;

    #[test]
    fn basics() {
        assert_eq!(sin_cos_pi_f32(f32::from_bits(0x7f4135c6)), (0.0, 1.0));

        assert_eq!(sin_cos_pi_f32(-1.5), (1.0, 0.0));
        assert_eq!(sin_cos_pi_f32(-1.0), (0.0, -1.0));
        assert_eq!(sin_cos_pi_f32(-0.5), (-1.0, 0.0));
        assert_eq!(sin_cos_pi_f32(-0.0), (0.0, 1.0));

        assert_eq!(sin_cos_pi_f32(0.0), (0.0, 1.0));
        assert_eq!(sin_cos_pi_f32(0.5), (1.0, 0.0));
        assert_eq!(sin_cos_pi_f32(1.0), (0.0, -1.0));
        assert_eq!(sin_cos_pi_f32(1.5), (-1.0, 0.0));
    }
}
