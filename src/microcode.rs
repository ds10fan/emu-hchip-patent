//! The reservation table of US 5,170,369 (columns 5 and 6) as control words.
//!
//! Rows 1 to 8 are the patent's eight states. Row 0 is the load state the
//! text describes in "At the beginning of the cycle, accumulator 10 is
//! loaded with the input signal" (4:13); in a chip that multiplexes many
//! sections it coincides with the previous section's row 8, whose result
//! "is the filter output and hence the input to the next stage" (4:63).
//! Here it also loads LATCH 13 and LATCH 20 from the delay memory.
//!
//! Rows 1 to 6 compute the poles (Yn into the delay memory); rows 7 and 8
//! compute the zeros on the same delay values (direct form II). A section
//! without zeros runs the same rows with A2' = 1, A1' = 2.
//!
//! `MICROCODE_GAIN` appends an output gain, which the patent does not have:
//! row 8 also latches MAout (the adder output) into LATCH 18 with the gain
//! coefficient, rows 9 and 10 wait for the multiplier pipeline, and row 11
//! passes the product through the adder into ACC 10. In a pipelined chip
//! rows 9 and 10 would overlap the next section's rows.
//!
//! In row 0 the adder-side fields are don't-cares and repeat row 1.

use crate::section::{Acc10, Acc11, Cin, Coef, Ctl, Gate, Mux15, Mux16, MuxM};

const ROW0: Ctl = Ctl {
    name: "0 load",
    mux15: Mux15::Z2N,
    mux16: Mux16::Acc10,
    gate: Gate::Dc,
    cin: Cin::One,
    acc10: Acc10::Input,
    acc11: Acc11::Hold,
    mux_m: MuxM::Adder,
    load18: None,
    dmem_write: false,
    latch_z: true,
};

// Z2' + (SA = Xn) & DC + 1 -> LATCH 18, coefficient B2'
const ROW1: Ctl = Ctl {
    name: "1",
    mux15: Mux15::Z2N,
    mux16: Mux16::Acc10,
    gate: Gate::Dc,
    cin: Cin::One,
    acc10: Acc10::Hold,
    acc11: Acc11::Hold,
    mux_m: MuxM::Adder,
    load18: Some(Coef::B2),
    dmem_write: false,
    latch_z: false,
};

// Z1' + (SA = Xn) & DC + 1 -> LATCH 18, coefficient B1'
const ROW2: Ctl = Ctl {
    name: "2",
    mux15: Mux15::Z1N,
    mux16: Mux16::Acc10,
    gate: Gate::Dc,
    cin: Cin::One,
    acc10: Acc10::Hold,
    acc11: Acc11::Hold,
    mux_m: MuxM::Adder,
    load18: Some(Coef::B1),
    dmem_write: false,
    latch_z: false,
};

// Z2' + 2Z1 + 1 = 2Z1 - Z2 -> ACC 11
const ROW3: Ctl = Ctl {
    name: "3",
    mux15: Mux15::Z2N,
    mux16: Mux16::Z1x2,
    gate: Gate::Always,
    cin: Cin::One,
    acc10: Acc10::Hold,
    acc11: Acc11::Adder,
    mux_m: MuxM::Adder,
    load18: None,
    dmem_write: false,
    latch_z: false,
};

// b2'(..) + (SA = Xn') & IDC + IDC = -AR2 -> ACC 10; Z2 -> LATCH 18, coefficient A2'
const ROW4: Ctl = Ctl {
    name: "4",
    mux15: Mux15::Prod,
    mux16: Mux16::Acc10N,
    gate: Gate::Idc,
    cin: Cin::Idc,
    acc10: Acc10::Adder,
    acc11: Acc11::Hold,
    mux_m: MuxM::Z2,
    load18: Some(Coef::A2),
    dmem_write: false,
    latch_z: false,
};

// b1'(..) + (-AR2)' + 1 = AR -> ACC 10; Z1 -> LATCH 18, coefficient A1'
const ROW5: Ctl = Ctl {
    name: "5",
    mux15: Mux15::Prod,
    mux16: Mux16::Acc10N,
    gate: Gate::Always,
    cin: Cin::One,
    acc10: Acc10::Adder,
    acc11: Acc11::Hold,
    mux_m: MuxM::Z1,
    load18: Some(Coef::A1),
    dmem_write: false,
    latch_z: false,
};

// (2Z1 - Z2) + AR + 0 = Yn -> ACC 11
const ROW6: Ctl = Ctl {
    name: "6",
    mux15: Mux15::Acc11,
    mux16: Mux16::Acc10,
    gate: Gate::Always,
    cin: Cin::Zero,
    acc10: Acc10::Hold,
    acc11: Acc11::Adder,
    mux_m: MuxM::Adder,
    load18: None,
    dmem_write: false,
    latch_z: false,
};

// a2'Z2 + AR' + 1 = -MA1 -> ACC 10; Yn -> DELAY MEMORY
const ROW7: Ctl = Ctl {
    name: "7",
    mux15: Mux15::Prod,
    mux16: Mux16::Acc10N,
    gate: Gate::Always,
    cin: Cin::One,
    acc10: Acc10::Adder,
    acc11: Acc11::Hold,
    mux_m: MuxM::Adder,
    load18: None,
    dmem_write: true,
    latch_z: false,
};

// a1'Z1 + (-MA1)' + 1 = MAout -> ACC 10
const ROW8: Ctl = Ctl {
    name: "8",
    mux15: Mux15::Prod,
    mux16: Mux16::Acc10N,
    gate: Gate::Always,
    cin: Cin::One,
    acc10: Acc10::Adder,
    acc11: Acc11::Hold,
    mux_m: MuxM::Adder,
    load18: None,
    dmem_write: false,
    latch_z: false,
};

// Gain rows. Row 8 additionally latches MAout with the gain coefficient.
const ROW8G: Ctl = Ctl {
    name: "8g",
    load18: Some(Coef::G),
    ..ROW8
};

// Wait for the multiplier pipeline; everything holds.
const ROW9: Ctl = Ctl {
    name: "9",
    mux15: Mux15::Prod,
    mux16: Mux16::Acc10,
    gate: Gate::Never,
    cin: Cin::Zero,
    acc10: Acc10::Hold,
    acc11: Acc11::Hold,
    mux_m: MuxM::Adder,
    load18: None,
    dmem_write: false,
    latch_z: false,
};

const ROW10: Ctl = Ctl { name: "10", ..ROW9 };

// g * MAout + 0 + 0 -> ACC 10
const ROW11: Ctl = Ctl {
    name: "11",
    acc10: Acc10::Adder,
    ..ROW9
};

/// The patent's section: load state plus the eight rows.
pub static MICROCODE: [Ctl; 9] = [ROW0, ROW1, ROW2, ROW3, ROW4, ROW5, ROW6, ROW7, ROW8];

/// The section followed by an output gain.
pub static MICROCODE_GAIN: [Ctl; 12] = [
    ROW0, ROW1, ROW2, ROW3, ROW4, ROW5, ROW6, ROW7, ROW8G, ROW9, ROW10, ROW11,
];

/// Render control words as a text table.
pub fn table_of(rows: &[Ctl]) -> String {
    let mut s = String::new();
    s.push_str(
        "cycle | mux15 | mux16  | gate   | cin  | acc10 | acc11 | mux_m | load18 | dmem_w | latch_z\n",
    );
    s.push_str(
        "------|-------|--------|--------|------|-------|-------|-------|--------|--------|--------\n",
    );
    for c in rows {
        s.push_str(&format!(
            "{:<5} | {:<5} | {:<6} | {:<6} | {:<4} | {:<5} | {:<5} | {:<5} | {:<6} | {:<6} | {}\n",
            c.name,
            format!("{:?}", c.mux15),
            format!("{:?}", c.mux16),
            format!("{:?}", c.gate),
            format!("{:?}", c.cin),
            format!("{:?}", c.acc10),
            format!("{:?}", c.acc11),
            format!("{:?}", c.mux_m),
            c.load18.map(|c| format!("{:?}", c)).unwrap_or("-".into()),
            if c.dmem_write { "yes" } else { "-" },
            if c.latch_z { "yes" } else { "-" },
        ));
    }
    s
}

/// The patent's section as a text table.
pub fn table() -> String {
    table_of(&MICROCODE)
}
