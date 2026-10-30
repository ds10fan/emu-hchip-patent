//! Does the section lowpass as the microcode says it should?
//!
//! The expected response is computed from the coefficients the decoder
//! actually delivers (quantised), so these tests check the arithmetic, not
//! the code table's resolution. Measurements use a sine whose amplitude is
//! scaled so that nothing saturates; the impulse-response DFT is a second
//! check away from DC, where the floor-truncation residual in the tail does
//! not matter.

use hchip_filter::design::{db, dft, ideal_response, lowpass_poles, pole_rt, primes, unprimes};
use hchip_filter::{impulse_response, sine_gain, Params, Section};
use std::f64::consts::{FRAC_1_SQRT_2, PI};

const FS: f64 = 48000.0;

fn section(dc: bool, fc: f64, q: f64, p: Params) -> Section {
    let (b1, b2) = lowpass_poles(fc, q, FS);
    let (b1p, b2p) = primes(b1, b2);
    let mut s = Section::new(p);
    s.dc = dc;
    s.set_coefs(b2p, b1p, 1.0, 2.0);
    s
}

fn max_gain(eff: [f64; 5], dc: bool) -> f64 {
    (0..=2400)
        .map(|k| ideal_response(eff, dc, false, PI * k as f64 / 2400.0).abs())
        .fold(0.0, f64::max)
}

/// Measure with a sine at each frequency; return the largest |error| in dB.
fn sine_errors(dc: bool, fc: f64, q: f64, p: Params, freqs: &[f64]) -> Vec<(f64, f64, f64)> {
    let proto = section(dc, fc, q, p);
    let eff = proto.effective_coefs_f64();
    let amp = (30000.0 / max_gain(eff, dc)).floor().min(30000.0);
    freqs
        .iter()
        .map(|&f| {
            let mut s = section(dc, fc, q, p);
            let g = sine_gain(&mut s, f, FS, amp, 4096, 8192);
            let ideal = ideal_response(eff, dc, false, 2.0 * PI * f / FS).abs();
            assert_eq!(s.stats.delay_sat, 0, "delay saturated at {f} Hz");
            (f, db(g), db(ideal))
        })
        .collect()
}

const FREQS: [f64; 13] = [
    20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 1500.0, 2000.0, 3000.0, 5000.0, 10000.0, 20000.0,
    23000.0,
];

#[test]
fn lowpass_1khz_dc_off_matches_ideal() {
    for (f, meas, ideal) in sine_errors(false, 1000.0, FRAC_1_SQRT_2, Params::default(), &FREQS) {
        assert!(
            (meas - ideal).abs() < 0.15,
            "{f} Hz: {meas:.2} vs {ideal:.2} dB"
        );
    }
}

#[test]
fn lowpass_1khz_dc_on_matches_ideal() {
    // Full-scale sines clip the X - Z operand near the cutoff in a 16-bit
    // LATCH 18 (see docs/WIDTHS_AND_SATURATION.md), so measure at 18 bits
    // here; the 16-bit behaviour is covered by `dc_on_operand_clips`.
    let p = Params {
        mult_in_bits: 18,
        ..Params::default()
    };
    for (f, meas, ideal) in sine_errors(true, 1000.0, FRAC_1_SQRT_2, p, &FREQS) {
        assert!(
            (meas - ideal).abs() < 0.15,
            "{f} Hz: {meas:.2} vs {ideal:.2} dB"
        );
    }
}

#[test]
fn resonant_1khz_q10_dc_on_matches_ideal() {
    let r = sine_errors(true, 1000.0, 10.0, Params::default(), &FREQS);
    for &(f, meas, ideal) in &r {
        assert!(
            (meas - ideal).abs() < 0.15,
            "{f} Hz: {meas:.2} vs {ideal:.2} dB"
        );
    }
    // and it is a resonance: about +19.5 dB at 1 kHz, falling on both sides
    let at = |f: f64| r.iter().find(|x| x.0 == f).unwrap().1;
    assert!(at(1000.0) > 18.0 && at(1000.0) < 21.0);
    assert!(at(500.0) < 4.0 && at(2000.0) < -8.0);
}

#[test]
fn lowpass_100hz_dc_on_matches_ideal() {
    let p = Params {
        mult_in_bits: 18,
        ..Params::default()
    };
    for (f, meas, ideal) in sine_errors(true, 100.0, FRAC_1_SQRT_2, p, &FREQS) {
        if ideal > -60.0 {
            assert!(
                (meas - ideal).abs() < 0.2,
                "{f} Hz: {meas:.2} vs {ideal:.2} dB"
            );
        }
    }
}

#[test]
fn it_is_a_lowpass() {
    // Monotonic above the cutoff, about -12 dB/octave well above it, and the
    // DC gain is 1 (DC on) or 1/(B1' - B2') (DC off).
    for dc in [false, true] {
        let p = Params {
            mult_in_bits: 18,
            ..Params::default()
        };
        let octaves = [2000.0, 4000.0, 8000.0, 16000.0];
        let r = sine_errors(dc, 1000.0, FRAC_1_SQRT_2, p, &octaves);
        for w in r.windows(2) {
            assert!(w[1].1 < w[0].1 - 6.0, "not falling: {:?}", r);
        }
        let slope = r[2].1 - r[1].1;
        assert!(slope < -10.0 && slope > -13.0, "slope {slope} dB/oct");
        let eff = section(dc, 1000.0, FRAC_1_SQRT_2, p).effective_coefs_f64();
        let dc_gain = if dc { 1.0 } else { 1.0 / (eff[1] - eff[0]) };
        let low = sine_errors(dc, 1000.0, FRAC_1_SQRT_2, p, &[20.0]);
        assert!((low[0].1 - db(dc_gain)).abs() < 0.15);
    }
}

#[test]
fn impulse_dft_matches_away_from_dc() {
    for (dc, amp) in [(false, 2048), (true, 32767)] {
        let mut s = section(dc, 1000.0, FRAC_1_SQRT_2, Params::default());
        let eff = s.effective_coefs_f64();
        let h = impulse_response(&mut s, amp, 16384);
        assert_eq!(s.stats.delay_sat, 0);
        for f in [500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0] {
            let w = 2.0 * PI * f / FS;
            let meas = db(dft(&h, w).abs() / amp as f64);
            let ideal = db(ideal_response(eff, dc, false, w).abs());
            assert!(
                (meas - ideal).abs() < 0.15,
                "dc={dc} {f} Hz: {meas:.2} vs {ideal:.2}"
            );
        }
    }
}

/// The quantised poles are where the design asked, within the code table's
/// resolution.
#[test]
fn quantised_poles_are_close() {
    let s = section(true, 1000.0, FRAC_1_SQRT_2, Params::default());
    let eff = s.effective_coefs_f64();
    let (b1, b2) = unprimes(eff[1], eff[0]);
    let (r, theta) = pole_rt(b1, b2);
    let f = theta * FS / (2.0 * PI);
    assert!(r > 0.89 && r < 0.93, "R = {r}");
    assert!(f > 600.0 && f < 800.0, "pole angle {f} Hz");
}

/// With DC off a 1 kHz section has 36 dB of DC gain and a full-scale input
/// saturates the delay memory; the output stays bounded and the counters
/// say so.
#[test]
fn dc_off_full_scale_saturates_but_stays_bounded() {
    let mut s = section(false, 1000.0, FRAC_1_SQRT_2, Params::default());
    let mut peak = 0i64;
    for i in 0..4000 {
        let x = (30000.0 * (2.0 * PI * 100.0 * i as f64 / FS).sin()).round() as i64;
        peak = peak.max(s.process(x).abs());
    }
    assert!(s.stats.delay_sat > 0);
    assert!(peak < 1 << 18, "peak {peak}");
}

/// With DC on, the X - Z operand of a full-scale sine near the cutoff
/// exceeds 16 bits and LATCH 18 clips it.
#[test]
fn dc_on_operand_clips_at_16_bits() {
    let mut s = section(true, 1000.0, FRAC_1_SQRT_2, Params::default());
    sine_gain(&mut s, 1000.0, FS, 30000.0, 4096, 8192);
    assert!(s.stats.mult_in_sat > 0);
    let mut s = section(true, 1000.0, FRAC_1_SQRT_2, Params::default());
    sine_gain(&mut s, 1000.0, FS, 15000.0, 4096, 8192);
    assert_eq!(s.stats.mult_in_sat, 0);
}

/// A 6-bit coefficient mantissa cannot carry B2' plus a small difference:
/// at 100 Hz the difference is lost, B1' == B2', and the DC-off section has
/// a pole on the unit circle.
#[test]
fn six_bit_mantissa_loses_the_difference() {
    let p6 = Params {
        mant_bits: 6,
        ..Params::default()
    };
    let eff6 = section(false, 100.0, FRAC_1_SQRT_2, p6).effective_coefs_f64();
    assert_eq!(eff6[0], eff6[1]);
    let eff16 = section(false, 100.0, FRAC_1_SQRT_2, Params::default()).effective_coefs_f64();
    assert!(eff16[1] > eff16[0]);
}
