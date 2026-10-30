# Problems met while building the emulator

In the order they came up. Each entry says what was observed, what caused
it, and what was decided. Where a decision is a guess about the real chip it
is marked as such.

## 1. The "Prod" column of the reservation table is off by one row

The column shows the b2' and b1' products in rows 4 and 5, the same rows
whose "Adder Input 14" consumes them, but shows a2'Z2 in row 6 and a1'Z1 in
row 7, one row before rows 7 and 8 consume them. A single product latch
(LATCH 38) cannot hold a2'Z2 through row 7 while presenting a1'Z1 in row 7.

Decision: a constant pipeline latency. The operand is latched at the end of
row k and the product is on the bus in row k+3; that is forced by rows 1→4
and 2→5 and then requires the raw delays Z2 and Z1 to be latched at the ends
of rows 4 and 5. The A products in the Prod column are treated as a
misprint. Side effect: LATCH 18 is loaded in rows 1, 2, 4, 5 and free in
rows 3, 6, 7, 8, the four slots the patent reserves for interpolation.

## 2. Loading the input collides with storing the output

ACC 10 holds Xn from row 1 and receives MAout at the end of row 8. For one
section run on its own there is no cycle in which the next input can be
loaded. The text says "at the beginning of the cycle, accumulator 10 is
loaded with the input signal" and "MAout = Xn".

Decision: a row 0 load state (ACC 10 ← input, LATCH 13/20 ← delay memory)
precedes the eight rows. In a chip that multiplexes many sections this is
the previous section's row 8, so it costs nothing there; for a lone section
it is a ninth cycle. Guess about the chip: the chain output is the adder
output of the last row, so the next input can be loaded in the same cycle.

## 3. A 6-bit coefficient mantissa loses the pole frequency

The hypothesised code table has a five-bit mantissa and a four-bit shift, so
the first version gave the multiplier a six-bit mantissa, enough to hold any
one decoded code exactly. At 100 Hz / 48 kHz this produced B1' == B2',
1 + B1 + B2 = 0, a pole exactly on z = 1 and infinite DC gain in the DC-off
form. Cause: the first coefficient is the second plus a decoded difference
(FIG. 8, ADDER 51), and renormalising the sum 1216 + 11 to six bits drops
the 11. The difference B1' − B2' ≈ 2θ² is what sets the pole frequency,
which is why the patent encodes it separately.

Decision: 16-bit mantissa. Then the exponent is 0 below 1.0 and 1 or 2 up to
4.0, which reads the abstract's "16 bit coefficients are adequate" literally
and makes FIG. 7's block floating point a two-bit range extension. At 20 Hz
the difference is 0.9 units of 2^-16, the resolution limit the abstract
alludes to. The six-bit failure is kept as a test.

## 4. The DC-on form clips its multiplier operand at full scale

With DC on, a 30000-amplitude sine at the 1 kHz cutoff measured +0.60 dB
where −3.14 dB was expected, and the impulse response (small values) was
fine. Cause: the operands are Xn − Z2 and Xn − Z1. A two-pole lags 90° at
the cutoff, so |Xn − Z| reaches 1.22 × full scale (1.4 × above the cutoff)
and a 16-bit LATCH 18 clips it; 8673 clips in 12288 samples. With a 17-bit
LATCH 18, or the input 6 dB down, the error is 0.00 dB.

Decision: keep 16 bits saturating as the default, count the clips, and note
that the real chip either has an operand latch one bit wider than the delay
memory or accepts this distortion of the correction term. The DC-on
response tests measure at 18 bits and a separate test checks the clipping.

## 5. Sine measurements need the amplitude scaled by the filter's gain

The first sine measurements of the DC-off section were 10 dB low at every
low frequency and the Q = 10 resonance read 0.7 dB instead of 19.5 dB. Cause:
the section's own gain (64 at DC with DC off, 9.7 at the resonance) drove
the 16-bit delay memory into saturation; this is a property of the filter,
not an error. The harness now scales the sine so that the loudest point of
the ideal response stays below 30000 in the delay (468 LSB for the 1 kHz
DC-off section, 5 LSB at 100 Hz).

## 6. Floor truncation leaves a residual; rounding leaves a limit cycle

Impulse-response tails did not go to zero: a constant 12 LSB with DC off, 1
LSB with DC on at Q = 0.707. Cause: the products are floor-truncated and, of
negated operands, so AR = ceil(B2' r) − ceil(B1' r) is zero for a residual r
up to about 1/(B1' − B2') LSB. Trying round-to-nearest instead gave a
sustained ±37 LSB oscillation at the Q = 10 resonance.

Decision: floor stays the default (simpler hardware, bounded residual). The
residual makes the impulse-response DFT unreliable near DC and in deep
stopbands, so the sine measurement is the primary check and the DFT a
secondary one away from DC.

## 7. The DC-off form has no usable dynamic range at low cutoffs

1/(B1' − B2') is 64 (36 dB) at 1 kHz and 5958 (75.5 dB) at 100 Hz. With a
16-bit delay memory a full-scale input saturates immediately; the input has
to be scaled down by the DC gain, which the structure cannot do by itself
("no scaling of either input or output"). This is the background's
"extreme dynamic range problems ... with very low frequency passbands" and
the reason FIG. 5 exists. Not a bug; the emulator models both forms and the
tests show the DC-off saturation stays bounded.

## 8. A wrong test expectation

A test assumed h[0] = 0 for the DC-on form because the direct input path is
removed. It is not zero: the DC-on transfer function is
(B1' − B2')/D(z), so h[0] = (B1' − B2') x[0]. The emulator was right; the
test was fixed. Worth keeping in mind when reading the form: the "removed"
direct path reappears as the numerator constant.

## 9. Input wider than the accumulator

`input_bits` defaulted to 24 with a 24-bit accumulator; setting
`acc_bits = 20` alone would have loaded a 24-bit value into a 20-bit ACC 10.
The load now clamps to `min(input_bits, acc_bits)`. At 20 bits the measured
responses are identical to 24 bits and the overflow counter stays at zero,
as the 13 × 2^15 worst-case bound predicts.

## Smaller notes

- MUX 16 needs a zero for the "& DC" and "& IDC" terms; nothing in the
  drawing provides it. Modelled as a gate on adder input 17.
- The mux in front of LATCH 18 is never mentioned in the text; it is the
  only path for the raw delays to reach the multiplier for the A products.
- The delay memory write could be in row 7 or 8 (ACC 11 holds Yn from the
  end of row 6); row 7 was chosen. Address generation for a multi-section
  chip is not modelled.
- The direct MUX 16 → ACC 11 line is in the netlist and the control word
  but no row of the reservation table uses it.
- FIG. 8 does not draw the feedback of the previous exponent (LATCH 50)
  into LOGIC 49 that the aligned addition needs; it is assumed.
