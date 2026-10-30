//! The cycle-level datapath against the straight-line reference model.

use hchip_filter::coef::BlockFloat;
use hchip_filter::mult::Rounding;
use hchip_filter::reference::{reference_sample, RefOut};
use hchip_filter::{Params, Section};

/// Small deterministic PRNG (xorshift64*), to keep the crate dependency-free.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % (hi - lo + 1) as u64) as i64
    }
}

fn check(p: Params, dc: bool, seed: u64, samples: usize, input_range: i64) {
    let mut rng = Rng(seed | 1);
    let mut s = Section::new(p);
    s.dc = dc;
    s.set_codes(
        rng.range(0, 255) as u8,
        rng.range(0, 255) as u8,
        rng.range(0, 255) as u8,
        rng.range(0, 255) as u8,
    );
    let coefs: [BlockFloat; 5] = s.effective_coefs();
    for n in 0..samples {
        let x = rng.range(-input_range, input_range);
        let (z1, z2) = (s.dmem[0], s.dmem[1]);
        let want: RefOut = reference_sample(&p, dc, s.with_gain, x, z1, z2, coefs);
        let out = s.process(x);
        assert_eq!(
            out, want.out,
            "output differs at sample {n} (dc={dc}, seed={seed}, x={x}, z1={z1}, z2={z2})"
        );
        assert_eq!(
            s.dmem[0], want.y_stored,
            "stored Yn differs at sample {n} (dc={dc}, seed={seed})"
        );
    }
}

#[test]
fn matches_reference_with_gain_rows() {
    for seed in 1..20u64 {
        for dc in [false, true] {
            let mut rng = Rng(seed | 1);
            let p = Params::default();
            let mut s = Section::new(p);
            s.dc = dc;
            s.set_codes(
                rng.range(0, 255) as u8,
                rng.range(0, 255) as u8,
                rng.range(0, 255) as u8,
                rng.range(0, 255) as u8,
            );
            s.set_gain_q8(rng.range(0, 0xFFFF) as u16);
            let coefs = s.effective_coefs();
            for _ in 0..300 {
                let x = rng.range(-(1 << 15), 1 << 15);
                let (z1, z2) = (s.dmem[0], s.dmem[1]);
                let want = reference_sample(&p, dc, true, x, z1, z2, coefs);
                let out = s.process(x);
                assert_eq!(out, want.out, "dc={dc} seed={seed}");
                assert_eq!(s.dmem[0], want.y_stored);
            }
        }
    }
}

#[test]
fn matches_reference_default_widths() {
    for seed in 1..40u64 {
        for dc in [false, true] {
            check(Params::default(), dc, seed, 400, 1 << 15);
        }
    }
}

#[test]
fn matches_reference_with_saturating_accumulator_and_rounding() {
    let p = Params {
        acc_saturate: true,
        product_round: Rounding::Nearest,
        ..Params::default()
    };
    for seed in 1..20u64 {
        for dc in [false, true] {
            check(p, dc, seed, 400, 1 << 20);
        }
    }
}

#[test]
fn matches_reference_with_narrow_words() {
    let p = Params {
        acc_bits: 20,
        input_bits: 16,
        delay_bits: 14,
        mult_in_bits: 12,
        mult_lsb_drop: 2,
        mant_bits: 8,
        ..Params::default()
    };
    for seed in 1..20u64 {
        for dc in [false, true] {
            check(p, dc, seed, 400, 1 << 15);
        }
    }
}

#[test]
fn matches_reference_with_wrapping_operand_and_exact_coef_sum() {
    let p = Params {
        mult_in_saturate: false,
        product_saturate: false,
        exact_coef_sum: true,
        ..Params::default()
    };
    for seed in 1..20u64 {
        for dc in [false, true] {
            check(p, dc, seed, 400, 1 << 15);
        }
    }
}

/// With A1' = 2 and A2' = 1 the zero half is transparent: the section
/// output equals the value written to the delay memory, as long as nothing
/// saturates.
#[test]
fn pure_two_pole_output_equals_delay_value() {
    let mut rng = Rng(7);
    for dc in [false, true] {
        let mut s = Section::new(Params::default());
        s.dc = dc;
        s.set_coefs(0.17, 0.19, 1.0, 2.0);
        assert_eq!(s.effective_coefs_f64()[2], 1.0);
        assert_eq!(s.effective_coefs_f64()[3], 2.0);
        for _ in 0..2000 {
            let x = rng.range(-400, 400);
            let y = s.process(x);
            assert_eq!(y, s.dmem[0]);
        }
        assert_eq!(s.stats.delay_sat, 0);
    }
}

#[test]
fn first_sample_of_impulse() {
    // y[0] = x[0] for the all-pole section (DC off): H = 1/D(z).
    let mut s = Section::new(Params::default());
    s.set_coefs(0.17, 0.19, 1.0, 2.0);
    assert_eq!(s.process(12345), 12345);
    // y[0] = (B1' - B2') x[0] for the DC form: H = (B1' - B2')/D(z).
    let mut s = Section::new(Params::default());
    s.dc = true;
    s.set_coefs(0.17, 0.19, 1.0, 2.0);
    let eff = s.effective_coefs_f64();
    let want = (eff[1] - eff[0]) * 12345.0;
    let got = s.process(12345) as f64;
    assert!((got - want).abs() <= 2.0, "got {got}, want {want}");
}

/// Worst-case bus values for a single section with 16-bit inputs and
/// delays. With B2', A2' <= 1 and B1', A1' <= 5 (code 255 in each role) the
/// rows are bounded, in units of 2^15, by 3 (row 3), 2 (row 4), 7 (row 5),
/// 10 (row 6), 8 (row 7) and 13 (row 8), so 20 bits hold everything. This
/// drives the section with adversarial values and checks the bound; the
/// random drive reaches the row-5 bound of 7.
#[test]
fn single_section_worst_case_fits_in_20_bits() {
    use hchip_filter::microcode::MICROCODE;
    let p = Params {
        acc_bits: 32,
        ..Params::default()
    };
    let mut worst = 0i64;
    for (b2, b1, a2, a1) in [
        (255u8, 255u8, 255u8, 255u8),
        (255, 0, 255, 0),
        (0, 255, 0, 255),
    ] {
        let mut s = Section::new(p);
        s.set_codes(b2, b1, a2, a1);
        let mut rng = Rng(99);
        for n in 0..20000 {
            let x = match rng.range(0, 3) {
                0 => 32767,
                1 => -32768,
                _ => rng.range(-32768, 32767),
            };
            s.input = if n % 7 == 0 { x } else { x.signum() * 32767 };
            for ctl in MICROCODE.iter() {
                let t = s.step(ctl);
                worst = worst.max(t.sum.abs()).max(t.acc10.abs()).max(t.acc11.abs());
            }
        }
    }
    println!(
        "worst |bus value| = {worst} = {:.2} * 2^15",
        worst as f64 / 32768.0
    );
    assert!(worst < 1 << 19, "{worst} needs more than 20 bits");
    assert!(worst > 6 << 15, "adversarial drive too weak: {worst}");
}
