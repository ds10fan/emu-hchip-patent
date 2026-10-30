//! Coefficient codes, their expansion (FIG. 8) and the block floating point
//! form the multiplier consumes (FIG. 7).
//!
//! The patent gives no code-to-value formula ("a ROM or a shifting
//! technique", 5:55). The table used here, `decode_linear`, was recovered
//! by measurement on the chip (VOICE_INTERFACE.md) and is an assumption of
//! this model: a hidden-bit mantissa shifted by a 4-bit exponent, every 16 codes
//! doubling the value, with the top range linear so that the value runs to
//! 2^18 at code 255.
//!
//! The same table serves three roles with two scalings (VOICE_INTERFACE.md):
//!
//! - the X byte of a word, decoded as B2' (or A2') = table / 2^18;
//! - the F byte, decoded as f² = table / 2^16, the difference B1' − B2'
//!   (or A1' − A2') that the decoder adds to the X value;
//! - the gain byte of word 9, taken as table / 2^18 here.
//!
//! One unit of a linear coefficient is therefore 2^-18 (`FRAC_BITS`).
//!
//! A code can carry 8 fraction bits (`u16`, code × 256): the interpolated
//! code of a dynamic filter. The fraction extends the mantissa, which makes
//! the expansion piecewise linear between adjacent integer codes.

/// Fixed-point scale of a linear coefficient: 1.0 == 1 << FRAC_BITS.
pub const FRAC_BITS: u32 = 18;

/// Code (as X) that decodes to exactly 1.0; the neutral X byte.
pub const CODE_ONE_X: u8 = 255;
/// Code (as F) that decodes to exactly 1.0; the neutral F byte.
pub const CODE_ONE_F: u8 = 207;

/// How a code is scaled when expanded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// X byte: a second coefficient, B2' or A2', table / 2^18.
    Second,
    /// F byte: the difference added to the held second coefficient, table / 2^16.
    First,
    /// Gain byte: table / 2^18, not added to anything.
    Gain,
}

impl Role {
    /// Left shift that brings the raw table value to 2^-18 units.
    pub fn shift(self) -> u32 {
        match self {
            Role::Second | Role::Gain => 0,
            Role::First => 2,
        }
    }
}

/// A coefficient as the multiplier sees it: unsigned mantissa and signed
/// exponent, value `mant * 2^exp` in units of 2^-FRAC_BITS. A negative
/// exponent means the mantissa carries bits below the coefficient unit
/// (fraction bits of an interpolated code).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BlockFloat {
    pub mant: u32,
    pub exp: i32,
}

impl BlockFloat {
    /// Whole units of 2^-FRAC_BITS (floor).
    pub fn linear(self) -> i64 {
        if self.exp >= 0 {
            (self.mant as i64) << self.exp
        } else {
            (self.mant as i64) >> (-self.exp)
        }
    }

    /// Value as a real number (1.0 == coefficient one), fraction included.
    pub fn to_f64(self) -> f64 {
        self.mant as f64 * 2f64.powi(self.exp) / (1u64 << FRAC_BITS) as f64
    }

    /// Normalise an unsigned linear value into at most `mant_bits` mantissa
    /// bits, truncating low bits if it does not fit.
    pub fn from_linear(v: i64, mant_bits: u32) -> Self {
        assert!(v >= 0, "coefficients are unsigned");
        BlockFloat { mant: 0, exp: 0 }
            .with(v as u64, 0)
            .normalised(mant_bits)
    }

    fn with(self, mant: u64, exp: i32) -> (u64, i32) {
        let _ = self;
        (mant, exp)
    }

    /// Drop low mantissa bits until the mantissa fits `mant_bits`.
    pub fn normalised(self, mant_bits: u32) -> Self {
        (self.mant as u64, self.exp).normalised(mant_bits)
    }
}

trait Norm {
    fn normalised(self, mant_bits: u32) -> BlockFloat;
}

impl Norm for (u64, i32) {
    fn normalised(self, mant_bits: u32) -> BlockFloat {
        let (mant, exp) = self;
        let bits = 64 - mant.leading_zeros();
        let drop = bits.saturating_sub(mant_bits);
        BlockFloat {
            mant: (mant >> drop) as u32,
            exp: exp + drop as i32,
        }
    }
}

/// The code table, as recovered. Unitless; see `Role` for the scaling.
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

/// The table for a code with 8 fraction bits (`code_q8` = code × 256), as
/// LOGIC 43 and BARREL SHIFTER 44 produce it: the mantissa is the hidden
/// bit, the four mantissa bits and the eight fraction bits together
/// (`mant_q8`), and the shift comes from the integer part. The value in
/// table units is `mant_q8 * 2^shift / 256`.
pub fn decode_q8_parts(code_q8: u16) -> (u32, i32) {
    let c = (code_q8 >> 8) as u32 + 1;
    let f = (code_q8 & 255) as u32;
    let e = c >> 4;
    let m = c & 15;
    if c >= 224 {
        (((c - 192) << 8) | f, 12)
    } else if e != 0 {
        (((16 + m) << 8) | f, e as i32 - 1)
    } else {
        ((m << 8) | f, 0)
    }
}

/// The table for a code with 8 fraction bits, in whole table units (floor).
pub fn decode_q8(code_q8: u16) -> i64 {
    let (mant_q8, shift) = decode_q8_parts(code_q8);
    ((mant_q8 as i64) << shift) >> 8
}

/// The table for a code with 8 fraction bits, as a real number of table
/// units (fraction kept).
pub fn decode_q8_f64(code_q8: u16) -> f64 {
    let (mant_q8, shift) = decode_q8_parts(code_q8);
    mant_q8 as f64 * 2f64.powi(shift) / 256.0
}

/// LOGIC 43 and BARREL SHIFTER 44 as mantissa and shift, for an integer
/// code.
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
            exp: e as i32 - 1,
        }
    } else {
        BlockFloat { mant: m, exp: 0 }
    }
}

/// Nearest integer code for a raw table value (ties to the lower code).
/// Design-time helper, not hardware.
pub fn encode_table(v: i64) -> u8 {
    let v = v.clamp(0, 1 << 18);
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

/// Nearest code for a real coefficient value in the given role.
pub fn encode_f64(x: f64, role: Role) -> u8 {
    let scale = (1u64 << (FRAC_BITS - role.shift())) as f64;
    encode_table((x * scale).round().clamp(0.0, (1u64 << 18) as f64) as i64)
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
        assert!((6..=24).contains(&mant_bits), "mantissa must hold 6 bits");
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

    /// Expand one code (with 8 fraction bits) in the given role. For
    /// `Role::First` the result is the held second coefficient plus the
    /// decoded difference.
    pub fn decode(&mut self, code_q8: u16, role: Role) -> BlockFloat {
        // LOGIC 43, BARREL SHIFTER 44, LATCH 45 / LATCH 46: the mantissa
        // keeps the code's fraction bits, the exponent accounts for them.
        let (mant_q8, shift) = decode_q8_parts(code_q8);
        let d = (mant_q8 as u64, shift + role.shift() as i32 - 8).normalised(self.mant_bits);
        let out = match role {
            Role::Second | Role::Gain => d,
            Role::First => {
                if self.exact_sum {
                    let e = d.exp.min(self.latch47.exp);
                    let sum = ((d.mant as u64) << (d.exp - e))
                        + ((self.latch47.mant as u64) << (self.latch47.exp - e));
                    (sum, e).normalised(self.mant_bits)
                } else {
                    let s = self.latch47;
                    let e = d.exp.max(s.exp);
                    let a = d.mant >> (e - d.exp); // BARREL SHIFTER 52
                    let b = s.mant >> (e - s.exp); // BARREL SHIFTER 53
                    let sum = a + b; // ADDER 51, at most mant_bits + 1 bits
                    (sum as u64, e).normalised(self.mant_bits)
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
    fn decode_code_matches_table() {
        for code in 0..=255u8 {
            assert_eq!(
                decode_code(code).linear(),
                decode_linear(code),
                "code {code}"
            );
            assert_eq!(
                decode_q8((code as u16) << 8),
                decode_linear(code),
                "code {code}"
            );
        }
    }

    #[test]
    fn table_is_monotonic_and_ends_at_2_pow_18() {
        let mut prev = -1;
        for code in 0..=255u8 {
            let v = decode_linear(code);
            assert!(v > prev, "code {code}");
            prev = v;
        }
        assert_eq!(decode_linear(0), 1);
        assert_eq!(decode_linear(CODE_ONE_F), 1 << 16);
        assert_eq!(decode_linear(CODE_ONE_X), 1 << 18);
    }

    #[test]
    fn fraction_interpolates_linearly_between_codes() {
        // Including across an exponent boundary (code 62 -> 63: 124 -> 128)
        // and into the linear top range.
        for code in [5u8, 62, 137, 222, 223, 240] {
            let lo = decode_linear(code);
            let hi = decode_linear(code + 1);
            for f in [0u16, 64, 128, 192] {
                let want = lo + (hi - lo) * f as i64 / 256;
                assert_eq!(decode_q8(((code as u16) << 8) | f), want, "code {code}.{f}");
            }
        }
        // the document's example: X = 137.12 -> B2' = 0.012755
        // (106 + 124 * 64 / 255 = 137.1216, quantised to 1/256)
        let q8 = (137 << 8) | (0.1216f64 * 256.0).round() as u16;
        let b2p = decode_q8_f64(q8) / (1u64 << 18) as f64;
        assert!((b2p - 0.012755).abs() < 2e-6, "{b2p}");
        let mut dec = Decoder::new(16, false);
        assert!((dec.decode(q8, Role::Second).to_f64() - b2p).abs() < 1e-12);
    }

    #[test]
    fn encode_round_trips() {
        for code in 0..=255u8 {
            assert_eq!(encode_table(decode_linear(code)), code);
        }
        assert_eq!(encode_f64(1.0, Role::Second), CODE_ONE_X);
        assert_eq!(encode_f64(1.0, Role::First), CODE_ONE_F);
        assert_eq!(encode_f64(4.0, Role::First), 255);
        assert_eq!(encode_f64(0.25, Role::Second), CODE_ONE_F);
    }

    #[test]
    fn roles_scale_as_documented() {
        // word 50405040: B2' = 0.001038, f^2 = 0.002075 (VOICE_INTERFACE.md)
        let mut dec = Decoder::new(16, false);
        let b2 = dec.decode(0x50 << 8, Role::Second).to_f64();
        let b1 = dec.decode(0x40 << 8, Role::First).to_f64();
        assert!((b2 - 0.001038).abs() < 1e-6, "{b2}");
        assert!((b1 - b2 - 0.002075).abs() < 1e-6, "{}", b1 - b2);
        // the neutral word FFCF: B2' = 1, B1' = 2
        let a2 = dec.decode(0xFF << 8, Role::Second).to_f64();
        let a1 = dec.decode(0xCF << 8, Role::First).to_f64();
        assert_eq!(a2, 1.0);
        assert_eq!(a1, 2.0);
    }

    #[test]
    fn fraction_bits_survive_in_the_mantissa() {
        // F = 62.22 -> 124.88 table units -> f^2 = 0.001905, not 124 units
        let q8 = (62 << 8) | (0.22f64 * 256.0).round() as u16;
        let mut dec = Decoder::new(16, false);
        let second = dec.decode(0, Role::Second).to_f64();
        let first = dec.decode(q8, Role::First);
        let f2 = first.to_f64() - second;
        assert!((f2 - 0.001905).abs() < 2e-6, "{f2}");
        assert!(first.exp < 0, "{first:?}");
    }

    #[test]
    fn sixteen_bit_mantissa_is_exact_for_single_codes() {
        for code in 0..=255u8 {
            for role in [Role::Second, Role::First] {
                let v = decode_linear(code) << role.shift();
                let d = BlockFloat::from_linear(v, 16);
                assert_eq!(d.linear(), v, "code {code}");
            }
        }
    }
}
