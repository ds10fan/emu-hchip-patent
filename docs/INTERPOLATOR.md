# The interpolator: FIGs. 9 to 12, Tables I and II

`src/interp.rs` implements the time behaviour of the interpolating variable
x. This part of the patent is specified to the bit: Table I gives the
increment code, FIG. 10's text gives the per-sample update, and Table II
lists the encoder's output for every difference range and approach time.
The emulator reproduces all of Table II (`tests/interp.rs`).

## x, destination, clamp

x is 16 bits. The destination (and clamp) is an 8-bit byte "translated into
a 16-bit linear value by simply duplicating the bits" (7:25): byte c stands
for c·0x0101, so cutoff 255 is x = 0xFFFF and cutoff 128 is 0x8080. That is
why the measured relationship reads as "cutoff/255" without any division:
c·0x0101/0xFFFF = c/255 exactly.

The voice interpolates every code as C(x) = Ca + x(Cb − Ca) with this x,
rounded to 8 fraction bits (`voice::interp_x`). One x per voice is assumed;
the patent does not say whether coefficients share x and the E4 has one
cutoff per voice.

## The increment code (Table I, FIG. 9)

8 bits: sign S, 3-bit shift F, 4-bit data d. Decoded to 16-bit signed:

| | F = 0 | F ≥ 1 |
|---|---|---|
| S = 0 | d | (16 + d) << (F − 1) |
| S = 1 | d − 16 | (d − 32) << (F − 1) |

Range −2048 (0xf0) to 1984 (0x7f). The lattice is a hidden-bit code like
the coefficient table; negative codes are not in code order (0x8f is −1,
0x90 is −32). The adder carry is set for positive increments (Table I
note), so the step is value + 1 for S = 0 and a zero step cannot be
reached: aiming at a destination always gets there.

## The update (FIG. 10)

Each sample, two phases on one adder:

1. sum = x + step; overflow if the 16-bit result wrapped (carry 81 against
   sign 82).
2. compare sum with the duplicated clamp in the direction of the
   increment: exceeded if the increment is positive and sum > clamp, or
   negative and sum < clamp.

If overflow or exceeded, x becomes the clamp; otherwise x becomes sum. At
rest on the clamp the forced carry pushes past it every sample and is
clamped back, so x stays put.

## The encoder (FIGs. 11, 12, Table II)

Given a destination, the current x and an approach time index k (eight
values, 32 << k samples: 32 to 4096), the hardware computes the 17-bit
difference destination − x and derives the code in one sample. Checked
against every row of Table II at all eight times (ends and middle of each
range, 1872 points): the code is the Table I lattice floor of the
difference shifted right arithmetically by 5 + k, saturating at 0x7f and
0xf0. That is: divide by the approach time rounding toward minus infinity
(an arithmetic shift of the sign-extended difference, which is what the
first-set-bit logic of FIG. 12 does), then keep the top five significant
bits.

Two consequences of the floor plus forced carry:

| sweep | nominal | actual |
|---|---|---|
| 0 → 255 at k = 0 | 32 | 34 samples (2047 floors to 1984, +1 → 1985 per sample) |
| 0 → 255 at k = 5 | 1024 | 1041 samples (63 floors to 62, +1 → 63 per sample) |
| 0x10 → 0x11 at k = 7 | 4096 | 257 samples (difference 257 >> 12 = 0, step is the carry alone) |

Approach times are nominal: small differences arrive early, large ones a
few percent late.

## Retargeting

Because the increment is computed from the current x, a new destination
given mid-sweep starts a correct new sweep immediately, which is the
patent's stated purpose (6:43-68). `Voice::sweep_to(cutoff, k)` does it;
`tests/interp.rs::retarget_mid_sweep` checks it.

## A sweep seen from the filter

`hfilter voice --type lp2 --vq 64 --cutoff 0 --sweep-to 255 --time 5`:

| sample | x | X0 code | F0 code | B2' | f0 [Hz] | Q |
|---|---|---|---|---|---|---|
| 0 | 0000 | 106.00 | 7.00 | 0.003296 | 84.5 | 3.35 |
| 128 | 1f80 | 121.26 | 34.07 | 0.006411 | 184.6 | 3.76 |
| 256 | 3f00 | 136.52 | 61.14 | 0.012459 | 328.6 | 3.43 |
| 384 | 5e80 | 151.77 | 88.21 | 0.024193 | 603.1 | 3.22 |
| 512 | 7e00 | 167.03 | 115.28 | 0.046936 | 1089.0 | 2.97 |
| 640 | 9d80 | 182.29 | 142.35 | 0.090973 | 1940.9 | 2.66 |
| 768 | bd00 | 197.55 | 169.42 | 0.176147 | 3675.6 | 2.48 |
| 896 | dc80 | 212.80 | 196.49 | 0.340698 | 7183.9 | 2.26 |
| 1024 | fc00 | 228.06 | 223.56 | 0.579102 | 16110.1 | 2.44 |

Linear motion of x gives a near-exponential motion of the pole frequency
(a doubling every 130 samples or so) with Q roughly held: the point of
interpolating in the encoded space (5:34-39; the ARMAdillo paper).

## What is modelled where

- Table I decode, FIG. 10 update, FIG. 11/12 encoder: `interp.rs`, bit
  exact to the patent.
- The coefficient math unit C(x) = Ca + x(Cb − Ca): `voice::interp_x`,
  functional. In the chip it would use the four free LATCH 18 slots of the
  frame (rows 3, 6, 7, 8), one per coefficient of the section, with x on the
  unsigned input and the code difference on the signed input; its adder is
  not the signal adder, which is busy in every row.
- Not known from the patent or the document: how the E4 turns its filter
  envelope and modulation into destination bytes and approach-time indices,
  and whether anything else than the cutoff is swept.
