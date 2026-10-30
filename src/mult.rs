//! FIG. 7: the block floating point multiplier 24, as a three-latch pipeline.
//!
//! Stage 32 multiplies the signed operand in LATCH 18 by the unsigned
//! mantissa in LATCH 19. LATCH 35 holds that product while LATCH 34 holds
//! the exponent; stage 36 completes the product and BARREL SHIFTER 37 scales
//! it by the exponent, and LATCH 38 holds the result for MUX 15.
//!
//! The shift combines the coefficient exponent with the fixed realignment
//! from the coefficient's `FRAC_BITS` fractional bits, so a coefficient of
//! exactly 1.0 (mantissa 16, exponent 12) multiplies by one.

use crate::bits::{sat, wrap};
use crate::coef::{BlockFloat, FRAC_BITS};

/// How BARREL SHIFTER 37 drops the bits it shifts out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    /// Arithmetic right shift: round toward minus infinity.
    Floor,
    /// Add a half before shifting: round to nearest, ties up.
    Nearest,
}

#[derive(Clone, Copy, Debug)]
pub struct MultParams {
    /// LSBs the operand had dropped before the multiply (see
    /// `Params::mult_lsb_drop`); the shifter compensates for them.
    pub lsb_drop: u32,
    pub round: Rounding,
    /// Width of LATCH 38.
    pub product_bits: u32,
    /// Clamp (true) or wrap (false) a product that does not fit LATCH 38.
    pub product_saturate: bool,
}

/// Stage 36 + BARREL SHIFTER 37: scale a raw operand*mantissa product by
/// the exponent and realign to the data's binary point.
pub fn scale_product(raw: i64, exp: i32, p: &MultParams) -> i64 {
    let shift = FRAC_BITS as i64 - exp as i64 - p.lsb_drop as i64;
    let v = if shift > 0 {
        match p.round {
            Rounding::Floor => raw >> shift,
            Rounding::Nearest => (raw + (1i64 << (shift - 1))) >> shift,
        }
    } else {
        raw << (-shift)
    };
    if p.product_saturate {
        sat(v, p.product_bits)
    } else {
        wrap(v, p.product_bits)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Multiplier {
    /// LATCH 18: signed operand.
    pub latch18: i64,
    /// LATCH 19 (mantissa) together with the exponent from LATCH 50.
    pub latch19: BlockFloat,
    /// LATCH 35: operand times mantissa.
    pub latch35: i64,
    /// LATCH 34: exponent travelling with LATCH 35.
    pub latch34: i32,
    /// LATCH 38: the product.
    pub latch38: i64,
}

impl Multiplier {
    /// Value on the MULTIPLIER PRODUCT net during the current cycle.
    pub fn product(&self) -> i64 {
        self.latch38
    }

    /// Clock edge at the end of a cycle. All latches update together;
    /// LATCH 18/19 take `load` if given, else keep their content.
    pub fn clock(&mut self, load: Option<(i64, BlockFloat)>, p: &MultParams) {
        self.latch38 = scale_product(self.latch35, self.latch34, p);
        self.latch35 = self.latch18 * self.latch19.mant as i64;
        self.latch34 = self.latch19.exp;
        if let Some((operand, coef)) = load {
            self.latch18 = operand;
            self.latch19 = coef;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> MultParams {
        MultParams {
            lsb_drop: 0,
            round: Rounding::Floor,
            product_bits: 24,
            product_saturate: true,
        }
    }

    #[test]
    fn unity_and_two_are_exact() {
        use crate::coef::{Decoder, Role, CODE_ONE_F, CODE_ONE_X};
        let mut dec = Decoder::new(16, false);
        let one = dec.decode((CODE_ONE_X as u16) << 8, Role::Second);
        let two = dec.decode((CODE_ONE_F as u16) << 8, Role::First);
        for v in [-32768i64, -1, 0, 1, 12345, 32767] {
            assert_eq!(scale_product(v * one.mant as i64, one.exp, &p()), v);
            assert_eq!(scale_product(v * two.mant as i64, two.exp, &p()), 2 * v);
        }
    }

    #[test]
    fn latency_is_three_cycles() {
        let mut m = Multiplier::default();
        let one = BlockFloat::from_linear(1 << FRAC_BITS, 16);
        m.clock(Some((1000, one)), &p()); // end of cycle 1
        m.clock(None, &p()); // end of cycle 2
        assert_ne!(m.product(), 1000);
        m.clock(None, &p()); // end of cycle 3
        assert_eq!(m.product(), 1000); // usable in cycle 4
    }
}
