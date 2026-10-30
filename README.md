# hchip-filter: emulator of the US 5,170,369 IIR filter section

A cycle-level emulator, written in Rust with no dependencies, of the digital
audio filter section in E-mu's patent US 5,170,369 (Rossum, "Dynamic digital
IIR audio filter", 1992), the filter block of the E-mu H-chip. It covers

- FIG. 6, the datapath: adder, four muxes, two accumulators, delay memory
  latches, multiplier input latches;
- FIG. 7, the block floating point multiplier as a three-latch pipeline;
- FIG. 8, the coefficient decoder with the difference-coded first
  coefficient;
- the reservation table as microcode: eight control words plus a load state.

The filter is a second-order all-pole section in the patent's transformed
form, Yn = Xn + 2Z1 − Z2 − B1'Z1 + B2'Z2 (FIG. 3), with the optional
unity-DC-gain form of FIG. 5 selected by a mode bit. The FIR half (A1', A2')
is executed by the microcode and set transparent for the lowpass.

## Documents

| File | Contents |
|---|---|
| `docs/NETLIST.md` | FIGs. 6, 7, 8 as instance/net lists, with the two lines missing from the FIG. 6 drawing added |
| `docs/CONTROL_SIGNALS.md` | the control word, the microcode table, and how each field was derived from the reservation table |
| `docs/WIDTHS_AND_SATURATION.md` | word widths, saturation points, what was measured when they were varied |
| `docs/VERIFICATION.md` | the lowpass verification: method, traces, measured responses |
| `docs/PROBLEMS.md` | the problems met while building it and what was decided about each |
| `PATENT_US5170369.md`, `patent_figs/` | transcription of the patent and crops of its figures |
| `emupatent.pdf` | the patent; `00634131.pdf` is Rossum's ARMAdillo coefficient-encoding paper, `00482994.pdf` his filter-morphing paper |

## Layout

```
src/bits.rs        wrap / saturate / complement helpers
src/coef.rs        code table, block floating point coefficient, FIG. 8 decoder
src/mult.rs        FIG. 7 multiplier pipeline
src/section.rs     FIG. 6 datapath, Params (widths), Ctl (control word), Section::step
src/microcode.rs   the reservation table as control words
src/reference.rs   straight-line integer model of one sample, for the datapath tests
src/design.rs      lowpass design, ideal response, DFT
src/lib.rs         impulse_response, sine_gain
src/bin/hfilter.rs command line tool
tests/datapath.rs  emulator vs. reference, pure-two-pole identity, worst-case bound
tests/lowpass.rs   response measurements against the ideal response
```

## Use

```
cargo test --release
cargo run --release -- microcode                       # the control words
cargo run --release -- trace --x 10000 --samples 3     # per-cycle bus values
cargo run --release -- response --fc 1000 --q 0.7      # DC-off lowpass vs ideal
cargo run --release -- response --dc --fc 1000 --q 10  # DC-on resonant
cargo run --release -- sine --dc --f 1000 --sine-amp 30000 --mult-bits 17
cargo run --release -- coefs                           # code table
cargo run --release -- impulse --dc --fc 1000 > h.csv
```

Width options accepted by every command: `--acc-bits B --acc-sat
--delay-bits B --mult-bits B --drop N --mant-bits B --exact-sum --round`.
In code the same settings are fields of `Params`, for example
`Params { acc_bits: 20, ..Params::default() }` for a 20-bit accumulator.

Library use:

```rust
use hchip_filter::{Params, Section};
let mut s = Section::new(Params::default());
s.dc = true;                              // FIG. 5 form, unity DC gain
s.set_coefs(0.1719, 0.1875, 1.0, 2.0);    // B2', B1', A2', A1' (A = 1, 2: no zeros)
let y = s.process(x);                     // one sample, nine cycles
```

## Decisions not in the patent

- Coefficient codes decode by the hypothesised hidden-bit table in
  `coef::decode_linear`, scaled so that code 255 is 4.0 and 1.0 is 2^16.
- B1' and A1' are stored as differences from B2' and A2' and added in the
  decoder, as FIG. 8 draws and the ARMAdillo paper states.
- Widths: 24-bit accumulator and input, 16-bit saturating delay memory,
  16-bit saturating multiplier operand, 16-bit coefficient mantissa,
  floor-truncated product. See `docs/WIDTHS_AND_SATURATION.md`.
- The multiplier latency is three cycles (operand latched at the end of row
  k, product consumed in row k+3); the reservation table's "Prod" column is
  off by one for the A products and was not followed literally.
- A load state (row 0) precedes the eight rows for a lone section.
