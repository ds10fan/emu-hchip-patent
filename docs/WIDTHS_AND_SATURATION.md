# Word widths and saturation

The patent fixes almost no widths and describes no saturation in the audio
path. This file records what it does say, the choices made in the emulator,
why, and what the emulator measured when a choice was varied. Every width is
a field of `Params` (`src/section.rs`) and can be changed from the command
line (`hfilter --acc-bits --delay-bits --mult-bits --drop --mant-bits
--acc-sat --round --exact-sum`).

## What the patent says

| Item | Patent | Where |
|---|---|---|
| Linear coefficient | "16 bit coefficients are adequate for good frequency resolution" | abstract, 1:54 |
| Compressed coefficient code | "as few as 8 bits" | abstract, 1:63 |
| Decoder shift field | 4 bits | 5:64 |
| Multiplier data | "similar to the input and output data resolution"; the multiplier inputs are "a truncation of said first and second output delays", narrower than the delays | abstract, 1:56; claims 1 and 7 |
| Delay memory, accumulators, product, input, output | not stated | |
| Saturation, overflow | nothing for the audio path; only the interpolator clamps | 7:40-8:5 |

Given as prior knowledge for this work: the delay memories saturate; the
accumulator is wider than the delay memories, wide enough that input plus
delays does not saturate; the section input is wider than 16 bits and may be
as wide as the accumulator; the multiplier size is unknown.

## One binary point for the whole adder

ADDER 12 sums, in different cycles, the input, a sign-extended delay value,
twice a delay value, a product, and the two accumulators. Whatever the widths,
all of these must meet the adder with the same LSB. The emulator therefore
has a single data LSB: the delay memory, accumulators, product latch and
input all count in the same unit, and the accumulator's extra bits are
headroom above the delay's range, not extra precision below it.

The alternative, a delay memory that stores the top 16 bits of a wider
accumulator (a truncation on the way into the memory), would put the
recursion's quantisation noise at the delay write. The patent places the
noise at the multiplier input (3:36-48) and the claims put the truncation
there too, so that alternative is not modelled. Dropping low bits at the
multiplier input is modelled (`mult_lsb_drop`, below).

## Delay memory: 16 bits, saturating (`delay_bits = 16`)

Yn goes from ACC 11 into DELAY MEMORY 22 in row 7, and that is the only
place where the emulator clips to the delay width. Wrapping there would
corrupt the state of a recursive filter for many samples; saturating bounds
the error and lets the filter recover, which is why the memories were said
to saturate. 16 bits is the audio resolution of the era and the width that
makes the memory for 64 sections times the voice count affordable.

The DC-off form has a DC gain of 1/(B1' − B2') and saturates the delay with
ordinary inputs (the test `dc_off_full_scale_saturates_but_stays_bounded`);
see "Dynamic range of the DC-off form" below.

## Accumulators and adder: 24 bits, wrapping (`acc_bits = 24`, `acc_saturate = false`)

Bounds for one section with |Xn|, |Z1|, |Z2| ≤ 2^15 and the coefficients at
their maxima (B2', A2' ≤ 1 from an X code; B1', A1' ≤ 1 + 4 from an F
code), in units of 2^15:

| Row | Value | Bound |
|---|---|---|
| 3 | 2Z1 − Z2 | 3 |
| 4 | −AR2 = ±Xn + B2' Z2 | 2 |
| 5 | AR = AR2 + B1' Z1 | 7 |
| 6 | Yn = AR + 2Z1 − Z2 | 10 |
| 7 | −MA1 = A2' Z2 − AR | 8 |
| 8 | MAout = MA1 + A1' Z1 | 13 |

So 20 bits are enough for one section fed with a 16-bit-range input, and
"input + delays does not saturate" holds with four bits of headroom. An
adversarial run (`single_section_worst_case_fits_in_20_bits`) reaches the
row 5 bound of 7 × 2^15. For the pure-pole coefficients used in the
lowpass tests (A1' = 2, A2' = 1) the row 8 bound is 10 × 2^15.

The output of a section is not saturated and becomes the next section's
input, so a chain can present up to 13 × 2^15 to the next adder; 24 bits
absorb that with margin. The default accumulator wraps because the figure
shows a plain adder and because, within the bounds above, wrapping never
happens; `acc_saturate = true` is available and the `acc_overflow` counter
reports any excursion.

## Input: as wide as the accumulator (`input_bits = 24`)

"MAout = Xn": a section's input is the previous section's ACC 10, so the
input bus is the accumulator bus. The chip's external input is probably
narrower (16 to 18 bits); `input_bits` sign-extends whatever is given.

## Multiplier operand, LATCH 18: 16 bits, saturating (`mult_in_bits = 16`)

Two things happen at LATCH 18.

**Truncation (claims 1 and 7).** The claims say the multiplier sees a
truncation of the delays. The emulator can drop low bits of the operand
(`mult_lsb_drop`), with the barrel shifter compensating. Measured on the
1 kHz, Q = 10, DC-on section (sine measurement error against the ideal
response, in dB):

| Operand | 5 kHz | 10 kHz | 15 kHz | 20 kHz | 23 kHz |
|---|---|---|---|---|---|
| 16 bits | 0.02 | −0.01 | 0.01 | −0.01 | 0.01 |
| 14 bits (drop 2) | 0.00 | 0.05 | −0.01 | 0.07 | −0.64 |
| 12 bits (drop 4) | 0.04 | 0.35 | −3.58 | 0.93 | 1.71 |
| 10 bits (drop 6) | 0.01 | −0.31 | 2.52 | −4.57 | 11.85 |

The stopband of this section is at −45 dB; a 14-bit operand still resolves
it, 12 bits put the truncation noise at the stopband level, 10 bits bury it.
So "similar to the input and output data resolution" means 14 to 16 bits.
The default drops nothing.

**Range.** With DC off the operands are −Z2 and −Z1 and fit the delay
width. With DC on they are Xn − Z2 and Xn − Z1, differences of two
full-scale signals. At the cutoff a two-pole section lags 90°, so |Xn − Z|
reaches 1.22 × full scale there and 1.4 × above it. With a 16-bit LATCH 18 a
30000-amplitude sine at 1 kHz through the 1 kHz DC-on section clips the
operand 8673 times in 12288 samples and reads +0.60 dB instead of −3.14 dB:

| LATCH 18 | sine 30000 at 1 kHz | clips |
|---|---|---|
| 16 bits | +0.60 dB (ideal −3.14) | 8673 |
| 17 bits | −3.14 dB | 0 |
| 18 bits | −3.14 dB | 0 |
| 16 bits, sine 15000 | −3.14 dB | 0 |

The same clipping shows in two more places. The neutral word (B2' = 1,
B1' = 2, a transparent DC-on section) is not transparent at 16 bits for a
full-scale jump: −20000 followed by 32767 gives X − Z1 = 52767
(`neutral_word_clips_at_16_bits`). And a resonant DC-on section near its
peak has |X − Z| up to (1 + peak gain) times the input: the 2-pole lowpass
at cutoff 255 (18 kHz, Q 2.5) reads 7 dB low at 20 kHz with a 5900 input
at 16 bits and exactly right at 18.

So either the real multiplier's signed input is wider than the delay
memory (17 bits covers the neutral word, 18 the resonances), or signals
into a DC-on section are kept well below full scale, or the chip accepts
this clipping (it distorts only the correction term B(Xn − Z), not the
main path 2Z1 − Z2). The default keeps 16 bits with saturation and counts
the clips in `mult_in_sat`; the tests that need a clean DC-on measurement
use 18 bits. Saturating rather than wrapping at LATCH 18 is again the
choice that keeps a wrong product bounded.

## Coefficient: 16-bit mantissa, signed exponent (`mant_bits = 16`)

The code table is one function used with two scalings
(VOICE_INTERFACE.md): an X byte gives B2' (or A2') = table/2^18, an F byte
gives the difference f² = B1' − B2' (or A1' − A2') = table/2^16. One unit of
coefficient is therefore 2^-18 (`coef::FRAC_BITS`): B2' runs from 2^-18 to
exactly 1.0 (code 255) and f² from 2^-16 to 4.0, so B1' can reach 5.0. The
neutral word X = 0xFF, F = 0xCF is B2' = 1, B1' = 2.

The first version of the emulator gave the multiplier a 6-bit mantissa,
enough to hold any single decoded code exactly. That fails: the first
coefficient is the second plus a decoded difference, and at 100 Hz the
difference (44 units) against B2' (4864 units) is lost when the sum is
renormalised to 6 bits. Then B1' = B2', 1 + B1 + B2 = 0, a pole sits on
z = 1 and the DC-off section has infinite DC gain
(`six_bit_mantissa_loses_the_difference`). The difference B1' − B2' ≈ 2θ²
is what sets the pole frequency, which is why the patent encodes it
separately (5:48-54) and why the abstract says 16-bit coefficients are
adequate.

A second thing the mantissa must carry is the fraction of an interpolated
code. The voice interpolates codes in 1/256 steps; truncating the decoded
value to whole table units would quantise a small f² in steps of 0.8 %
(124.88 units at a 62.22 code became 124). The decoder therefore keeps the
code's eight fraction bits in the mantissa and lets the exponent go
negative (`BlockFloat.exp: i32`); the barrel shifter of FIG. 7 then shifts
further right. With a 16-bit mantissa nothing is lost for a single code
(13 to 15 bits), and the aligned addition of FIG. 8 only drops bits when
the two exponents differ by more than the spare mantissa bits.

So FIG. 7's "block floating point ... to achieve a greater range" reads as
a 16-bit multiplier whose exponent runs from about −8 to +2, not as a short
mantissa with a wide exponent.

## Product, LATCH 38: floor, saturating into the accumulator width

The barrel shifter is an arithmetic right shift, so the product is rounded
toward minus infinity (`product_round = Floor`). The product of a 16-bit
operand and a coefficient of at most 4.0 is below 2^18 and never saturates
in 24 bits; `product_sat` counts if it does.

Floor and round-to-nearest leave different tails after an impulse:

| Section | Floor | Nearest |
|---|---|---|
| 1 kHz Q = 10, DC on | tail exactly 0 | limit cycle, ±37 LSB |
| 1 kHz Q = 0.707, DC on | constant 1 LSB | |
| 1 kHz Q = 0.707, DC off | constant 12 LSB | |

Floor produces a dead band where the two products cancel after rounding:
with DC off a residual r survives while ceil(B2' r) = ceil(B1' r), i.e. up to
about 1/(B1' − B2') LSB, here 64. Round-to-nearest sustains a limit cycle at
the resonance instead. Floor is the simpler hardware and the better
behaviour, so it is the default.

## Low cutoffs: sub-LSB products

With a 16-bit signal path and products truncated to the signal LSB, the
DC-on correction terms B(X − Z) at a low cutoff are smaller than one LSB:
at the 2-pole lowpass's cutoff 0 (84 Hz, Q 3.35) a 50 Hz sine of 9000 LSB
has |X − Z1| of about 60 LSB and B2'(X − Z) of 0.2 LSB. Floor truncation
biases the recursion and the measured response is 0.25 dB low at 50 Hz;
round-to-nearest halves that, and the operand or accumulator width changes
nothing. A product with fraction bits below the signal LSB, or a wider
signal path, would be needed to do better; whether the chip has either is
unknown. The document's 0.02 dB agreement was measured on the stock types,
presumably at moderate cutoffs.

## Dynamic range of the DC-off form

With DC off the section is 1/(1 + B1 z⁻¹ + B2 z⁻²) with no scaling, so its
DC gain is 1/(B1' − B2'):

| Cutoff (Q = 0.707) | B1' − B2' | DC gain |
|---|---|---|
| 1 kHz | 1/64 | 64 (36 dB) |
| 100 Hz | 11/65536 | 5958 (75.5 dB) |

A 16-bit delay memory therefore saturates unless the input is scaled down by
the DC gain: 468 LSB at 1 kHz, 5 LSB at 100 Hz for a clean measurement. This
is the "extreme dynamic range problems ... when filters with very low
frequency passbands are used" of the background (1:24) and the reason for
the FIG. 5 form, whose DC gain is 1 at any cutoff. The emulator supports
both; the DC bit selects.

## Defaults

| Parameter | Default | Range worth exploring |
|---|---|---|
| `acc_bits` / `acc_saturate` | 24 / wrap | 20 is enough for one section |
| `input_bits` | 24 | 16 to 24 |
| `delay_bits` | 16, saturating | 14 to 18 |
| `mult_in_bits` / `mult_in_saturate` | 16 / saturate | 17 or 18 for clean DC-on at full scale |
| `mult_lsb_drop` | 0 | 2 is harmless, 4 degrades the stopband |
| `mant_bits` | 16 | 6 fails at low cutoffs; the mantissa also carries 8 fraction bits of an interpolated code |
| `exact_coef_sum` | false (aligned add) | |
| `product_round` / `product_saturate` | Floor / saturate | Nearest produces limit cycles |
