/// Rough translation of Paul Zimmermann's binary32_exhaustive to Rust.
/// https://gitlab.inria.fr/zimmerma/math_accuracy/-/blob/master/binary32_exhaustive/check_exhaustive.c
use std::{
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

use narcissus_maths::{exp_f32, next_after_f32, sin_cos_pi_f32, tan_pi_f32};

use gmp_mpfr_sys::mpfr;

// Less than 280 cause precision errors in the 2 * pi calculation, introducing extra failures.
const PREC: u32 = 280;
const COUNT: u32 = 2_139_095_040_u32;

pub enum Round {
    /// Round to nearest
    TiesToEven, // MPFR_RNDN,
    /// Round toward minus infinity
    TowardNegative, // MPFR_RNDD
    /// Round toward plus infinity
    TowardPositive, // MPFR_RNDU
    /// Round toward zero
    TowardZero, // MPFR_RNDZ
    /// Away from zero.
    AwayZero, // MPFR_RNDA
}

impl Round {
    fn to_raw(self) -> mpfr::rnd_t {
        match self {
            Round::TiesToEven => mpfr::rnd_t::RNDN,
            Round::TowardNegative => mpfr::rnd_t::RNDD,
            Round::TowardPositive => mpfr::rnd_t::RNDU,
            Round::TowardZero => mpfr::rnd_t::RNDZ,
            Round::AwayZero => mpfr::rnd_t::RNDA,
        }
    }
}

pub struct Float(mpfr::mpfr_t);

impl Float {
    pub fn new(precision: u32) -> Self {
        unsafe {
            let mut x = MaybeUninit::uninit();
            mpfr::init2(x.as_mut_ptr(), precision as i64);
            Self(x.assume_init())
        }
    }

    pub fn with_value_f32(precision: u32, value: f32, round: Round) -> Self {
        unsafe {
            let mut x = MaybeUninit::uninit();
            mpfr::init2(x.as_mut_ptr(), precision as i64);
            mpfr::set_flt(x.as_mut_ptr(), value, round.to_raw());
            Self(x.assume_init())
        }
    }

    pub fn to_f32(&self, round: Round) -> f32 {
        unsafe { mpfr::get_flt(&self.0, round.to_raw()) }
    }

    pub fn to_f64(&self, round: Round) -> f64 {
        unsafe { mpfr::get_d(&self.0, round.to_raw()) }
    }

    pub fn exp(&self) -> i64 {
        unsafe { mpfr::get_exp(&self.0) }
    }

    pub fn set_const_pi(&mut self, round: Round) {
        let _ = unsafe { mpfr::const_pi(&mut self.0, round.to_raw()) };
    }

    pub fn set_2_exp_u64(&mut self, x: u64, e: i64, round: Round) {
        let _ = unsafe { mpfr::set_ui_2exp(&mut self.0, x, e, round.to_raw()) };
    }

    pub fn set_f32(&mut self, x: f32, round: Round) {
        let _ = unsafe { mpfr::set_flt(&mut self.0, x, round.to_raw()) };
    }

    pub fn set_precision(&mut self, precision: u32, round: Round) {
        unsafe { mpfr::prec_round(&mut self.0, precision as i64, round.to_raw()) };
    }

    pub fn add_assign(&mut self, rhs: &Float, round: Round) {
        let _ = unsafe { mpfr::add(&mut self.0, &self.0, &rhs.0, round.to_raw()) };
    }

    pub fn sub_assign(&mut self, rhs: &Float, round: Round) {
        let _ = unsafe { mpfr::sub(&mut self.0, &self.0, &rhs.0, round.to_raw()) };
    }

    pub fn mul_assign(&mut self, rhs: &Float, round: Round) {
        let _ = unsafe { mpfr::mul(&mut self.0, &self.0, &rhs.0, round.to_raw()) };
    }

    pub fn div_assign(&mut self, rhs: &Float, round: Round) {
        let _ = unsafe { mpfr::div(&mut self.0, &self.0, &rhs.0, round.to_raw()) };
    }

    /// multiplies self by 2 raised to `rhs`.
    pub fn mul_2_assign_i64(&mut self, rhs: i64, round: Round) {
        let _ = unsafe { mpfr::mul_2si(&mut self.0, &self.0, rhs, round.to_raw()) };
    }

    /// divides self by 2 raised to `rhs`.
    pub fn div_2_assign_u64(&mut self, rhs: u64, round: Round) {
        let _ = unsafe { mpfr::div_2ui(&mut self.0, &self.0, rhs, round.to_raw()) };
    }

    pub fn sin_mut(&mut self, round: Round) -> i32 {
        unsafe { mpfr::sin(&mut self.0, &self.0, round.to_raw()) }
    }

    pub fn cos_mut(&mut self, round: Round) -> i32 {
        unsafe { mpfr::cos(&mut self.0, &self.0, round.to_raw()) }
    }

    pub fn tan_mut(&mut self, round: Round) -> i32 {
        unsafe { mpfr::tan(&mut self.0, &self.0, round.to_raw()) }
    }

    pub fn exp_mut(&mut self, round: Round) -> i32 {
        unsafe { mpfr::exp(&mut self.0, &self.0, round.to_raw()) }
    }

    pub fn abs_mut(&mut self, round: Round) -> i32 {
        unsafe { mpfr::abs(&mut self.0, &self.0, round.to_raw()) }
    }

    pub fn subnormalize(&mut self, t: i32, round: Round) {
        let _ = unsafe { mpfr::subnormalize(&mut self.0, t, round.to_raw()) };
    }
}

impl Drop for Float {
    fn drop(&mut self) {
        unsafe { mpfr::clear(&mut self.0) }
    }
}

#[derive(Clone, Copy, Debug)]
struct TestRange {
    base: u32,
    end: u32,
}

#[derive(Clone, Copy, Default)]
struct FloatErrors {
    num_errors: u32,
    num_errors_2: u32,
    max_error_f64: f64,
    max_error_ulp: u32,
    max_error_val: u32,
}

impl std::fmt::Debug for FloatErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let errors = format!(
            "{} ({:.2}%)",
            self.num_errors,
            (self.num_errors as f64 / (COUNT * 2) as f64) * 100.0
        );
        let errors_2 = format!(
            "{} ({:.2}%)",
            self.num_errors_2,
            (self.num_errors_2 as f64 / (COUNT * 2) as f64) * 100.0
        );

        f.debug_struct("FloatErrors")
            .field("num_errors", &errors)
            .field("num_errors_2", &errors_2)
            .field("max_error_f64", &self.max_error_f64)
            .field("max_error_ulp", &self.max_error_ulp)
            .field("max_error_val", &self.max_error_val)
            .finish()
    }
}

struct FloatCheck {
    ref_fn: fn(&mut Float, &Float) -> i32,
    our_fn: fn(f32) -> f32,

    tan_mode: bool,

    pi: Float,
    tmp: Float,

    errors: FloatErrors,
}

impl FloatCheck {
    fn new(ref_fn: fn(&mut Float, &Float) -> i32, our_fn: fn(f32) -> f32, tan_mode: bool) -> Self {
        unsafe {
            mpfr::set_emin(-148);
            mpfr::set_emax(128);
        }

        let mut pi = Float::new(PREC);
        pi.set_const_pi(Round::TiesToEven);

        Self {
            ref_fn,
            our_fn,
            tan_mode,
            pi,
            tmp: Float::new(PREC),
            errors: Default::default(),
        }
    }

    fn ulp_error_f64(&mut self, our_value: f32, x: f32) -> f64 {
        let (emin, emax) = unsafe { (mpfr::get_emin(), mpfr::get_emax()) };
        unsafe {
            mpfr::set_emin(mpfr::get_emin_min());
            mpfr::set_emax(mpfr::get_emax_max());
        }

        let mut y = Float::new(24);
        if our_value.is_infinite() {
            y.set_2_exp_u64(1, 128, Round::TowardNegative)
        } else {
            y.set_f32(our_value, Round::TowardNegative)
        }
        self.tmp.set_f32(x, Round::TowardNegative);
        (self.ref_fn)(&mut self.tmp, &self.pi);
        let e = self.tmp.exp();
        self.tmp.sub_assign(&y, Round::AwayZero);
        self.tmp.abs_mut(Round::TowardNegative);
        y.set_2_exp_u64(1, e - PREC as i64 - 1, Round::TowardNegative);
        self.tmp.add_assign(&y, Round::AwayZero);
        let e = if e - 24 < -149 { -149 } else { e - 24 };
        self.tmp.mul_2_assign_i64(-e, Round::TowardNegative);
        let err = self.tmp.to_f64(Round::AwayZero);

        unsafe {
            mpfr::set_emin(emin);
            mpfr::set_emax(emax);
        }

        err
    }

    fn ulp_error(&mut self, our_value: f32, ref_value: f32, x: f32) -> u32 {
        if our_value == ref_value {
            return 0;
        }

        let mut our_value = our_value;
        let mut ref_value = ref_value;

        if ref_value.is_infinite() {
            if our_value.is_infinite() {
                // Then ours and reference have different signs.
                assert!(our_value * ref_value < 0.0);
                return u32::MAX;
            }

            our_value /= 2.0;
            self.tmp.set_f32(x, Round::TowardNegative);
            (self.ref_fn)(&mut self.tmp, &self.pi);
            self.tmp.div_2_assign_u64(1, Round::TowardNegative);
            ref_value = self.tmp.to_f32(Round::TowardNegative);

            if ref_value.is_infinite() {
                ref_value = if ref_value > 0.0 {
                    f32::from_bits(0x7f00_0000)
                } else {
                    f32::from_bits(0xff00_0000)
                };
            }
        }

        if our_value.is_infinite() {
            assert!(!ref_value.is_infinite());
            // If ours gives +/-Inf but the correct rounding is in the binary32 range, assume we gave
            // +/-2^128.
            ref_value /= 2.0;
            our_value = if our_value > 0.0 {
                f32::from_bits(0x7f00_0000)
            } else {
                f32::from_bits(0xff00_0000)
            }
        }

        let err = our_value - ref_value;
        let ulp = next_after_f32(ref_value, our_value) - ref_value;
        let err = (err / ulp).abs();
        if err >= u32::MAX as f32 {
            u32::MAX
        } else {
            err as u32
        }
    }

    #[inline(always)]
    fn check(&mut self, u: u32) {
        let x = f32::from_bits(u);
        assert!(x.is_finite());

        let our_value = (self.our_fn)(x);

        if self.tan_mode {
            let fract = x.fract();
            if fract == 0.5 || fract == -0.5 {
                assert!(our_value.is_infinite());
                return;
            }
        }

        self.tmp.set_f32(x, Round::TiesToEven);
        let inex = (self.ref_fn)(&mut self.tmp, &self.pi);
        self.tmp.subnormalize(inex, Round::TiesToEven);
        let ref_value = self.tmp.to_f32(Round::TiesToEven);

        if our_value != ref_value && !(our_value.is_nan() && ref_value.is_nan()) {
            self.errors.num_errors += 1;

            let err = self.ulp_error(our_value, ref_value, x);
            if err > 1 {
                self.errors.num_errors_2 += 1;
            }
            let err_f64 = self.ulp_error_f64(our_value, x);

            if err > self.errors.max_error_ulp
                || (err == self.errors.max_error_ulp && err_f64 > self.errors.max_error_f64)
            {
                self.errors.max_error_ulp = err;
                self.errors.max_error_f64 = err_f64;
                self.errors.max_error_val = u;
            }
        }
    }
}

fn check_exhaustive_f32(
    ref_fn: fn(&mut Float, &Float) -> i32,
    our_fn: fn(f32) -> f32,
    tan_mode: bool,
) -> FloatErrors {
    const SPLIT: u32 = 512;

    // Generate ranges.
    assert_eq!(COUNT % SPLIT, 0);
    let per_split = COUNT / SPLIT;

    let work_index = AtomicUsize::new(0);
    let mut work = (0..SPLIT)
        .map(|i| {
            let base = i * per_split;
            let end = i * per_split + per_split;
            TestRange { base, end }
        })
        .collect::<Vec<_>>();

    // The larger numbers towards the end of the test range are more costly to evaluate. So improve
    // scheduling by running those long jobs first.
    work.reverse();

    let mut errors = FloatErrors::default();

    // Spawn threads.
    std::thread::scope(|s| {
        let num_threads = std::thread::available_parallelism().unwrap().get();

        let threads = (0..num_threads)
            .map(|_| {
                s.spawn(|| {
                    let mut float_check = FloatCheck::new(ref_fn, our_fn, tan_mode);
                    loop {
                        let index = work_index.fetch_add(1, Ordering::SeqCst);
                        if let Some(range) = work.get(index) {
                            for i in range.base..range.end {
                                float_check.check(i);
                            }
                            for i in range.base..range.end {
                                float_check.check(0x8000_0000 + i);
                            }
                            continue;
                        }
                        break;
                    }
                    float_check.errors
                })
            })
            .collect::<Vec<_>>();

        for thread in threads {
            let thread_errors = thread.join().unwrap();
            errors.num_errors += thread_errors.num_errors;
            errors.num_errors_2 += thread_errors.num_errors_2;

            if thread_errors.max_error_ulp > errors.max_error_ulp
                || thread_errors.max_error_f64 > errors.max_error_f64
            {
                errors.max_error_ulp = thread_errors.max_error_ulp;
                errors.max_error_f64 = thread_errors.max_error_f64;
                errors.max_error_val = thread_errors.max_error_val;
            }
        }
    });

    errors
}

fn ref_sin_pi_f32(x: &mut Float, pi: &Float) -> i32 {
    x.mul_assign(pi, Round::TiesToEven);
    x.sin_mut(Round::TiesToEven)
}

fn ref_cos_pi_f32(x: &mut Float, pi: &Float) -> i32 {
    x.mul_assign(pi, Round::TiesToEven);
    x.cos_mut(Round::TiesToEven)
}

fn ref_tan_pi_f32(x: &mut Float, pi: &Float) -> i32 {
    x.mul_assign(pi, Round::TiesToEven);
    x.tan_mut(Round::TiesToEven)
}

fn ref_exp_f32(x: &mut Float, _: &Float) -> i32 {
    x.exp_mut(Round::TiesToEven)
}

#[test]
#[ignore]
pub fn exhaustive_sin_pi() {
    let errors = check_exhaustive_f32(ref_sin_pi_f32, |a| sin_cos_pi_f32(a).0, false);
    println!("SIN: {errors:?}");
    assert_eq!(errors.max_error_ulp, 1);
    assert_eq!(errors.num_errors, 55_943_962);
}

#[test]
#[ignore]
pub fn exhaustive_cos_pi() {
    let errors = check_exhaustive_f32(ref_cos_pi_f32, |a| sin_cos_pi_f32(a).1, false);
    println!("COS: {errors:?}");
    assert_eq!(errors.num_errors, 45_882_714);
    assert_eq!(errors.max_error_ulp, 1);
}

#[test]
#[ignore]
pub fn exhaustive_tan_pi() {
    let errors = check_exhaustive_f32(ref_tan_pi_f32, tan_pi_f32, true);
    println!("TAN: {errors:?}");
    assert_eq!(errors.num_errors, 100_555_422);
    assert_eq!(errors.max_error_ulp, 2);
}

#[test]
#[ignore]
pub fn exhaustive_exp() {
    let errors = check_exhaustive_f32(ref_exp_f32, |a| exp_f32(a), false);
    println!("EXP: {errors:?}");
    assert_eq!(errors.max_error_ulp, 1);
}
