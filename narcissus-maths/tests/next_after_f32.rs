use narcissus_maths::next_after_f32;

mod libc {
    use std::ffi::c_float;
    extern "C" {
        pub fn nextafterf(x: c_float, y: c_float) -> c_float;
    }
}

#[inline(always)]
fn nextafterf(x: f32, y: f32) -> f32 {
    unsafe { libc::nextafterf(x, y) }
}

fn test_towards(y: f32) {
    for u in 0..=0xffff_ffff_u32 {
        let x = f32::from_bits(u);
        let ours = next_after_f32(x, y);
        let reference = nextafterf(x, y);
        assert!(
            ours == reference || (ours.is_nan() && reference.is_nan()),
            "x ({u:X}): ours ({ours}) != reference ({reference})"
        );
    }
}

#[test]
#[ignore]
fn next_after_f32_to_zero() {
    test_towards(0.0)
}

#[test]
#[ignore]
fn next_after_f32_to_inf() {
    test_towards(f32::INFINITY)
}

#[test]
#[ignore]
fn next_after_f32_to_neg_inf() {
    test_towards(f32::NEG_INFINITY)
}
