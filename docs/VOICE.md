# The voice: zeros, gain, and three sections in series

`src/voice.rs` maps the E4 voice of `VOICE_INTERFACE.md` (ten 32-bit words,
a cutoff, a flags byte) onto the patent's datapath. This file says how, what
the microcode for zeros and gain is, and what the model measures for the
stock filter types. `hfilter voice --type lp2|lp6|hp4 --vq 64 --cutoff N`
prints everything below; `tests/voice.rs` checks it.

## Zeros are rows 7 and 8, not a second microcode

The reservation table computes a full biquad in one frame. Rows 1 to 6 are
the poles: Yn goes into the delay memory. Rows 7 and 8 are the zeros: the
same delay values Z2, Z1 are multiplied by A2', A1' (latched in rows 4 and
5) and combined with AR into MAout. That is direct form II, the FIR taps
sharing the recursion's delay line (FIGs. 2 and 4).

What a "zero word" shares with a "pole word" is the decoding, not a
program: X gives A2' = table/2^18 exactly as it gives B2', F gives the
difference A1' − A2' = table/2^16 exactly as it gives f², and the numerator
MAout/Yn = 1 + (A1' − 2) z⁻¹ + (1 − A2') z⁻² is the document's
N(z) = 1 − (2 − f² − B2') z⁻¹ + (1 − B2') z⁻² with the zero word's values.
A section without zeros runs the same rows with the neutral codes
(X = 0xFF, F = 0xCF: A2' = 1, A1' = 2), and MAout equals Yn.

Because the zeros act on the pole section's state, the measured
"zero section acting on its input" of the document and this model are the
same transfer function; only where clipping happens differs.

## Gain: three extra rows on multiplier 24

The patent has no gain. With flag 0x40, word 9 is not a zero word but a
gain code, and no choice of the row 7 and 8 operands yields g·Yn (the AR
term cannot be scaled from Z1 and Z2). The emulator therefore runs
`MICROCODE_GAIN` for the third section:

```
8g    | Prod  | Acc10N | Always | One  | Adder | Hold  | Adder | G      |   row 8 as before, and MAout -> LATCH 18 with the gain
9     | Prod  | Acc10  | Never  | Zero | Hold  | Hold  | Adder | -      |   multiplier pipeline
10    | Prod  | Acc10  | Never  | Zero | Hold  | Hold  | Adder | -      |   multiplier pipeline
11    | Prod  | Acc10  | Never  | Zero | Adder | Hold  | Adder | -      |   g * MAout + 0 -> ACC 10
```

Only drawn paths are used: MUXM's adder input carries MAout (the row 8
sum) into LATCH 18, the product returns through MUX 15, input 17 is gated
to zero (the same gate the DC terms need), and ACC 10 takes the result. The
section's zeros are set neutral. Rows 9 and 10 are idle for a lone voice;
a pipelined chip would overlap them with the next section's rows 1 and 2,
which do not use the adder's product input.

The gain value is table(code)/2^18, so 0xFF is 1.0 and 0xFE is 0.984. The
document only states amplitudes relative to code 0xFE; the absolute level
is a guess. The gain could equally live outside the filter (an output
multiplier or the mixer); using the voice's own multiplier is the natural
reading of "word 9 replaces the third zero word".

The operand of the gain product passes LATCH 18, so with a 16-bit latch the
voice output is clipped to 16 bits before the gain. Without the flag the
output gain is 1 and word 9 is the third zero word.

## Word to section mapping

| Section | Pole word | Zero word | DC mode |
|---|---|---|---|
| 0 | 0 | 1 | always on (f²/D) |
| 1 | 4 | 5 | on, off with flag 0x80 (1/D) |
| 2 | 8 | 9, or gain with flag 0x40 | on, off with flag 0x80 |

Words 2, 3, 6, 7 are not used. A word's upper half-word is the frame for
cutoff 0 and the lower one for cutoff 255; each half-word is X (high byte)
and F (low byte). The neutral word is 0xFFCFFFCF.

## Interpolation, and where the expansion sits

The code at a cutoff is C(x) = c0 + x(c255 − c0) with x the voice's 16-bit
interpolating variable, a cutoff byte duplicated (c·0x0101, so 255 is
0xFFFF): no division, and c·0x0101/0xFFFF is exactly c/255. The result is
kept with 8 fraction bits (nearest 1/256; at x = 0xFFFF it can be 1/256
below c255 because 0xFFFF is 1 − 1/65536). The decoder expands it with the
fraction in the mantissa: LOGIC 43 takes the exponent from the integer part and BARREL
SHIFTER 44 shifts the hidden bit, the four mantissa bits and the eight
fraction bits together. The value is piecewise linear between adjacent
integer codes, including across exponent boundaries, which is what the
document's fractional-code tables show (its tables use exact real codes; the
1/256 quantisation changes the coefficients by at most 1 part in 4000).

This is the patent's order: the coefficient math unit forms
C(x) = Ca + x(Cb − Ca) and "the coefficient to be decoded ... would be the
output of the coefficient math unit" (5:58-64). Expansion follows
interpolation. The interpolation multiplies are the four free LATCH 18
slots of the frame (rows 3, 6, 7, 8); here they are done in software. The
FIG. 10 to 12 hardware moves x over time (destination and approach time):
see INTERPOLATOR.md and `Voice::sweep_to`.

## Cost

A lone voice takes 9 + 9 + 12 = 30 datapath cycles per sample. At 48 kHz
that is 1.44 M cycles per second per voice.

## Measured stock types

Input sine scaled so that no delay memory saturates; 18-bit LATCH 18 for
the tests (see below for 16); "ideal" is computed from the coefficients the
decoder delivers. Only rows where the ideal output exceeds 100 LSB are
meaningful; the others are the quantisation floor.

### 2 Pole Lowpass, Q 64 (word 0 = 6A07E6E3, word 9 = 00F600FE)

Decoded words at the three cutoffs, as the document tabulates them:

| cutoff | X | F | B2' | f² | f0 | Q | gain |
|---|---|---|---|---|---|---|---|
| 0 | 106.00 | 7.00 | 0.003296 | 0.000122 | 84.5 Hz | 3.35 | 0.859 |
| 128 | 168.24 | 117.43 | 0.049301 | 0.021904 | 1146 Hz | 2.97 | 0.922 |
| 255 | 230.00 | 227.00 | 0.609375 | 2.250000 | 18150 Hz | 2.53 | 0.984 |

Response at cutoff 128 (input 9954):

| f [Hz] | ideal dB | measured dB | err |
|---|---|---|---|
| 20 | −0.70 | −0.80 | −0.09 |
| 100 | −0.64 | −0.66 | −0.02 |
| 500 | 0.99 | 1.00 | 0.01 |
| 1000 | 7.74 | 7.75 | 0.00 |
| 2000 | −7.22 | −7.22 | 0.00 |
| 5000 | −25.54 | −25.55 | 0.00 |
| 10000 | −36.97 | −36.97 | 0.00 |
| 20000 | −45.06 | −45.07 | −0.01 |

At cutoff 0 (84 Hz, Q 3.35) the error reaches −0.25 dB at 50 Hz: the
products of the DC-on form are below one LSB there and floor truncation
biases them (round-to-nearest halves the error; operand width makes no
difference). At cutoff 255 the 20 kHz point is 7 dB low with a 16-bit
LATCH 18 and exact with 18 bits: near the 18 kHz resonance the operand
X − Z reaches (1 + peak gain) times the input amplitude.

### 6 Pole Lowpass, Q 64 (words 0, 4, 8 = 7008ECDE, word 9 = 00EC00FE), cutoff 128

Each section 1097 Hz, Q 2.28; gain 0.844. The internal peak is 12.8
(22 dB) so the input is 2340.

| f [Hz] | ideal dB | measured dB | err |
|---|---|---|---|
| 20 | −1.46 | −1.55 | −0.09 |
| 200 | −0.68 | −0.70 | −0.02 |
| 500 | 3.81 | 3.82 | 0.01 |
| 1000 | 20.32 | 20.31 | −0.01 |
| 2000 | −24.78 | −24.74 | 0.03 |
| 5000 | −78.45 | (floor) | |

### 4th Order Highpass, Q 64 (words 1, 5 = 80048004; 4, 8 = 7004EFCF; flags 0xC0), cutoff 128

Zeros at 67 Hz (Q 1.05) in sections 0 and 1, unnormalised poles at 891 Hz
(Q 1.72) in sections 1 and 2.

| f [Hz] | ideal dB | measured dB | err |
|---|---|---|---|
| 200 | −51.76 | −52.43 | −0.67 |
| 500 | −15.02 | −15.05 | −0.03 |
| 1000 | 10.52 | 10.52 | 0.00 |
| 2000 | 3.36 | 3.37 | 0.00 |
| 5000 | 0.85 | 0.86 | 0.00 |
| 20000 | 0.43 | 0.43 | 0.00 |

Below 200 Hz the signal between the zero section and the unnormalised pole
section is a few LSB and the measurement is noise; the structure puts the
attenuating zeros before the amplifying poles.

## What the voice does not model

- How the E4 derives destination bytes and approach times from its
  envelopes and modulation; the voice only offers `set_cutoff` (jump) and
  `sweep_to` (destination and approach-time index).
- How the chip multiplexes voices and sections; each section here has its
  own two-word delay memory and runs its full program.
- The absolute output level, and any output or mixer gain outside the
  filter.
