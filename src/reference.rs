//! A straight-line statement of what the reservation table computes for one
//! sample, using the same word widths and the same product scaling as the
//! datapath but none of its muxes, latches or pipeline. The datapath tests
//! check the cycle-level emulator against this.

use crate::bits::{sat, wrap};
use crate::coef::BlockFloat;
use crate::mult::scale_product;
use crate::section::Params;

/// Result of one sample: the value written to the delay memory and the
/// section output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefOut {
    pub y_stored: i64,
    pub out: i64,
}

/// `coefs` are [B2', B1', A2', A1'] as the decoder delivers them.
pub fn reference_sample(
    p: &Params,
    dc: bool,
    x: i64,
    z1: i64,
    z2: i64,
    coefs: [BlockFloat; 4],
) -> RefOut {
    let mp = p.mult();
    let prod = |operand: i64, c: BlockFloat| scale_product(operand * c.mant as i64, c.exp, &mp);
    let [b2, b1, a2, a1] = coefs;

    let x = wrap(x, p.input_bits); // row 0: ACC 10 <- input
    let g = if dc { x } else { 0 }; // "(SA = Xn) & DC"

    // rows 1, 2: multiplier operands
    let op_b2 = p.operand(p.acc_fix(g - z2));
    let op_b1 = p.operand(p.acc_fix(g - z1));
    // row 3
    let two_z1_minus_z2 = p.acc_fix(2 * z1 - z2);
    // row 4: -AR2 = b2'(op) + ((SA = Xn') & IDC) + IDC
    let neg_ar2 = p.acc_fix(prod(op_b2, b2) + if dc { 0 } else { -x });
    // row 5: AR = b1'(op) + AR2
    let ar = p.acc_fix(prod(op_b1, b1) - neg_ar2);
    // row 6: Yn = (2Z1 - Z2) + AR
    let yn = p.acc_fix(two_z1_minus_z2 + ar);
    // rows 7, 8: -MA1 = a2'Z2 - AR; MAout = a1'Z1 + MA1
    let neg_ma1 = p.acc_fix(prod(p.operand(z2), a2) - ar);
    let out = p.acc_fix(prod(p.operand(z1), a1) - neg_ma1);

    RefOut {
        y_stored: sat(yn, p.delay_bits),
        out,
    }
}

/// The same recursion in floating point with unquantised products, for
/// checking the fixed-point model's accuracy rather than its wiring.
pub fn reference_f64(dc: bool, x: f64, z1: f64, z2: f64, coefs: [f64; 4]) -> (f64, f64) {
    let [b2, b1, a2, a1] = coefs;
    let g = if dc { x } else { 0.0 };
    let ar = (if dc { 0.0 } else { x }) + b2 * (z2 - g) - b1 * (z1 - g);
    let yn = ar + 2.0 * z1 - z2;
    let out = ar + a1 * z1 - a2 * z2;
    (yn, out)
}
