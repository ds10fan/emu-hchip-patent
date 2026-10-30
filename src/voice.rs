//! One voice as the E4 firmware programs it (VOICE_INTERFACE.md): ten
//! 32-bit control words, a cutoff and a flags byte, mapped onto three
//! sections of the patent's datapath run in series.
//!
//! Word layout: the upper half-word is the frame for cutoff 0, the lower
//! one for cutoff 255; each half-word is an X code (high byte) and an F
//! code (low byte). The code used at a cutoff is interpolated linearly in
//! code space and carries 8 fraction bits into the decoder.
//!
//! Section k uses pole word 4k (B2' from X, B1' = B2' + f² from F) and
//! zero word 4k+1 (A2', A1' likewise). Words 2, 3, 6, 7 are not used. With
//! `FLAG_GAIN` word 9 is not a zero word: its F bytes are the output gain,
//! run as the gain rows of section 2, and section 2's zeros are neutral.
//! With `FLAG_UNNORMALISED` sections 1 and 2 run the DC-off form (1/D);
//! section 0 is always the DC-on form (f²/D).
//!
//! The code interpolation C(x) = Ca + x(Cb − Ca) is done here in software;
//! in the patent it is the coefficient math unit that precedes the decoder
//! and uses the multiplier's free slots. x is the voice's interpolating
//! variable (`interp::Interpolator`), 16 bits, a cutoff byte duplicated
//! (cutoff 255 = 0xFFFF). It is updated once per sample; `set` jumps it to
//! the cutoff, `sweep_to` lets it move there at one of the eight approach
//! times.

use crate::coef::{decode_q8_f64, Role, CODE_ONE_F, CODE_ONE_X, FRAC_BITS};
use crate::design::{pole_fq, Complex};
use crate::interp::{dup, Interpolator};
use crate::section::{Params, Section, Stats};

/// X = 0xFF, F = 0xCF in both frames: B2' = 1, f² = 1, transparent in every role.
pub const NEUTRAL_WORD: u32 = 0xFFCF_FFCF;
/// Word 9 is a gain, not a zero word.
pub const FLAG_GAIN: u8 = 0x40;
/// Words 4 and 8 run unnormalised (1/D) instead of normalised (f²/D).
pub const FLAG_UNNORMALISED: u8 = 0x80;

const POLE_WORDS: [usize; 3] = [0, 4, 8];
const ZERO_WORDS: [usize; 3] = [1, 5, 9];

/// The four code bytes of a word: (X at cutoff 0, F at cutoff 0, X at 255, F at 255).
pub fn word_codes(word: u32) -> (u8, u8, u8, u8) {
    (
        (word >> 24) as u8,
        (word >> 16) as u8,
        (word >> 8) as u8,
        word as u8,
    )
}

/// C(x) = Ca + x (Cb − Ca) with x a 16-bit fraction of 65536, rounded to
/// 8 fraction bits of a code. For a cutoff byte c, x = c * 0x0101, which
/// makes this c0 + (c255 − c0) * c / 255 to within 1/65536 of a code.
pub fn interp_x(c0: u8, c255: u8, x: u16) -> u16 {
    let c0 = c0 as i32;
    let c255 = c255 as i32;
    ((c0 * 65536 + (c255 - c0) * x as i32 + 128) >> 8) as u16
}

/// The code at a static cutoff byte.
pub fn interp_q8(c0: u8, c255: u8, cutoff: u8) -> u16 {
    interp_x(c0, c255, dup(cutoff))
}

/// What one word means at a cutoff, in the document's terms.
#[derive(Clone, Copy, Debug)]
pub struct WordInfo {
    /// Interpolated codes, as real numbers.
    pub x_code: f64,
    pub f_code: f64,
    pub b2p: f64,
    pub f2: f64,
    /// Natural frequency (Hz) and Q of the root pair of D(z).
    pub f0: f64,
    pub q: f64,
}

/// Decode a word at a cutoff (X as a second coefficient, F as the
/// difference) without the decoder's mantissa limit.
pub fn word_info(word: u32, cutoff: u8, fs: f64) -> WordInfo {
    let (x0, f0, x255, f255) = word_codes(word);
    let xq = interp_q8(x0, x255, cutoff);
    let fq = interp_q8(f0, f255, cutoff);
    let unit = (1u64 << FRAC_BITS) as f64;
    let b2p = decode_q8_f64(xq) * (1u64 << Role::Second.shift()) as f64 / unit;
    let f2 = decode_q8_f64(fq) * (1u64 << Role::First.shift()) as f64 / unit;
    let b1p = b2p + f2;
    let (f0hz, q) = pole_fq(b1p - 2.0, 1.0 - b2p, fs);
    WordInfo {
        x_code: xq as f64 / 256.0,
        f_code: fq as f64 / 256.0,
        b2p,
        f2,
        f0: f0hz,
        q,
    }
}

#[derive(Clone, Debug)]
pub struct Voice {
    pub p: Params,
    pub words: [u32; 10],
    /// The programmed cutoff: the interpolator's clamp (destination).
    pub cutoff: u8,
    pub flags: u8,
    pub sections: [Section; 3],
    /// The interpolating variable x, shared by all words of the voice.
    pub interp: Interpolator,
    /// x the sections' codes were last computed for.
    applied_x: u16,
}

impl Voice {
    pub fn new(p: Params) -> Self {
        let mut v = Voice {
            p,
            words: [NEUTRAL_WORD; 10],
            cutoff: 0,
            flags: FLAG_GAIN,
            sections: [Section::new(p), Section::new(p), Section::new(p)],
            interp: Interpolator::at(0),
            applied_x: 0,
        };
        v.words[9] = 0x00FF_00FF;
        v.apply();
        v
    }

    /// Program the voice with the cutoff applied at once (x jumps).
    /// Coefficient memories are reloaded; the signal state is kept.
    pub fn set(&mut self, words: [u32; 10], cutoff: u8, flags: u8) {
        self.words = words;
        self.flags = flags;
        self.set_cutoff(cutoff);
    }

    /// Jump x to `cutoff`.
    pub fn set_cutoff(&mut self, cutoff: u8) {
        self.cutoff = cutoff;
        self.interp.jump(cutoff);
        self.apply();
    }

    /// Let x move from where it is to `cutoff` in about 32 << k samples
    /// (FIG. 11). The codes follow x sample by sample.
    pub fn sweep_to(&mut self, cutoff: u8, k: u8) {
        self.cutoff = cutoff;
        self.interp.set_target(cutoff, k);
    }

    /// The current interpolating variable.
    pub fn x(&self) -> u16 {
        self.interp.x
    }

    /// Interpolate the codes at the current x and load the three sections
    /// (the coefficient math unit, four codes per section).
    fn apply(&mut self) {
        let x = self.interp.x;
        let gain_mode = self.flags & FLAG_GAIN != 0;
        let unnorm = self.flags & FLAG_UNNORMALISED != 0;
        for k in 0..3 {
            let (px0, pf0, px255, pf255) = word_codes(self.words[POLE_WORDS[k]]);
            let b2 = interp_x(px0, px255, x);
            let b1 = interp_x(pf0, pf255, x);
            let third_is_gain = k == 2 && gain_mode;
            let (a2, a1) = if third_is_gain {
                ((CODE_ONE_X as u16) << 8, (CODE_ONE_F as u16) << 8)
            } else {
                let (zx0, zf0, zx255, zf255) = word_codes(self.words[ZERO_WORDS[k]]);
                (interp_x(zx0, zx255, x), interp_x(zf0, zf255, x))
            };
            let s = &mut self.sections[k];
            s.dc = !(k > 0 && unnorm);
            s.set_codes_q8(b2, b1, a2, a1);
            if third_is_gain {
                let (_, g0, _, g255) = word_codes(self.words[9]);
                s.set_gain_q8(interp_x(g0, g255, x));
            } else {
                s.with_gain = false;
            }
        }
        self.applied_x = x;
    }

    /// One sample: update x (FIG. 10), re-interpolate the codes if it
    /// moved, then run the three sections.
    pub fn process(&mut self, x: i64) -> i64 {
        self.interp.update();
        if self.interp.x != self.applied_x {
            self.apply();
        }
        let mut v = x;
        for s in self.sections.iter_mut() {
            v = s.process(v);
        }
        v
    }

    /// Frequency response of the whole voice from the effective
    /// coefficients of its sections.
    pub fn ideal_response(&self, w: f64) -> Complex {
        self.sections
            .iter()
            .fold(Complex::new(1.0, 0.0), |h, s| h * s.ideal_response(w))
    }

    /// Response at the delay memory of section `k`: the sections before it
    /// in full, section `k` itself without its gain rows (the delay memory
    /// holds Yn before the gain).
    pub fn partial_response(&self, k: usize, w: f64) -> Complex {
        let before = self.sections[..k]
            .iter()
            .fold(Complex::new(1.0, 0.0), |h, s| h * s.ideal_response(w));
        before * self.sections[k].response(w, false)
    }

    /// Largest magnitude seen at any delay memory over the band, so a test
    /// input can be scaled to keep every delay memory in range.
    pub fn max_internal_gain(&self) -> f64 {
        let mut m = 0.0f64;
        for i in 0..=1200 {
            let w = std::f64::consts::PI * i as f64 / 1200.0;
            for k in 0..3 {
                m = m.max(self.partial_response(k, w).abs());
            }
        }
        m
    }

    pub fn reset_state(&mut self) {
        for s in self.sections.iter_mut() {
            s.reset_state();
        }
    }

    pub fn stats(&self) -> [Stats; 3] {
        [
            self.sections[0].stats,
            self.sections[1].stats,
            self.sections[2].stats,
        ]
    }

    /// Datapath cycles one sample costs with the standalone programs.
    pub fn cycles_per_sample(&self) -> usize {
        self.sections.iter().map(|s| s.program().len()).sum()
    }
}

/// The stock filter types of VOICE_INTERFACE.md: (words, flags).
pub fn stock_type(name: &str, q: u8) -> Option<([u32; 10], u8)> {
    let mut w = [NEUTRAL_WORD; 10];
    match name {
        "lp2" => {
            let (w0, w9) = match q {
                0 => (0x8408_FFCF, 0x00FE_00FE),
                64 => (0x6A07_E6E3, 0x00F6_00FE),
                127 => (0x5205_D4F3, 0x00EF_00FE),
                _ => return None,
            };
            w[0] = w0;
            w[9] = w9;
            Some((w, FLAG_GAIN))
        }
        "lp6" if q == 64 => {
            w[0] = 0x7008_ECDE;
            w[4] = 0x7008_ECDE;
            w[8] = 0x7008_ECDE;
            w[9] = 0x00EC_00FE;
            Some((w, FLAG_GAIN))
        }
        "hp4" if q == 64 => {
            w[1] = 0x8004_8004;
            w[5] = 0x8004_8004;
            w[4] = 0x7004_EFCF;
            w[8] = 0x7004_EFCF;
            w[9] = 0x00FE_00FE;
            Some((w, FLAG_GAIN | FLAG_UNNORMALISED))
        }
        _ => None,
    }
}
