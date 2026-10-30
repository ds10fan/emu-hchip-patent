//! The interpolator against Table II of the patent and the voice sweeping
//! its cutoff.

use hchip_filter::design::{db, Complex};
use hchip_filter::interp::{approach_samples, decode_increment, dup, encode_increment};
use hchip_filter::voice::{stock_type, word_info};
use hchip_filter::{Params, Voice};
use std::f64::consts::PI;

#[path = "table_ii_data.rs"]
mod table_ii;

const FS: f64 = 48000.0;

/// Every row of Table II, every approach time, the ends and the middle of
/// each difference range.
#[test]
fn table_ii_is_reproduced() {
    let mut checked = 0;
    for &(lo, hi, codes) in table_ii::TABLE_II {
        for k in 0..8u8 {
            for d17 in [lo, hi, (lo + hi) / 2] {
                let diff = if d17 & 0x10000 != 0 {
                    d17 as i32 - 0x20000
                } else {
                    d17 as i32
                };
                let got = encode_increment(diff, k);
                assert_eq!(
                    got, codes[k as usize],
                    "difference {d17:05x} ({diff}) at 32<<{k}: got {got:02x}, table {:02x}",
                    codes[k as usize]
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 1800);
}

/// The columns halve: the same difference over twice the time gives the
/// code of half the increment (floored).
#[test]
fn approach_times_halve_the_increment() {
    for &(lo, _, codes) in table_ii::TABLE_II {
        let diff = if lo & 0x10000 != 0 {
            lo as i32 - 0x20000
        } else {
            lo as i32
        };
        for k in 0..7u8 {
            let a = decode_increment(codes[k as usize]);
            let b = decode_increment(codes[k as usize + 1]);
            assert!(b <= a / 2 + 1 && b >= a / 2 - 1024, "{diff}: {a} -> {b}");
        }
        assert_eq!(approach_samples(0), 32);
        assert_eq!(approach_samples(7), 4096);
    }
}

fn measure(v: &Voice, f: f64, amp: f64) -> f64 {
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
    acc.abs() / (amp * n as f64 / 2.0)
}

/// A sweep from cutoff 0 to 255 takes about 32 << k samples, the codes
/// follow x on the way, and at the end the voice is the static cutoff-255
/// voice.
#[test]
fn voice_sweeps_to_the_destination() {
    let (words, flags) = stock_type("lp2", 64).unwrap();
    let p = Params {
        mult_in_bits: 18,
        ..Params::default()
    };
    let mut v = Voice::new(p);
    v.set(words, 0, flags);
    v.sweep_to(255, 5); // 1024 samples
    let mut n = 0;
    let mut mid_checked = false;
    while !v.interp.settled() {
        v.process(0);
        n += 1;
        if !mid_checked && v.x() >= 0x8080 {
            // x just crossed cutoff 128: word 0 decoded for cutoff 128
            let want = word_info(words[0], 128, FS);
            let eff = v.sections[0].effective_coefs_f64();
            assert!(
                (eff[0] - want.b2p).abs() < 0.02 * want.b2p,
                "{} vs {}",
                eff[0],
                want.b2p
            );
            assert!((500..=530).contains(&n), "crossed at {n}");
            mid_checked = true;
        }
        assert!(n < 2000);
    }
    assert!(mid_checked);
    // 65535 >> 10 = 63 floors to 62 on the lattice, plus the carry: 63 per
    // sample, 1041 samples for the nominal 1024
    assert!((1024..=1056).contains(&n), "{n} samples");
    assert_eq!(v.x(), 0xffff);
    let mut stat = Voice::new(p);
    stat.set(words, 255, flags);
    assert_eq!(v.sections[0].coef_mem, stat.sections[0].coef_mem);
    assert_eq!(v.sections[2].coef_mem, stat.sections[2].coef_mem);
    // and it measures the same
    let amp = 5000.0;
    for f in [500.0, 5000.0] {
        let a = db(measure(&v, f, amp));
        let b = db(measure(&stat, f, amp));
        assert!((a - b).abs() < 0.01, "{f} Hz: {a} vs {b}");
    }
}

/// Retargeting mid-sweep: the increment is recomputed from the current x,
/// so the new destination is reached in its own approach time.
#[test]
fn retarget_mid_sweep() {
    let mut v = Voice::new(Params::default());
    v.sweep_to(255, 7); // slow, 4096 samples
    for _ in 0..1000 {
        v.process(0);
    }
    let x_mid = v.x();
    assert!(x_mid > 0x3000 && x_mid < 0x4800, "{x_mid:04x}");
    v.sweep_to(64, 0); // fast, down
    let mut n = 0;
    while !v.interp.settled() {
        v.process(0);
        n += 1;
        assert!(n < 100);
    }
    assert_eq!(v.x(), dup(64));
    assert!(n <= 34, "{n}");
}

/// x at rest at a cutoff gives exactly the static codes (the forced carry
/// is clamped away every sample).
#[test]
fn resting_x_is_static() {
    let (words, flags) = stock_type("lp6", 64).unwrap();
    let mut v = Voice::new(Params::default());
    v.set(words, 100, flags);
    let codes = v.sections[1].coef_mem;
    for _ in 0..100 {
        v.process(123);
    }
    assert_eq!(v.x(), dup(100));
    assert_eq!(v.sections[1].coef_mem, codes);
}
