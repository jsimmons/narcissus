// Norbert Juffa's exp for CUDA, incompletely translated.
// https://forums.developer.nvidia.com/t/a-more-accurate-performance-competitive-implementation-of-expf/47528

use crate::f32_to_i32;

pub fn exp_f32(a: f32) -> f32 {
    // exp(a) = 2**i * exp(f); i = rintf (a / log(2))
    let j = a.mul_add(std::f32::consts::LOG2_E, 12582912.0) - 12582912.0; // 0x1.715476p0, 0x1.8p23
    let f = j.mul_add(-6.931_457_5e-1, a); // -0x1.62e400p-1  // log_2_hi
    let f = j.mul_add(-1.428_606_8e-6, f); // -0x1.7f7d1cp-20 // log_2_lo

    let i = f32_to_i32(j);

    // approximate r = exp(f) on interval [-log(2)/2, +log(2)/2]
    let r = 1.378_059_4e-3_f32; // 0x1.694000p-10
    let r = r.mul_add(f, 8.373_124_5e-3); // 0x1.125edcp-7
    let r = r.mul_add(f, 4.166_953_6e-2); // 0x1.555b5ap-5
    let r = r.mul_add(f, 1.666_647_2e-1); // 0x1.555450p-3
    let r = r.mul_add(f, 4.999_998_5e-1); // 0x1.fffff6p-2
    let r = r.mul_add(f, 1.0);
    let r = r.mul_add(f, 1.0);

    // exp(a) = 2**i * r
    let ia = if i > 0 { 0_u32 } else { 0x83000000 };
    let s = f32::from_bits(0x7f000000_u32.wrapping_add(ia));
    let t = f32::from_bits(((i as u32) << 23).wrapping_sub(ia) as u32);
    let r = r * s;
    let r = r * t;

    // handle special cases: severe overflow / underflow
    if a.abs() >= 104.0 {
        if a > 0.0 { f32::INFINITY } else { 0.0 }
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use crate::exp_f32;

    #[test]
    fn basics() {
        assert_eq!(
            exp_f32(255544082189650076565756907338896244736.0),
            f32::INFINITY
        );
    }
}
