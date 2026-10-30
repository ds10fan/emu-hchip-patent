# Verification: does the microcode make a lowpass?

Sample rate 48 kHz throughout. Everything below is reproducible with the
`hfilter` binary (`cargo run --release -- ...`) and the tests
(`cargo test --release`).

## Method

1. **Wiring.** The cycle-level emulator (muxes, gate, adder, latches,
   three-stage multiplier pipeline, delay memory) is compared sample by
   sample with a straight-line integer statement of the reservation table
   (`src/reference.rs`) over random inputs and random coefficient codes, in
   both DC modes and with several width settings. Any mis-wiring of a mux
   select, carry-in, latch load or pipeline stage shows up as a mismatch
   (`tests/datapath.rs`).
2. **Pure two-pole.** A2' = 1.0 (code 207) and A1' = 2.0 (code 207 as the
   difference) make the FIR half transparent: the section output MAout equals
   the Yn written to the delay memory, bit for bit, as long as nothing
   saturates (`pure_two_pole_output_equals_delay_value`).
3. **Response.** A lowpass is designed from (fc, Q): the denominator of the
   bilinear two-pole, converted to B1' = B1 + 2 and B2' = 1 − B2, encoded to
   the nearest codes. The expected response is computed from the coefficients
   the decoder actually delivers, so coefficient quantisation is excluded and
   only the arithmetic is judged. Two measurements:
   - a sine at each frequency, amplitude scaled so the loudest point of the
     ideal response stays inside the 16-bit delay, correlated with the input
     over 8192 samples after 4096 of settling;
   - the DFT of a 16384-sample impulse response.
4. **Lowpass properties.** Monotonic fall above the cutoff, −12 dB/octave
   asymptote, DC gain of 1 (DC on) or 1/(B1' − B2') (DC off), a resonance
   peak of the designed height at Q = 10.

## Cycle trace

`hfilter trace --x 10000 --samples 3 --fc 5000 --q 1` (DC off, decoded
B2' = 0.46875, B1' = 0.78125). Register columns show the content during the
cycle; `sum` is the adder output stored at its end.

```
--- sample 1: x = 0                                         (z1=10000 z2=0)
cycle        in14      in17 cin       sum |     acc10     acc11      prod   latch18
0 load         -1         0   1         0 |     10000     10000         0         0
1              -1         0   1         0 |         0     10000         0         0
2          -10001         0   1    -10000 |         0     10000         0         0
3              -1     20000   1     20000 |         0     10000         0    -10000
4               0        -1   1         0 |         0     20000         0    -10000
5           -7813        -1   1     -7813 |         0     20000     -7813         0
6           20000     -7813   0     12187 |     -7813     20000     -7813     10000
7               0      7812   1      7813 |     -7813     12187         0     10000
8          20000     -7814   1     12187 |      7813     12187     20000     10000
output = 12187, delay memory = [12187, 10000]
```

Row 2 forms −Z1 = −10000 for the B1' multiply; row 3 stores 2Z1 − Z2 = 20000
in ACC 11; row 5 adds the product floor(−10000 × 0.78125) = −7813 to AR2 = 0;
row 6 gives Yn = 20000 − 7813 = 12187; rows 7 and 8 subtract A2'Z2 = 0 and
add A1'Z1 = 20000 back and return 12187: the pure-pole output equals Yn. The
products appear on `prod` three cycles after their operand was latched.

## 1 kHz, Q = 0.707, DC off (`hfilter response --fc 1000`)

Decoded B2' = 0.171875 (5632·2^3 units of 2^-18), B1' = 0.1875; poles
R = 0.910, 695 Hz; DC gain 64 (36.1 dB). Impulse 2048, sine 468 (scaled by
the DC gain).

| f [Hz] | ideal dB | DFT dB | err | sine dB | err |
|---|---|---|---|---|---|
| 20 | 36.12 | 35.79 | −0.33 | 36.04 | −0.09 |
| 50 | 36.12 | 36.11 | −0.01 | 36.15 | 0.03 |
| 100 | 36.12 | 36.11 | −0.01 | 36.09 | −0.03 |
| 200 | 36.10 | 36.11 | 0.00 | 36.09 | −0.01 |
| 500 | 35.79 | 35.80 | 0.00 | 35.80 | 0.01 |
| 1000 | 32.98 | 33.00 | 0.02 | 32.98 | 0.00 |
| 1500 | 28.23 | 28.20 | −0.03 | 28.22 | 0.00 |
| 2000 | 23.81 | 23.83 | 0.01 | 23.81 | −0.01 |
| 3000 | 17.08 | 17.08 | 0.00 | 17.08 | 0.00 |
| 5000 | 8.47 | 8.52 | 0.05 | 8.47 | 0.00 |
| 7000 | 2.94 | 2.94 | 0.00 | 2.94 | 0.00 |
| 10000 | −2.60 | −2.63 | −0.03 | −2.61 | 0.00 |
| 15000 | −8.02 | −8.08 | −0.06 | −8.02 | 0.00 |
| 20000 | −10.62 | −10.55 | 0.07 | −10.63 | −0.01 |
| 23000 | −11.19 | −11.12 | 0.07 | −11.19 | −0.01 |

The response is the all-pole 1/D(z): 36 dB at DC, −3 dB at the designed
cutoff relative to DC, 12 dB per octave above it, flattening to −11 dB at
Nyquist because the section has no zeros. The 20 Hz DFT error is the
floor-truncation residual in the impulse tail (12 LSB constant), not a
response error; the sine measurement does not see it.

## 1 kHz, Q = 0.707, DC on (`hfilter response --dc --fc 1000`)

Same poles, DC gain exactly 1. Impulse 32767, sine 30000.

| f [Hz] | ideal dB | DFT dB | err | sine dB | err |
|---|---|---|---|---|---|
| 20 | 0.00 | −0.05 | −0.05 | −0.09 | −0.09 |
| 50 | 0.00 | 0.06 | 0.06 | 0.03 | 0.03 |
| 100 | 0.00 | 0.05 | 0.06 | −0.03 | −0.03 |
| 200 | −0.02 | 0.03 | 0.05 | −0.03 | −0.01 |
| 500 | −0.33 | −0.34 | −0.01 | −0.32 | 0.01 |
| 1000 | −3.14 | −3.07 | 0.07 | 0.60 | 3.74 |
| 1500 | −7.90 | −7.96 | −0.07 | −5.36 | 2.53 |
| 2000 | −12.31 | −12.28 | 0.04 | −10.98 | 1.33 |
| 3000 | −19.04 | −19.08 | −0.03 | −18.78 | 0.27 |
| 5000 | −27.65 | −27.68 | −0.03 | −27.65 | 0.00 |
| 7000 | −33.18 | −33.22 | −0.04 | −33.18 | 0.00 |
| 10000 | −38.73 | −38.72 | 0.01 | −38.73 | 0.00 |
| 15000 | −44.14 | −44.17 | −0.03 | −44.14 | 0.00 |
| 20000 | −46.74 | −46.73 | 0.01 | −46.75 | 0.00 |
| 23000 | −47.31 | −47.32 | −0.01 | −47.31 | 0.00 |

The impulse DFT matches within 0.07 dB everywhere. The full-scale sine is
off by up to 3.7 dB between 1 and 3 kHz: the DC form multiplies Xn − Z, which
exceeds 16 bits near the cutoff where the output is in quadrature with the
input, and LATCH 18 clips it (8673 clips at 1 kHz). With a 17-bit LATCH 18,
or the sine at 15000, the error is 0.00 dB (`hfilter sine --dc --f 1000
--sine-amp 30000 --mult-bits 17`). The tests measure this case at 18 bits
and check the clipping separately. See WIDTHS_AND_SATURATION.md.

## 1 kHz, Q = 10, DC on (`hfilter response --dc --fc 1000 --q 10`)

Decoded B2' = 0.013184, B1' = 0.029785; R = 0.9934, 987 Hz; peak gain 9.74
(19.8 dB). Impulse 32767, sine 3080.

| f [Hz] | ideal dB | DFT dB | err | sine dB | err |
|---|---|---|---|---|---|
| 20 | 0.00 | −0.36 | −0.37 | −0.08 | −0.08 |
| 50 | 0.02 | 0.29 | 0.27 | 0.06 | 0.04 |
| 100 | 0.09 | 0.31 | 0.23 | 0.07 | −0.02 |
| 200 | 0.36 | 0.18 | −0.18 | 0.36 | 0.00 |
| 500 | 2.55 | 2.68 | 0.13 | 2.57 | 0.02 |
| 1000 | 19.46 | 19.56 | 0.10 | 19.47 | 0.01 |
| 1500 | −2.34 | −2.53 | −0.19 | −2.33 | 0.00 |
| 2000 | −9.78 | −9.82 | −0.03 | −9.78 | 0.01 |
| 3000 | −18.19 | −18.00 | 0.19 | −18.21 | −0.02 |
| 5000 | −27.51 | −27.71 | −0.20 | −27.49 | 0.02 |
| 7000 | −33.22 | −33.16 | 0.07 | −33.21 | 0.01 |
| 10000 | −38.86 | −39.03 | −0.17 | −38.87 | −0.01 |
| 15000 | −44.32 | −44.54 | −0.22 | −44.32 | 0.01 |
| 20000 | −46.94 | −46.99 | −0.05 | −46.95 | −0.01 |
| 23000 | −47.51 | −47.42 | 0.09 | −47.50 | 0.01 |

The resonance is where and as high as the decoded coefficients say. The sine
measurement is within 0.08 dB at every frequency; the DFT scatter of ±0.4 dB
comes from the finite 16384-point window against a slowly decaying response.

## 100 Hz, Q = 0.707, both modes

DC on (`--dc --fc 100`): decoded B2' = 1216, B1' = 1227 units; the sine
measurement is within 0.15 dB except at 100 and 200 Hz, where the full-scale
operand clips as above (+3.4 dB, +2.1 dB); with an 18-bit LATCH 18 the test
`lowpass_100hz_dc_on_matches_ideal` passes within 0.2 dB down to the −60 dB
level.

DC off (`--fc 100 --amp 64`): DC gain 5958 (75.5 dB); the sine must be 5 LSB
to stay inside the delay memory, and the measurement still follows the ideal
within 0.5 dB over most of the band. The impulse DFT is unusable above
2 kHz because the 12-LSB residual tail dominates the tiny high-frequency
content. With the 6-bit coefficient mantissa the same section had B1' = B2'
and a pole at z = 1 (`six_bit_mantissa_loses_the_difference`).

## The voice: zeros, gain, stock filter types

`tests/voice.rs` and VOICE.md: the decoded words reproduce the tables of
VOICE_INTERFACE.md (coefficients to 1 part in 4000, the 1/256 code
quantisation; f0 and Q to 0.3 % and 0.02), the neutral voice is
transparent (at 18 operand bits), the gain word scales the output by
table/2^18, and the 2-pole, 6-pole and 4th-order-highpass voices match
their ideal responses within 0.25 dB wherever the output exceeds 100 LSB,
0.5 dB at the 2-pole's cutoff 0.

## Lowpass properties (`it_is_a_lowpass`)

For both DC modes at 1 kHz: the gain falls by more than 6 dB per octave at
2, 4, 8 and 16 kHz; the slope between 4 and 8 kHz is between −10 and
−13 dB/octave (ideal −11.6 for a section without zeros this close to
Nyquist); the 20 Hz gain equals the DC gain of the form (0 dB or
1/(B1' − B2')) within 0.15 dB.

## Test list

```
bits::tests::wrap_and_sat
coef::tests::{decode_code_matches_table, table_is_monotonic_and_ends_at_2_pow_18,
              fraction_interpolates_linearly_between_codes, encode_round_trips,
              roles_scale_as_documented, fraction_bits_survive_in_the_mantissa,
              sixteen_bit_mantissa_is_exact_for_single_codes}
mult::tests::{unity_and_two_are_exact, latency_is_three_cycles}
datapath: matches_reference_default_widths
          matches_reference_with_gain_rows
          matches_reference_with_saturating_accumulator_and_rounding
          matches_reference_with_narrow_words
          matches_reference_with_wrapping_operand_and_exact_coef_sum
          pure_two_pole_output_equals_delay_value
          first_sample_of_impulse
          single_section_worst_case_fits_in_20_bits
lowpass:  lowpass_1khz_dc_off_matches_ideal
          lowpass_1khz_dc_on_matches_ideal
          resonant_1khz_q10_dc_on_matches_ideal
          lowpass_100hz_dc_on_matches_ideal
          it_is_a_lowpass
          impulse_dft_matches_away_from_dc
          quantised_poles_are_close
          dc_off_full_scale_saturates_but_stays_bounded
          dc_on_operand_clips_at_16_bits
          six_bit_mantissa_loses_the_difference
voice:    word_table_matches_document_lp2_q64
          word_table_matches_document_hp4_q64
          neutral_voice_is_transparent
          neutral_word_clips_at_16_bits
          lp2_q64_response_matches_ideal_at_three_cutoffs
          lp2_is_a_lowpass_that_moves_with_the_cutoff
          lp6_q64_response_matches_ideal
          hp4_q64_response_matches_ideal_and_is_a_highpass
          gain_word_scales_the_output
```

All 37 pass (`cargo test --release`).
