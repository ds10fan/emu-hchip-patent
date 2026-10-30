# H-chip: what one voice computes, as seen from its control words

What follows is measured on the E4 (`MEASURED_CLASSIC.md`, `PROBE2.md`): with flag 0x40 and at levels where
nothing saturates, the stock filter types are predicted from their words by this description to about 0.02 dB.
It describes the voice's transfer function, not the arithmetic that produces it.

## Control words

Per voice: words 0..9 (32 bits each), a cutoff (0..255) and a flags byte.

- Each word holds two half-words: the upper one is the frame for cutoff 0, the lower one the frame for
  cutoff 255.
- A half-word is two 8-bit codes, X (high byte) and F (low byte). Word 9 under flag 0x40 is one 8-bit gain code
  per half-word (low byte).
- The code used at a cutoff is interpolated linearly in code space:
  `code = code_0 + (code_255 - code_0) * cutoff / 255`.
- Codes are decoded by the table in `COEF_CODES.txt`: F gives f^2 (int / 2^16), X gives B2' (int / 2^18).
  Measured for codes up to 0xFE. The table is probably produced by this decode (it reproduces every measured
  value; a hidden-bit mantissa shifted by a 4-bit exponent, every 16 codes doubling the value, except that from
code 0xCF up the step stays 0x1000 and the values run linearly from 0x10000 to 0x40000):

  ```c
  unsigned dec(unsigned code) { unsigned c = code + 1, e = c >> 4, m = c & 15; return c >= 224 ? (c - 192) << 12 : e ? (16 + m) << (e - 1) : m; }
  ```

## Building blocks

For a word with coefficients f^2 and B2':

```
D(z) = 1 - (2 - f^2 - B2') z^-1 + (1 - B2') z^-2  =  1 - (2 - B1') z^-1 + (1 - B2') z^-2,   B1' = f^2 + B2'
```

B2' is decoded from the X code; f^2 from the F code; B1' is not stored, it is their sum. These are the patent's
coefficients (US 5,170,369, `PATENT_US5170369.md`): against `D(z) = 1 + B1 z^-1 + B2 z^-2` it stores the offsets
`B1' = B1 + 2` and `B2' = 1 - B2` [3:36-62], and log-encodes B2' and the difference `B1' - B2'` [5:48; claims 4,
5], which is f^2 here. Its coefficient decoder forms B1' by adding the decoded difference to the decoded B2'
[6:29]. Example, word 50405040: B2' = 0.001038, f^2 = 0.002075, B1' = 0.003113,
`D(z) = 1 - 1.996887 z^-1 + 0.998962 z^-2`.

- **pole section**, normalised: `f^2 / D(z)` (unity gain at DC)

  ```
  y[n] = f^2 x[n] + (2 - f^2 - B2') y[n-1] - (1 - B2') y[n-2]
  ```

- **pole section**, unnormalised: `1 / D(z)`

  ```
  y[n] = x[n] + (2 - f^2 - B2') y[n-1] - (1 - B2') y[n-2]
  ```

- **zero section**: `N(z) = D(z)`, the same polynomial used as a numerator, not normalised (gain f^2 at DC)

  ```
  y[n] = x[n] - (2 - f^2 - B2') x[n-1] + (1 - B2') x[n-2]
  ```

- **gain section**: `y[n] = g9 x[n]`

The difference equations give the measured transfer functions; they are not a statement about the chip's
internal arithmetic (its order of operations, word widths or rounding).

The neutral word `0xFFCFFFCF` (X = 0xFF, F = 0xCF: B2' = 1, f^2 = 1) gives 1 in every role.

## The voice

With flag 0x40 set:

```
H(z) = f0^2/D0(z) * N1(z) * N5(z) * P4(z) * P8(z) * g9
```

| word | role |
|---|---|
| 0 | pole word, always normalised |
| 1, 5 | zero words |
| 4, 8 | pole words: normalised; unnormalised (1/D) when flag 0x80 is set |
| 9 | gain g9: amplitude relative to code 0xFE = int(code) / int(0xFE) from `COEF_CODES.txt` |
| 2, 3, 6, 7 | no effect |

Without flag 0x40, word 9 is a third zero word `N9(z)` and the output gain is fixed.

## Examples (stock filter types)

The words below are what EOS writes for three stock types (dumped from the firmware's own coefficient code on the
68020 harness, `crates/eos320/examples/dump_words.rs`). In the tables, the codes are interpolated at the cutoff as
above, and f0 and Q are the pole (or zero) pair's frequency and Q computed from the roots of D(z)
(`s = fs ln z`, `f0 = |s| / 2 pi`, `Q = |s| / (-2 Re s)`) at fs = 48 kHz. Unused words are the neutral word
`FFCFFFCF`.

### 2 Pole Lowpass (12 dB/oct, one pole section)

Flags 0x40. Only word 0 and word 9 change with Q:

| Q | word 0 | word 9 |
|---|---|---|
| 0 | `8408FFCF` | `00FE00FE` |
| 64 | `6A07E6E3` | `00F600FE` |
| 127 | `5205D4F3` | `00EF00FE` |

At Q 0 the cutoff-255 frame is the neutral word (the filter is open). Word 0 at Q 64:

| cutoff | X code | F code | B2' | f^2 | f0 | Q |
|---|---|---|---|---|---|---|
| 0 | 106.00 | 7.00 | 0.003296 | 0.000122 | 84 Hz | 3.35 |
| 64 | 137.12 | 62.22 | 0.012755 | 0.001905 | 335 Hz | 3.41 |
| 128 | 168.24 | 117.43 | 0.049303 | 0.021906 | 1146 Hz | 2.97 |
| 192 | 199.36 | 172.65 | 0.190349 | 0.231618 | 3915 Hz | 2.43 |
| 255 | 230.00 | 227.00 | 0.609375 | 2.250000 | 18150 Hz | 2.53 |

Moving the cutoff moves both codes together (frequency); Q lowers X, which lowers B2' (less damping), with
F nearly the same at cutoff 0.

### 6 Pole Lowpass (36 dB/oct, three identical sections)

Flags 0x40. Words 0, 4 and 8 hold the same word (normalised pole sections in series). Q 64: words 0/4/8 =
`7008ECDE`, word 9 = `00EC00FE`. Each section:

| cutoff | X code | F code | B2' | f^2 | f0 | Q |
|---|---|---|---|---|---|---|
| 0 | 112.00 | 8.00 | 0.004150 | 0.000137 | 90 Hz | 2.82 |
| 64 | 143.12 | 61.71 | 0.015744 | 0.001874 | 332 Hz | 2.74 |
| 128 | 174.24 | 115.42 | 0.061022 | 0.019941 | 1097 Hz | 2.28 |
| 192 | 205.36 | 169.13 | 0.237224 | 0.204136 | 3724 Hz | 1.80 |
| 255 | 236.00 | 222.00 | 0.703125 | 1.937500 | 17430 Hz | 1.88 |

### 4th Order Highpass (24 dB/oct, two zero/pole pairs)

Flags 0xC0: words 4 and 8 are unnormalised poles 1/D, words 1 and 5 zeros N. Q 64: word 0 = `FFCFFFCF`,
words 1/5 = `80048004`, words 4/8 = `7004EFCF`, word 9 = `00FE00FE`. So `H = (N1/D4) (N5/D8) g9` with N1 = N5 and
D4 = D8.

Zero words (both frames equal, so independent of the cutoff):

| cutoff | X code | F code | B2' | f^2 | f0 | Q |
|---|---|---|---|---|---|---|
| 0 | 128.00 | 4.00 | 0.008301 | 0.000076 | 67 Hz | 1.05 |
| 255 | 128.00 | 4.00 | 0.008301 | 0.000076 | 67 Hz | 1.05 |

Pole words:

| cutoff | X code | F code | B2' | f^2 | f0 | Q |
|---|---|---|---|---|---|---|
| 0 | 112.00 | 4.00 | 0.004150 | 0.000076 | 67 Hz | 2.10 |
| 64 | 143.87 | 54.95 | 0.016479 | 0.001462 | 293 Hz | 2.31 |
| 128 | 175.75 | 105.90 | 0.065426 | 0.013134 | 891 Hz | 1.72 |
| 192 | 207.62 | 156.85 | 0.259743 | 0.116590 | 2823 Hz | 1.23 |
| 255 | 239.00 | 207.00 | 0.750000 | 1.000000 | 11377 Hz | 1.07 |

The zeros stay at 67 Hz while the poles move with the cutoff. Each pair N/D is flat at f_zero^2 / f_pole^2
(N(1) / D(1)) below the zeros, rises at 12 dB/oct between the zeros and the poles, and approaches 0 dB above the
poles (there N and D are nearly equal). Two pairs give the 24 dB/oct highpass below the cutoff.
