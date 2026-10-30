//! FIG. 6: one filter section, cycle by cycle.
//!
//! `Section::step` executes one control word: it evaluates the combinational
//! paths (muxes, adder, gate) from the current latch contents, then applies
//! the clock edge (accumulators, latches, multiplier pipeline, delay
//! memory). `Section::process` runs the microcode for one sample.

use crate::bits::{not, overflows, sat, wrap};
use crate::coef::{encode_f64, BlockFloat, CoefKind, Decoder};
use crate::microcode::MICROCODE;
use crate::mult::{MultParams, Multiplier, Rounding};

/// Word widths and overflow behaviour. None of these are in the patent; see
/// docs/WIDTHS_AND_SATURATION.md for the reasoning behind the defaults.
#[derive(Clone, Copy, Debug)]
pub struct Params {
    /// Width of ADDER 12, ACC 10, ACC 11, LATCH 38 and the busses between
    /// them. The LSB is the same as the delay memory's.
    pub acc_bits: u32,
    /// Clamp (true) or wrap (false) when an adder result does not fit.
    pub acc_saturate: bool,
    /// Width of the INPUT SIGNAL bus; sign-extended into ACC 10.
    pub input_bits: u32,
    /// Width of a DELAY MEMORY 22 word. Writes saturate.
    pub delay_bits: u32,
    /// Width of LATCH 18, the multiplier's signed operand, after dropping
    /// `mult_lsb_drop` low bits.
    pub mult_in_bits: u32,
    /// Clamp (true) or wrap (false) an operand that does not fit LATCH 18.
    pub mult_in_saturate: bool,
    /// Low bits removed from the operand before the multiply ("a truncation
    /// of said first and second output delays", claim 1).
    pub mult_lsb_drop: u32,
    /// Width of the coefficient mantissa (LATCH 19).
    pub mant_bits: u32,
    /// Exact or aligned addition in the FIG. 8 decoder; see `Decoder`.
    pub exact_coef_sum: bool,
    pub product_round: Rounding,
    /// Clamp (true) or wrap (false) a product that does not fit LATCH 38.
    pub product_saturate: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            acc_bits: 24,
            acc_saturate: false,
            input_bits: 24,
            delay_bits: 16,
            mult_in_bits: 16,
            mult_in_saturate: true,
            mult_lsb_drop: 0,
            mant_bits: 16,
            exact_coef_sum: false,
            product_round: Rounding::Floor,
            product_saturate: true,
        }
    }
}

impl Params {
    pub fn mult(&self) -> MultParams {
        MultParams {
            lsb_drop: self.mult_lsb_drop,
            round: self.product_round,
            product_bits: self.acc_bits,
            product_saturate: self.product_saturate,
        }
    }

    /// Bring an adder result to accumulator width.
    pub fn acc_fix(&self, v: i64) -> i64 {
        if self.acc_saturate {
            sat(v, self.acc_bits)
        } else {
            wrap(v, self.acc_bits)
        }
    }

    /// Form the multiplier operand (LATCH 18) from a bus value.
    pub fn operand(&self, v: i64) -> i64 {
        let full = self.mult_in_bits + self.mult_lsb_drop;
        let v = if self.mult_in_saturate {
            sat(v, full)
        } else {
            wrap(v, full)
        };
        v >> self.mult_lsb_drop
    }
}

// Control word fields. Names follow docs/NETLIST.md.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mux15 {
    /// LATCH 13 complement: -Z2 - 1
    Z2N,
    /// LATCH 20 complement: -Z1 - 1
    Z1N,
    Acc11,
    Prod,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mux16 {
    /// line 21: Z1 << 1
    Z1x2,
    Acc10,
    /// ACC 10 complement
    Acc10N,
}

/// The "& DC" / "& IDC" terms of the reservation table: when the named mode
/// bit is false, adder input 17 is zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    Always,
    Dc,
    Idc,
}

/// The "Adder CD" column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cin {
    Zero,
    One,
    Idc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acc10 {
    Hold,
    Adder,
    Input,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acc11 {
    Hold,
    Adder,
    /// The direct MUX 16 -> MUX11 path (not drawn in FIG. 6).
    Mux16,
}

/// The unnumbered mux in front of LATCH 18.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MuxM {
    Adder,
    Z2,
    Z1,
}

/// Coefficient memory slot. B1 and A1 hold difference codes (FIG. 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coef {
    B2 = 0,
    B1 = 1,
    A2 = 2,
    A1 = 3,
}

impl Coef {
    pub fn kind(self) -> CoefKind {
        match self {
            Coef::B2 | Coef::A2 => CoefKind::Second,
            Coef::B1 | Coef::A1 => CoefKind::First,
        }
    }
}

/// One microcode word: everything the datapath needs for one cycle.
#[derive(Clone, Copy, Debug)]
pub struct Ctl {
    pub name: &'static str,
    pub mux15: Mux15,
    pub mux16: Mux16,
    pub gate: Gate,
    pub cin: Cin,
    pub acc10: Acc10,
    pub acc11: Acc11,
    pub mux_m: MuxM,
    /// Load LATCH 18 (from MuxM) and LATCH 19 (with this coefficient) at
    /// the end of the cycle.
    pub load18: Option<Coef>,
    /// Write ACC 11 into DELAY MEMORY 22 at the end of the cycle.
    pub dmem_write: bool,
    /// Load LATCH 13 and LATCH 20 from DELAY MEMORY 22 at the end of the
    /// cycle.
    pub latch_z: bool,
}

/// Bus values during one cycle, for traces and tests.
#[derive(Clone, Copy, Debug, Default)]
pub struct Trace {
    pub name: &'static str,
    pub a14: i64,
    pub a17: i64,
    pub cin: i64,
    pub sum: i64,
    /// Register contents during the cycle (before the edge).
    pub acc10: i64,
    pub acc11: i64,
    pub prod: i64,
    pub latch18: i64,
    pub z1: i64,
    pub z2: i64,
}

/// Counters of the places where a value was clipped or would have wrapped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub delay_sat: u64,
    pub acc_overflow: u64,
    pub mult_in_sat: u64,
    pub product_sat: u64,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub p: Params,
    /// The DC gain stabilisation mode bit (FIG. 5 form when true).
    pub dc: bool,
    /// INPUT SIGNAL bus.
    pub input: i64,
    pub acc10: i64,
    pub acc11: i64,
    /// LATCH 13 (Y[n-2]) and LATCH 20 (Y[n-1]), sign-extended.
    pub latch13: i64,
    pub latch20: i64,
    /// DELAY MEMORY 22 words for this section: [Y[n-1], Y[n-2]].
    pub dmem: [i64; 2],
    /// COEF. MEMORY 23: codes for B2', B1'-B2', A2', A1'-A2'.
    pub coef_mem: [u8; 4],
    pub decoder: Decoder,
    pub mult: Multiplier,
    pub stats: Stats,
}

impl Section {
    pub fn new(p: Params) -> Self {
        Section {
            p,
            dc: false,
            input: 0,
            acc10: 0,
            acc11: 0,
            latch13: 0,
            latch20: 0,
            dmem: [0; 2],
            coef_mem: [0; 4],
            decoder: Decoder::new(p.mant_bits, p.exact_coef_sum),
            mult: Multiplier::default(),
            stats: Stats::default(),
        }
    }

    /// Clear the signal state (not the coefficients or parameters).
    pub fn reset_state(&mut self) {
        self.input = 0;
        self.acc10 = 0;
        self.acc11 = 0;
        self.latch13 = 0;
        self.latch20 = 0;
        self.dmem = [0; 2];
        self.decoder = Decoder::new(self.p.mant_bits, self.p.exact_coef_sum);
        self.mult = Multiplier::default();
        self.stats = Stats::default();
    }

    /// Load raw codes: B2', the B1' difference, A2', the A1' difference.
    pub fn set_codes(&mut self, b2: u8, b1_diff: u8, a2: u8, a1_diff: u8) {
        self.coef_mem = [b2, b1_diff, a2, a1_diff];
    }

    /// Encode real coefficients (nearest code). `b1p` and `a1p` are the
    /// full first coefficients; their differences to `b2p`/`a2p` are stored.
    pub fn set_coefs(&mut self, b2p: f64, b1p: f64, a2p: f64, a1p: f64) {
        self.set_codes(
            encode_f64(b2p),
            encode_f64(b1p - b2p),
            encode_f64(a2p),
            encode_f64(a1p - a2p),
        );
    }

    /// The coefficients the multiplier will actually see, in microcode
    /// order: [B2', B1', A2', A1'].
    pub fn effective_coefs(&self) -> [BlockFloat; 4] {
        let mut d = Decoder::new(self.p.mant_bits, self.p.exact_coef_sum);
        [Coef::B2, Coef::B1, Coef::A2, Coef::A1]
            .map(|c| d.decode(self.coef_mem[c as usize], c.kind()))
    }

    /// Same, as real numbers.
    pub fn effective_coefs_f64(&self) -> [f64; 4] {
        self.effective_coefs().map(|c| c.to_f64())
    }

    /// Execute one control word.
    pub fn step(&mut self, ctl: &Ctl) -> Trace {
        let p = self.p;
        let z2 = self.latch13;
        let z1 = self.latch20;

        // MUX 15 -> adder input 14
        let a14 = match ctl.mux15 {
            Mux15::Z2N => not(z2, p.acc_bits),
            Mux15::Z1N => not(z1, p.acc_bits),
            Mux15::Acc11 => self.acc11,
            Mux15::Prod => self.mult.product(),
        };
        // MUX 16 and its gate -> adder input 17
        let m16 = match ctl.mux16 {
            Mux16::Z1x2 => wrap(z1 << 1, p.acc_bits),
            Mux16::Acc10 => self.acc10,
            Mux16::Acc10N => not(self.acc10, p.acc_bits),
        };
        let pass = match ctl.gate {
            Gate::Always => true,
            Gate::Dc => self.dc,
            Gate::Idc => !self.dc,
        };
        let a17 = if pass { m16 } else { 0 };
        let cin = match ctl.cin {
            Cin::Zero => 0,
            Cin::One => 1,
            Cin::Idc => (!self.dc) as i64,
        };
        // ADDER 12
        let raw = a14 + a17 + cin;
        if overflows(raw, p.acc_bits) {
            self.stats.acc_overflow += 1;
        }
        let sum = p.acc_fix(raw);
        // MUXM -> LATCH 18 data
        let mm = match ctl.mux_m {
            MuxM::Adder => sum,
            MuxM::Z2 => z2,
            MuxM::Z1 => z1,
        };

        let trace = Trace {
            name: ctl.name,
            a14,
            a17,
            cin,
            sum,
            acc10: self.acc10,
            acc11: self.acc11,
            prod: self.mult.product(),
            latch18: self.mult.latch18,
            z1,
            z2,
        };

        // Clock edge.
        let load = ctl.load18.map(|c| {
            let operand = p.operand(mm);
            if operand != mm >> p.mult_lsb_drop {
                self.stats.mult_in_sat += 1;
            }
            let coef = self.decoder.decode(self.coef_mem[c as usize], c.kind());
            (operand, coef)
        });
        let full_product = crate::mult::scale_product(
            self.mult.latch35,
            self.mult.latch34,
            &MultParams {
                product_bits: 62,
                product_saturate: false,
                ..p.mult()
            },
        );
        if overflows(full_product, p.acc_bits) {
            self.stats.product_sat += 1;
        }
        self.mult.clock(load, &p.mult());
        match ctl.acc10 {
            Acc10::Hold => {}
            Acc10::Adder => self.acc10 = sum,
            Acc10::Input => self.acc10 = wrap(self.input, p.input_bits),
        }
        match ctl.acc11 {
            Acc11::Hold => {}
            Acc11::Adder => self.acc11 = sum,
            Acc11::Mux16 => self.acc11 = a17,
        }
        if ctl.dmem_write {
            let v = sat(self.acc11, p.delay_bits);
            if v != self.acc11 {
                self.stats.delay_sat += 1;
            }
            self.dmem[1] = self.dmem[0];
            self.dmem[0] = v;
        }
        if ctl.latch_z {
            self.latch20 = self.dmem[0];
            self.latch13 = self.dmem[1];
        }
        trace
    }

    /// Run the whole microcode for one input sample and return the section
    /// output (ACC 10 after the last cycle).
    pub fn process(&mut self, x: i64) -> i64 {
        self.input = x;
        for ctl in MICROCODE.iter() {
            self.step(ctl);
        }
        self.acc10
    }

    /// Like `process`, but returns the per-cycle traces too.
    pub fn process_traced(&mut self, x: i64) -> (i64, Vec<Trace>) {
        self.input = x;
        let traces = MICROCODE.iter().map(|ctl| self.step(ctl)).collect();
        (self.acc10, traces)
    }
}
