//! Coefficient representation and the FIG. 8 decoder.
//!
//! The patent gives no code-to-value formula ("a ROM or a shifting
//! technique", 5:55). The mapping used here is the hypothesis recovered from
//! the firmware that drives the chip (see `decode_linear`). It is a
//! hidden-bit octave-shift code: a 4-bit mantissa with an implied leading one
//! and a 4-bit shift, with the top range made linear so that 4.0 is reached.
//!
//! The decoded value is a block floating point number (mantissa, exponent),
//! which is what FIG. 7's multiplier consumes. One unit of coefficient equals
//! `1 << FRAC_BITS` in the linear value.

/// Fixed-point scale of a decoded linear coefficient: 1.0 == 1 << FRAC_BITS.
/// Chosen so that code 255 decodes to exactly 4.0, the upper limit of
/// B1' = B1 + 2 (B1 in [-2, 2]).
pub const FRAC_BITS: u32 = 16;

/// Largest decoded value, 4.0.
pub const MAX_LINEAR: i64 = 1 << (FRAC_BITS + 2);

/// A coefficient as the multiplier sees it: unsigned mantissa and exponent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BlockFloat {
    pub mant: u32,
    pub exp: u32,
}

impl BlockFloat {
    /// Linear value, `mant << exp`, in units of 2^-FRAC_BITS.
    pub fn linear(self) -> i64 {
        (self.mant as i64) << self.exp
    }

    /// Linear value as a real number (1.0 == coefficient one).
    pub fn to_f64(self) -> f64 {
        self.linear() as f64 / (1u64 << FRAC_BITS) as f64
    }

    /// Normalise an unsigned linear value into at most `mant_bits` mantissa
    /// bits, truncating low bits if it does not fit.
    pub fn from_linear(v: i64, mant_bits: u32) -> Self {
        assert!(v >= 0, "coefficients are unsigned");
        let bits = 64 - (v as u64).leading_zeros();
        let exp = bits.saturating_sub(mant_bits);
        BlockFloat {
            mant: (v >> exp) as u32,
            exp,
        }
    }
}

/// The hypothesised code-to-value mapping, written as it was recovered.
pub fn decode_linear(code: u8) -> i64 {
    let c = code as i64 + 1;
    let e = c >> 4;
    let m = c & 15;
    if c >= 224 {
        (c - 192) << 12
    } else if e != 0 {
        (16 + m) << (e - 1)
    } else {
        m
    }
}

/// LOGIC 43 and BARREL SHIFTER 44 of FIG. 8: the same mapping, split into
/// the mantissa and the shift that the hardware would produce.
pub fn decode_code(code: u8) -> BlockFloat {
    let c = code as u32 + 1;
    let e = c >> 4;
    let m = c & 15;
    if c >= 224 {
        BlockFloat {
            mant: c - 192,
            exp: 12,
        }
    } else if e != 0 {
        BlockFloat {
            mant: 16 + m,
            exp: e - 1,
        }
    } else {
        BlockFloat { mant: m, exp: 0 }
    }
}

/// Nearest code for a linear value (ties go to the lower code). This is a
/// design-time helper, not hardware.
pub fn encode_linear(v: i64) -> u8 {
    let v = v.clamp(0, MAX_LINEAR);
    let mut best = 0u8;
    let mut best_err = i64::MAX;
    for code in 0..=255u8 {
        let err = (decode_linear(code) - v).abs();
        if err < best_err {
            best_err = err;
            best = code;
        }
    }
    best
}

/// Nearest code for a real coefficient value in [0, 4].
pub fn encode_f64(x: f64) -> u8 {
    let v = (x * (1u64 << FRAC_BITS) as f64).round();
    encode_linear(v.clamp(0.0, MAX_LINEAR as f64) as i64)
}

/// Whether a coefficient is decoded as a value (second coefficient, B2' or
/// A2') or as a difference added to the previously decoded second
/// coefficient (first coefficient, B1' or A1').
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoefKind {
    Second,
    First,
}

/// The FIG. 8 decoder. `latch47` is the coefficient output latch together
/// with its exponent latch 50; it is the operand ADDER 51 adds to a decoded
/// difference.
#[derive(Clone, Debug)]
pub struct Decoder {
    /// Width of the mantissa delivered to the multiplier (LATCH 47 / 19).
    pub mant_bits: u32,
    /// If true, ADDER 51 adds the two linear values exactly and the sum is
    /// renormalised. If false (default), shifters 52 and 53 align both
    /// operands to the larger exponent first, dropping low bits of the
    /// smaller one, as block floating point hardware would.
    pub exact_sum: bool,
    latch47: BlockFloat,
}

impl Decoder {
    pub fn new(mant_bits: u32, exact_sum: bool) -> Self {
        assert!((6..=20).contains(&mant_bits), "mantissa must hold 6 bits");
        Decoder {
            mant_bits,
            exact_sum,
            latch47: BlockFloat::default(),
        }
    }

    /// Current content of LATCH 47 / LATCH 50.
    pub fn held(&self) -> BlockFloat {
        self.latch47
    }

    /// Decode one code. For `CoefKind::First` the result is the held second
    /// coefficient plus the decoded difference.
    pub fn decode(&mut self, code: u8, kind: CoefKind) -> BlockFloat {
        // LATCH 45 / LATCH 46
        let d = BlockFloat::from_linear(decode_code(code).linear(), self.mant_bits);
        let out = match kind {
            CoefKind::Second => d,
            CoefKind::First => {
                if self.exact_sum {
                    BlockFloat::from_linear(d.linear() + self.latch47.linear(), self.mant_bits)
                } else {
                    let s = self.latch47;
                    let e = d.exp.max(s.exp);
                    let a = d.mant >> (e - d.exp); // BARREL SHIFTER 52
                    let b = s.mant >> (e - s.exp); // BARREL SHIFTER 53
                    let sum = a + b; // ADDER 51, at most mant_bits + 1 bits
                    if sum >> self.mant_bits != 0 {
                        BlockFloat {
                            mant: sum >> 1,
                            exp: e + 1,
                        }
                    } else {
                        BlockFloat { mant: sum, exp: e }
                    }
                }
            }
        };
        self.latch47 = out;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_matches_formula() {
        for code in 0..=255u8 {
            assert_eq!(
                decode_code(code).linear(),
                decode_linear(code),
                "code {code}"
            );
        }
    }

    #[test]
    fn decode_is_monotonic_and_ends_at_four() {
        let mut prev = -1;
        for code in 0..=255u8 {
            let v = decode_linear(code);
            assert!(v > prev, "code {code}");
            prev = v;
        }
        assert_eq!(decode_linear(255), MAX_LINEAR);
        assert_eq!(decode_linear(0), 1);
        // 1.0 and 2.0 are exactly representable.
        assert_eq!(decode_linear(207), 1 << FRAC_BITS);
        assert_eq!(decode_linear(223), 2 << FRAC_BITS);
    }

    #[test]
    fn encode_round_trips() {
        for code in 0..=255u8 {
            assert_eq!(encode_linear(decode_linear(code)), code);
        }
        assert_eq!(encode_f64(1.0), 207);
        assert_eq!(encode_f64(2.0), 223);
        assert_eq!(encode_f64(4.0), 255);
    }

    #[test]
    fn six_bit_mantissa_is_exact() {
        for code in 0..=255u8 {
            let d = BlockFloat::from_linear(decode_linear(code), 6);
            assert_eq!(d.linear(), decode_linear(code), "code {code}");
            assert!(d.mant < 64);
        }
    }

    #[test]
    fn first_is_second_plus_difference() {
        let mut dec = Decoder::new(6, false);
        let b2 = dec.decode(207, CoefKind::Second); // 1.0
        assert_eq!(b2.to_f64(), 1.0);
        let b1 = dec.decode(207, CoefKind::First); // 1.0 + 1.0
        assert_eq!(b1.to_f64(), 2.0);
        // A small second coefficient plus a big difference: the aligned add
        // keeps the sum within one mantissa LSB of the exact sum.
        let s = dec.decode(10, CoefKind::Second).linear();
        let exact = s + decode_linear(200);
        let got = dec.decode(200, CoefKind::First);
        assert!(got.linear() <= exact && exact - got.linear() < (1 << got.exp));
    }
}
