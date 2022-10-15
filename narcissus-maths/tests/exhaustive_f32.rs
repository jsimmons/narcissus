/// Rough translation of Paul Zimmermann's binary32_exhaustive to Rust.
/// https://gitlab.inria.fr/zimmerma/math_accuracy/-/blob/master/binary32_exhaustive/check_exhaustive.c
use std::{
    ops::MulAssign,
    sync::atomic::{AtomicUsize, Ordering},
};

use narcissus_maths::{next_after_f32, sin_cos_pi_f32, tan_pi_f32};
use rug::{
    ops::{AssignRound, DivAssignRound},
    Assign,
};

fn ulp_error_f32(our_val: f32, ref_val: f32, x: f32, reference: fn(&mut rug::Float)) -> u32 {
    if our_val == ref_val {
        return 0;
    }

    let mut our_val = our_val;
    let mut ref_val = ref_val;

    if ref_val.is_infinite() {
        if our_val.is_infinite() {
            // Then ours and reference have different signs.
            assert!(our_val * ref_val < 0.0);
            return u32::MAX;
        }

        our_val /= 2.0;
        let mut ref_val_2 = rug::Float::new(24);
        ref_val_2.assign_round(x, rug::float::Round::Down);
        reference(&mut ref_val_2);
        ref_val_2.div_assign_round(2, rug::float::Round::Down);
        ref_val = ref_val_2.to_f32_round(rug::float::Round::Down);
        if ref_val.is_infinite() {
            ref_val = if ref_val > 0.0 {
                f32::from_bits(0x7f00_0000)
            } else {
                f32::from_bits(0xff00_0000)
            };
        }
    }

    if our_val.is_infinite() {
        assert!(!ref_val.is_infinite());
        // If ours gives +/-Inf but the correct rounding is in the binary32 range, assume we gave
        // +/-2^128.
        ref_val /= 2.0;
        our_val = if our_val > 0.0 {
            f32::from_bits(0x7f00_0000)
        } else {
            f32::from_bits(0xff00_0000)
        }
    }

    let err = our_val - ref_val;
    let ulp = next_after_f32(ref_val, our_val) - ref_val;
    let err = (err / ulp).abs();
    if err >= u32::MAX as f32 {
        u32::MAX
    } else {
        err as u32
    }
}

#[derive(Clone, Copy, Debug)]
struct TestRange {
    base: u32,
    end: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct FloatErrors {
    num_errors: u32,
    max_error_ulp: u32,
    max_error_val: u32,
}

const PREC: u32 = 280;

struct FloatCheck {
    reference: fn(&mut rug::Float),
    ours: fn(f32) -> f32,

    tan_mode: bool,

    pi: rug::Float,
    tmp: rug::Float,

    errors: FloatErrors,
}

impl FloatCheck {
    fn new(reference: fn(&mut rug::Float), ours: fn(f32) -> f32, tan_mode: bool) -> Self {
        Self {
            reference,
            ours,
            tan_mode,
            pi: rug::Float::with_val(PREC, rug::float::Constant::Pi),
            tmp: rug::Float::new(PREC),
            errors: Default::default(),
        }
    }

    #[inline(always)]
    fn check(&mut self, u: u32) {
        let x = f32::from_bits(u);
        assert!(x.is_finite());

        let our_val = (self.ours)(x);

        if self.tan_mode {
            let fract = x.fract();
            if fract == 0.5 || fract == -0.5 {
                assert!(our_val.is_infinite());
                return;
            }
        }

        self.tmp.assign(x);
        self.tmp.mul_assign(&self.pi);
        (self.reference)(&mut self.tmp);
        self.tmp.subnormalize_ieee();
        let ref_val = self.tmp.to_f32_round(rug::float::Round::Nearest);

        if our_val != ref_val && !(our_val.is_nan() && ref_val.is_nan()) {
            println!("u: {u:#08x}, our_val: {our_val}, ref_val: {ref_val}");

            self.errors.num_errors += 1;
            let e = ulp_error_f32(our_val, ref_val, x, self.reference);
            if e > self.errors.max_error_ulp {
                self.errors.max_error_ulp = e;
                self.errors.max_error_val = u;
            }
        }
    }
}

fn check_exhaustive_f32(reference: fn(&mut rug::Float), ours: fn(f32) -> f32, tan_mode: bool) {
    const COUNT: u32 = 2_139_095_040_u32;
    const SPLIT: u32 = 256;

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

    // Try and start with big numbers to avoid reallocs?
    work.reverse();

    let count = AtomicUsize::new(0);

    let mut errors = FloatErrors::default();

    // Spawn threads.
    std::thread::scope(|s| {
        let num_threads = std::thread::available_parallelism().unwrap().get();

        let threads = (0..num_threads)
            .map(|_| {
                s.spawn(|| {
                    let mut float_check = FloatCheck::new(reference, ours, tan_mode);
                    loop {
                        let index = work_index.fetch_add(1, Ordering::SeqCst);
                        if let Some(range) = work.get(index) {
                            let start = std::time::Instant::now();
                            for i in range.base..range.end {
                                float_check.check(i);
                            }
                            for i in range.base..range.end {
                                float_check.check(0x8000_0000 + i);
                            }

                            let i = count.fetch_add(1, Ordering::SeqCst);
                            println!(
                                "{:.1}% chunk {index} took {:?}",
                                ((i + 1) as f32 * (1.0 / SPLIT as f32)) * 100.0,
                                std::time::Instant::now() - start
                            );

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
            if thread_errors.max_error_ulp > errors.max_error_ulp {
                errors.max_error_ulp = thread_errors.max_error_ulp;
                errors.max_error_val = thread_errors.max_error_val;
            }
        }
    });

    println!(
        "errors: {} ({:.2}%)",
        errors.num_errors,
        (errors.num_errors as f64 / (COUNT * 2) as f64) * 100.0
    );
    println!("max error ulps: {}", errors.max_error_ulp);
    println!("max error error at: {:#08x}", errors.max_error_val);
    assert_eq!(errors.num_errors, 0);
}

fn ref_sin_pi_f32(x: &mut rug::Float) {
    x.sin_mut();
}

fn ref_cos_pi_f32(x: &mut rug::Float) {
    x.cos_mut();
}

fn ref_tan_pi_f32(x: &mut rug::Float) {
    x.tan_mut();
}

#[test]
#[ignore]
pub fn exhaustive_sin_pi() {
    check_exhaustive_f32(ref_sin_pi_f32, |a| sin_cos_pi_f32(a).0, false)
}

#[test]
#[ignore]
pub fn exhaustive_cos_pi() {
    check_exhaustive_f32(ref_cos_pi_f32, |a| sin_cos_pi_f32(a).1, false)
}

#[test]
#[ignore]
pub fn exhaustive_tan_pi() {
    check_exhaustive_f32(ref_tan_pi_f32, tan_pi_f32, true)
}
