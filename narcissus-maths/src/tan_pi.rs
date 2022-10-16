// Based on Norbert Juffa's tanpi posted to the cuda forums. Using my own polynomial, but that might
// be worse, todo: check whether polynomial is worse.
// https://forums.developer.nvidia.com/t/an-implementation-of-single-precision-tanpi-for-cuda/48024
//
// Sollya code for generating these polynomials is in `doc/sincostan.sollya`

const F32_TAN_PI_15_K: [f32; 7] = unsafe {
    std::mem::transmute::<[u32; 7], _>([
        0x41255def, // 0x1.4abbdep3
        0x4223335b, // 0x1.4666b6p5
        0x43234f4b, // 0x1.469e96p7
        0x441e4604, // 0x1.3c8c08p9
        0x4548fad9, // 0x1.91f5b2p11
        0xc21a6851, // -0x1.34d0a2p5
        0x47f775a0, // 0x1.eeeb4p16
    ])
};

/// Computes the tangent of `a` expressed in multiples of *pi* radians, or half-turns.
///
/// Returns `tan(a * pi)`
///
/// Error <= 1.60536 ulp.
///
/// # Examples
///
/// ```
/// use narcissus_maths::tan_pi_f32;
/// ```
pub fn tan_pi_f32(a: f32) -> f32 {
    const T: [f32; 7] = F32_TAN_PI_15_K;

    // Range reduction.
    let r = (a + a).round();
    let i: u32 = unsafe { r.to_int_unchecked() };
    let r = r.mul_add(-0.5, a);

    let e = if i.wrapping_add(1) & 2 != 0 {
        -0.0
    } else {
        0.0
    };

    // Core approximation.
    let r2 = r * r;
    let p = T[6];
    let p = p.mul_add(r2, T[5]);
    let p = p.mul_add(r2, T[4]);
    let p = p.mul_add(r2, T[3]);
    let p = p.mul_add(r2, T[2]);
    let p = p.mul_add(r2, T[1]);
    let p = p.mul_add(r2, T[0]);

    let t = r2 * r;
    let t = p.mul_add(t, -8.742278e-8 * r);
    let r = r.mul_add(std::f32::consts::PI, t);

    // Handle half-integer arguments.
    let r = if r == 0.0 { e } else { r };
    let r = if i & 1 == 1 { 1.0 / -r } else { r };

    // Handle integer arguments.
    if a == a.floor() {
        a * e
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use super::tan_pi_f32;

    #[test]
    fn basics() {
        assert_eq!(tan_pi_f32(0.0), 0.0);
        assert_eq!(tan_pi_f32(0.25), 1.0);
    }
}
