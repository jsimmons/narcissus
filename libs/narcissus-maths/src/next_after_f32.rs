/// Calculate the next representable floating-point value following x in the direction of y.
///
/// If y is less than x, these functions will return the largest representable number less than x.
///
/// # Returns
///
/// On success, the function returns the next representable floating-point value after x in the
/// direction of y.
///  
/// * If `x` equals `y`, then `y` is returned.
/// * If `x` or `y` is a `NaN`, a `NaN` is returned.
/// * If `x` is finite, and the result would overflow, a range error occurs, and the function
///   returns `inf` with the correct mathematical sign.
/// * If `x` is not equal to `y`, and the correct function result would be subnormal, zero, or
///   underflow, a range error occurs, and either the correct value (if it can be represented),
///   or `0.0`, is returned.
/// * If x equals y, the function returns y.
pub fn next_after_f32(x: f32, y: f32) -> f32 {
    if x.is_nan() || y.is_nan() {
        return x + y;
    }

    let ux = x.to_bits();
    let uy = y.to_bits();

    if ux == uy {
        return y;
    }

    let ax = ux & 0x7fff_ffff;
    let ay = uy & 0x7fff_ffff;

    let ux = if ax == 0 {
        if ay == 0 {
            return y;
        }
        (uy & 0x8000_0000) | 1
    } else if ax > ay || ((ux ^ uy) & 0x8000_0000) != 0 {
        ux - 1
    } else {
        ux + 1
    };

    let e = ux & 0x7f800000;
    // Overflow if ux is infinite, and x is finite.
    if e == 0x7f800000 {
        return x + x;
    }
    // Force underflow if ux is subnormal or zero.
    if e == 0 {
        let mut force_eval = 0.0;
        let val = f32::from_bits(ux);
        unsafe { std::ptr::write_volatile(&mut force_eval, x * x + val * val) };
    }

    f32::from_bits(ux)
}
