//! The voice model against VOICE_INTERFACE.md and against its own ideal
//! response.

use hchip_filter::design::{db, Complex};
use hchip_filter::voice::{stock_type, word_info, NEUTRAL_WORD};
use hchip_filter::{Params, Voice};
use std::f64::consts::PI;

const FS: f64 = 48000.0;

fn measure(v: &Voice, f: f64, amp: f64) -> (f64, Voice) {
    let w = 2.0 * PI * f / FS;
    let mut v2 = v.clone();
    v2.reset_state();
    let mut acc = Complex::new(0.0, 0.0);
    let (settle, n) = (4096usize, 8192usize);
    for i in 0..settle + n {
        let x = (amp * (w * i as f64).sin()).round() as i64;
        let y = v2.process(x) as f64;
        if i >= settle {
            acc = acc + Complex::expj(-w * i as f64).scale(y);
        }
    }
    (acc.abs() / (amp * n as f64 / 2.0), v2)
}

/// Compare the voice's measured response with its ideal one at a set of
/// frequencies, with the input scaled so that no section saturates.
/// Points where the ideal output is under 100 LSB are at the quantisation
/// floor and are skipped.
fn check_voice(v: &Voice, freqs: &[f64], tol_db: f64) {
    let amp = (30000.0 / v.max_internal_gain()).floor().min(30000.0);
    for &f in freqs {
        let w = 2.0 * PI * f / FS;
        let ideal_lin = v.ideal_response(w).abs();
        if ideal_lin * amp < 100.0 {
            continue;
        }
        let ideal = db(ideal_lin);
        let (g, v2) = measure(v, f, amp);
        for s in &v2.sections {
            assert_eq!(s.stats.delay_sat, 0, "delay saturated at {f} Hz");
        }
        assert!(
            (db(g) - ideal).abs() < tol_db,
            "{f} Hz: measured {:.2} dB, ideal {ideal:.2} dB",
            db(g)
        );
    }
}

fn wide() -> Params {
    // 18-bit operand so the DC-on form's X - Z does not clip (see
    // WIDTHS_AND_SATURATION.md); everything else default.
    Params {
        mult_in_bits: 18,
        ..Params::default()
    }
}

#[test]
fn word_table_matches_document_lp2_q64() {
    // 2 Pole Lowpass, Q 64, word 0 = 6A07E6E3 (VOICE_INTERFACE.md)
    let rows: [(u8, f64, f64, f64, f64, f64, f64); 5] = [
        (0, 106.00, 7.00, 0.003296, 0.000122, 84.0, 3.35),
        (64, 137.12, 62.22, 0.012755, 0.001905, 335.0, 3.41),
        (128, 168.24, 117.43, 0.049303, 0.021906, 1146.0, 2.97),
        (192, 199.36, 172.65, 0.190349, 0.231618, 3915.0, 2.43),
        (255, 230.00, 227.00, 0.609375, 2.250000, 18150.0, 2.53),
    ];
    // The document interpolates codes as exact reals; the voice keeps 8
    // fraction bits, so coefficients agree to about 1 part in 4000.
    let close = |a: f64, b: f64| (a - b).abs() <= 3e-4 * b.abs() + 1e-6;
    for (cutoff, x, f, b2p, f2, f0, q) in rows {
        let i = word_info(0x6A07_E6E3, cutoff, FS);
        assert!(
            (i.x_code - x).abs() < 0.01,
            "cutoff {cutoff}: X {}",
            i.x_code
        );
        assert!(
            (i.f_code - f).abs() < 0.01,
            "cutoff {cutoff}: F {}",
            i.f_code
        );
        assert!(close(i.b2p, b2p), "cutoff {cutoff}: B2' {}", i.b2p);
        assert!(close(i.f2, f2), "cutoff {cutoff}: f2 {}", i.f2);
        assert!(
            (i.f0 - f0).abs() < 0.003 * f0 + 0.5,
            "cutoff {cutoff}: f0 {}",
            i.f0
        );
        assert!((i.q - q).abs() < 0.02, "cutoff {cutoff}: Q {}", i.q);
    }
}

#[test]
fn word_table_matches_document_hp4_q64() {
    // zero words 80048004: 67 Hz, Q 1.05 at any cutoff; pole words 7004EFCF
    for cutoff in [0u8, 128, 255] {
        let z = word_info(0x8004_8004, cutoff, FS);
        assert!((z.b2p - 0.008301).abs() < 3e-6);
        assert!((z.f2 - 0.000076).abs() < 2e-6);
        assert!((z.f0 - 67.0).abs() < 1.0 && (z.q - 1.05).abs() < 0.01);
    }
    let p = word_info(0x7004_EFCF, 128, FS);
    assert!((p.b2p - 0.065426).abs() < 2e-5, "{}", p.b2p);
    assert!((p.f2 - 0.013134).abs() < 5e-6, "{}", p.f2);
    assert!((p.f0 - 891.0).abs() < 3.0 && (p.q - 1.72).abs() < 0.02);
}

#[test]
fn neutral_voice_is_transparent() {
    // With an 18-bit LATCH 18. At 16 bits the DC-on form clips X - Z for a
    // full-scale jump (-20000 to 32767), so even the neutral word is not
    // transparent at full scale; see `neutral_word_clips_at_16_bits`.
    let mut v = Voice::new(wide());
    v.set([NEUTRAL_WORD; 10], 100, 0x00);
    for (i, x) in [0i64, 1000, -20000, 32767, -32768, 5]
        .into_iter()
        .enumerate()
    {
        let y = v.process(x);
        assert_eq!(y, x, "sample {i}");
    }
    // with the gain word at full scale (0xFF -> 1.0) too
    let mut w = [NEUTRAL_WORD; 10];
    w[9] = 0x00FF_00FF;
    v.set(w, 0, 0x40);
    v.reset_state();
    for x in [1000i64, -20000, 32767] {
        assert_eq!(v.process(x), x);
    }
}

#[test]
fn neutral_word_clips_at_16_bits() {
    let mut v = Voice::new(Params::default());
    v.set([NEUTRAL_WORD; 10], 100, 0x00);
    let mut ok = true;
    for x in [0i64, 1000, -20000, 32767] {
        ok &= v.process(x) == x;
    }
    assert!(!ok);
    assert!(v.sections[0].stats.mult_in_sat > 0);
}

#[test]
fn lp2_q64_response_matches_ideal_at_three_cutoffs() {
    let (words, flags) = stock_type("lp2", 64).unwrap();
    // At cutoff 0 (84 Hz, Q 3.35) the products of the DC-on form are below
    // one LSB and floor truncation costs up to 0.25 dB near the resonance.
    for (cutoff, tol) in [(0u8, 0.5), (128, 0.2), (255, 0.2)] {
        let mut v = Voice::new(wide());
        v.set(words, cutoff, flags);
        assert_eq!(v.cycles_per_sample(), 9 + 9 + 12);
        check_voice(&v, &[50.0, 200.0, 1000.0, 3000.0, 10000.0, 20000.0], tol);
    }
}

#[test]
fn lp2_is_a_lowpass_that_moves_with_the_cutoff() {
    let (words, flags) = stock_type("lp2", 64).unwrap();
    let mut v = Voice::new(wide());
    v.set(words, 128, flags);
    let amp = (30000.0 / v.max_internal_gain()).floor();
    let g = |f: f64| db(measure(&v, f, amp).0);
    // ~1146 Hz, Q ~3: below the cutoff near the gain word's level, a peak
    // near 1.1 kHz, and 24 dB down an octave or two above it
    let (low, peak, high, top) = (g(100.0), g(1100.0), g(4400.0), g(16000.0));
    assert!(peak > low + 6.0, "no resonance: {low} {peak}");
    assert!(high < peak - 20.0, "{peak} {high}");
    assert!(top < high - 15.0, "{high} {top}");
    // at cutoff 255 the 100 Hz gain is the gain word alone (0xFE at 255)
    v.set_cutoff(255);
    let g255 = db(measure(&v, 100.0, 8000.0).0);
    let gain_word = v.sections[2].effective_coefs_f64()[4];
    assert!(
        (g255 - db(gain_word)).abs() < 0.1,
        "{g255} vs {}",
        db(gain_word)
    );
}

#[test]
fn lp6_q64_response_matches_ideal() {
    let (words, flags) = stock_type("lp6", 64).unwrap();
    let mut v = Voice::new(wide());
    v.set(words, 128, flags);
    check_voice(&v, &[50.0, 500.0, 1000.0, 1500.0, 2000.0, 2500.0], 0.25);
    // Three resonant sections in series fall off a cliff above the peak:
    // the drop from 1.5 to 2.5 kHz is the ideal one (over 25 dB). Further
    // up the output is below the quantisation floor for an input that
    // keeps the 20 dB internal peak inside the delay memory.
    let amp = (30000.0 / v.max_internal_gain()).floor();
    let a = db(measure(&v, 1500.0, amp).0);
    let b = db(measure(&v, 2500.0, amp).0);
    let ideal = |f: f64| db(v.ideal_response(2.0 * PI * f / FS).abs());
    let want = ideal(1500.0) - ideal(2500.0);
    assert!(want > 25.0, "ideal drop {want}");
    assert!((a - b - want).abs() < 1.0, "drop {} vs ideal {want}", a - b);
}

#[test]
fn hp4_q64_response_matches_ideal_and_is_a_highpass() {
    let (words, flags) = stock_type("hp4", 64).unwrap();
    let mut v = Voice::new(wide());
    v.set(words, 128, flags);
    // sections 1 and 2 run unnormalised, section 0 is neutral
    assert!(v.sections[0].dc && !v.sections[1].dc && !v.sections[2].dc);
    // Zeros come before the unnormalised poles, so below 1 kHz the signal
    // between the sections is a few hundred LSB and the comparison is
    // noise-limited; 300 Hz (-36 dB) is held to 0.5 dB, the rest to 0.25.
    check_voice(&v, &[300.0], 0.5);
    check_voice(&v, &[891.0, 2000.0, 5000.0, 15000.0], 0.25);
    let amp = (30000.0 / v.max_internal_gain()).floor();
    let g = |f: f64| db(measure(&v, f, amp).0);
    let (low, mid, high) = (g(100.0), g(300.0), g(5000.0));
    assert!(high > low + 30.0, "{low} {high}");
    assert!(mid > low + 15.0 && mid < high, "{low} {mid} {high}");
}

#[test]
fn gain_word_scales_the_output() {
    let mut w = [NEUTRAL_WORD; 10];
    let mut v = Voice::new(Params::default());
    for (code, expect) in [(0xFFu8, 1.0f64), (0xF6, 0.859375), (0x80, 0.008301)] {
        w[9] = ((code as u32) << 16) | code as u32;
        v.set(w, 0, 0x40);
        v.reset_state();
        let (g, _) = measure(&v, 1000.0, 20000.0);
        assert!(
            (g - expect).abs() < 0.002,
            "code {code:02X}: {g} vs {expect}"
        );
    }
}
