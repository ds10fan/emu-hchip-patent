//! Cycle-level emulator of the IIR filter section of US 5,170,369 (E-mu
//! "H-chip" dynamic digital audio filter): FIG. 6 datapath, FIG. 7 block
//! floating point multiplier, FIG. 8 coefficient decoder, driven by the
//! reservation table as microcode.

pub mod bits;
pub mod coef;
pub mod design;
pub mod microcode;
pub mod mult;
pub mod reference;
pub mod section;

pub use section::{Params, Section};

/// Impulse response of `n` samples for an impulse of height `amp`.
pub fn impulse_response(s: &mut Section, amp: i64, n: usize) -> Vec<i64> {
    (0..n)
        .map(|i| s.process(if i == 0 { amp } else { 0 }))
        .collect()
}

/// Steady-state gain at frequency `f` (Hz) measured with a sine of
/// amplitude `amp`: the section runs `settle` samples, then the output is
/// correlated with the input sine over `n` samples.
pub fn sine_gain(s: &mut Section, f: f64, fs: f64, amp: f64, settle: usize, n: usize) -> f64 {
    let w = 2.0 * std::f64::consts::PI * f / fs;
    let mut acc = design::Complex::new(0.0, 0.0);
    for i in 0..settle + n {
        let x = (amp * (w * i as f64).sin()).round() as i64;
        let y = s.process(x) as f64;
        if i >= settle {
            acc = acc + design::Complex::expj(-w * i as f64).scale(y);
        }
    }
    // |correlation| = amp * n / 2 * gain for a pure sine
    acc.abs() / (amp * n as f64 / 2.0)
}
