# Netlist of FIGs. 6, 7 and 8 of US 5,170,369

Read from a 300 dpi render of the drawing sheets (PDF pages 5, 6, 7). Every
connection is tagged:

- `[drawn]` visible in the figure,
- `[text]` not drawn but stated in the description or the reservation table,
- `[added]` not in the figure; the two lines missing from FIG. 6,
- `[assumed]` needed to make the reservation table executable, not in the patent.

Instance names use the patent numerals. Blocks the patent leaves unnumbered
get a name in capitals (MUX10, MUX11, MUXM, SHIFT21). Control inputs
(mux selects, latch loads, carry-in, memory control) are drawn nowhere in the
patent; they are listed per instance under `ctl:` with the names used by the
emulator's control word (see CONTROL_SIGNALS.md).

## FIG. 6: filter section datapath

### Nets

| Net | Meaning | Source |
|---|---|---|
| `IN` | INPUT SIGNAL, section input (previous section's output or chip input) | pin |
| `SUM` | ADDER 12 output | ADDER 12 |
| `A14` | adder input 14 | MUX 15 |
| `A17` | adder input 17 | MUX 16 |
| `CIN` | adder carry-in, the "Adder CD" column of the reservation table `[text]` | control |
| `A10`, `A10N` | ACC 10 true and complement (bubble) outputs | ACC 10 |
| `A11` | ACC 11 output | ACC 11 |
| `DOUT` | DELAY MEMORY 22 read data | DELAY MEMORY 22 |
| `Z2`, `Z2N` | LATCH 13 true and complement outputs: delay memory 2, Y[n-2] | LATCH 13 |
| `Z1`, `Z1N` | LATCH 20 true and complement outputs: delay memory 1, Y[n-1] | LATCH 20 |
| `Z1X2` | line 21: Z1 shifted left one bit (shift not drawn) `[text 4:31-34]` | SHIFT21 |
| `MM` | output of the unnumbered mux in front of LATCH 18 | MUXM |
| `L18` | LATCH 18 output = multiplier signed input (31 in FIG. 7) | LATCH 18 |
| `CM` | COEF. MEMORY 23 output (encoded coefficient; FIG. 8 sits between 23 and 19) | COEF MEMORY 23 |
| `L19` | LATCH 19 output = coefficient mantissa (33 in FIG. 7) | LATCH 19 |
| `PROD` | multiplier 24 output (LATCH 38 of FIG. 7) | MULT 24 |
| `OUT` | section output; the bus leaving ACC 10 to the right; "MAout = Xn" of the next section | = `A10` |

### Instances

```
ADDER 12          A = A14, B = A17, CIN = CIN                      -> SUM        [drawn; CIN: text]

MUX 15  (4:1)     in0 = Z2N  (LATCH 13 bubble)                     [drawn]
                  in1 = Z1N  (LATCH 20 bubble)                     [drawn]
                  in2 = A11  (ACC 11 output bus)                   [drawn]
                  in3 = PROD (multiplier 24 output)                [drawn]
                                                                   -> A14
                  ctl: mux15

MUX 16  (3:1)     in0 = Z1X2 (line 21)                             [drawn]
                  in1 = A10  (ACC 10 true)                         [drawn]
                  in2 = A10N (ACC 10 complement)                   [drawn]
                                                                   -> A17
                  ctl: mux16, gate
                  gate: A17 is forced to zero when the reservation table says
                  "& DC" (cycles 1, 2) or "& IDC" (cycle 4) and the condition
                  is false, and in the gain rows. No gate or zero input is
                  drawn.                                            [assumed]

MUX10   (3:1)     in0 = IN   (INPUT SIGNAL)                        [drawn]
                  in1 = SUM  (adder output)                        [drawn]
                  in2 = A10  (hold)                                [drawn]
                                                                   -> ACC 10 .D
                  the MUX10 -> ACC 10 line is missing in the figure [added]
                  ctl: acc10

ACC 10            D = MUX10.out                                    -> A10, A10N  [drawn]

MUX11   (3:1)     in0 = SUM  (adder output)                        [drawn]
                  in1 = A11  (hold)                                [drawn]
                  in2 = A17  (direct from MUX 16 output)           [added]
                                                                   -> ACC 11 .D
                  ctl: acc11

ACC 11            D = MUX11.out                                    -> A11        [drawn]

DELAY MEMORY 22   DIN = A11                                        [drawn]
                  DOUT                                             -> DOUT       [drawn]
                  ctl: dmem_write (address generation not shown)   [assumed]

LATCH 13          D = DOUT                                         -> Z2, Z2N    [drawn]
                  ctl: latch_z (load)
LATCH 20          D = DOUT                                         -> Z1, Z1N    [drawn]
                  ctl: latch_z (load)

SHIFT21           Z1X2 = Z1 << 1  (wiring, "shifted left one bit") [text]

MUXM    (3:1)     in0 = SUM  (long line along the top of the figure) [drawn]
                  in1 = Z2   (LATCH 13 true output)                [drawn]
                  in2 = Z1   (LATCH 20 true output)                [drawn]
                                                                   -> MM
                  ctl: mux_m
                  (The text never mentions this mux. in1/in2 are the only way
                  the raw delays reach the multiplier for a2'Z2 and a1'Z1.)

LATCH 18          D = MM                                           -> L18        [drawn]
                  ctl: load18
COEF MEMORY 23                                                     -> CM         [drawn]
                  ctl: coef (which of B2', B1', A2', A1'; plus the gain G
                  of the emulator's gain rows)
LATCH 19          D = CM                                           -> L19        [drawn]
                  ctl: load18 (loaded together with LATCH 18)      [text 4:21-23]

MULT 24           X = L18 (signed), Y = L19 (unsigned)             -> PROD       [drawn]
                  (detail in FIG. 7)
```

### Fan-out, net by net

```
IN    -> MUX10.in0
SUM   -> MUX10.in1, MUX11.in0, MUXM.in0
A10   -> MUX16.in1, MUX10.in2 (hold), OUT
A10N  -> MUX16.in2
A11   -> MUX15.in2, DELAY MEMORY 22.DIN, MUX11.in1 (hold)
A17   -> ADDER 12.B, MUX11.in2 [added]
DOUT  -> LATCH 13.D, LATCH 20.D
Z2    -> MUXM.in1
Z2N   -> MUX15.in0
Z1    -> SHIFT21, MUXM.in2
Z1N   -> MUX15.in1
Z1X2  -> MUX16.in0
PROD  -> MUX15.in3
```

### What the figure does not show

Mux selects, latch loads, the carry-in, the DC/IDC gate, memory addressing,
clocks, bit widths, and any saturation. The reservation table supplies the
selects and carry-in per cycle; widths and saturation are design decisions
(WIDTHS_AND_SATURATION.md).

## FIG. 7: multiplier 24 (block floating point)

### Nets

| Net | Meaning |
|---|---|
| `L18` | SIGNAL INPUT 31 (signed) = LATCH 18 of FIG. 6 |
| `MANT` | COEFFICIENT MANTISSA 33 (unsigned) = LATCH 19 of FIG. 6 |
| `EXP` | COEFFICIENT EXPONENT = LATCH 50 of FIG. 8 |
| `P32` | first stage output (carry-save partial products) |
| `L35` | pipeline latch 35 output |
| `P36` | second stage output, the full product |
| `L34` | exponent pipeline latch 34 output |
| `SH37` | barrel shifter 37 output |
| `PROD` | MULTIPLIER PRODUCT = LATCH 38 output = `PROD` of FIG. 6 |

### Instances

```
STAGE 32          A = L18 (SIGNED INPUT), B = MANT (UNSIGNED INPUT) -> P32   [drawn]
                  "Booth's recoders and Wallace trees"              [text]
LATCH 35          D = P32                                           -> L35   [drawn]
STAGE 36          in = L35                                          -> P36   [drawn]
                  "final lookahead carry adder"                     [text]
LATCH 34          D = EXP                                           -> L34   [drawn]
BARREL SHIFTER 37 data = P36, shift = L34                           -> SH37  [drawn]
LATCH 38          D = SH37                                          -> PROD  [drawn]
```

Pipeline timing implied by the reservation table: LATCH 18/19 load at the end
of cycle k, LATCH 35 at the end of k+1, LATCH 38 at the end of k+2, and
`PROD` is used by MUX 15 in cycle k+3. Operands formed in cycles 1 and 2 are
consumed as products in cycles 4 and 5; the raw delays loaded in cycles 4 and
5 are consumed in cycles 7 and 8. The table's "Prod" column lists a2'Z2 and
a1'Z1 one row earlier than the "Adder Input 14" column consumes them; the
emulator uses the constant latency, which is the only reading consistent with
rows 4 and 5.

## FIG. 8: coefficient decoder

### Nets

| Net | Meaning |
|---|---|
| `ENC` | ENCODED COEFFICIENT (from COEF. MEMORY 23, or the coefficient math unit in a dynamic filter) |
| `L41` | LATCH 41 output; `L41[hi]` is item 42, "the most significant bits (in this case 4 bits)"; `L41[lo]` the rest |
| `SH43` | shift amount from LOGIC 43 |
| `E43` | exponent from LOGIC 43 |
| `SH44` | BARREL SHIFTER 44 output (decoded value) |
| `L45` | LATCH 45 output |
| `L46` | LATCH 46 output (exponent of the decoded value) |
| `S52`, `S53` | BARREL SHIFTER 52, 53 outputs (aligned adder operands) |
| `SUM51` | ADDER 51 output |
| `M48` | MUX 48 output |
| `L47` | LATCH 47 output = COEFFICIENT OUTPUT (mantissa 33 of FIG. 7, via LATCH 19) |
| `L50` | LATCH 50 output = EXPONENT OUTPUT (to LATCH 34 of FIG. 7) |

### Instances

```
LATCH 41          D = ENC                                           -> L41         [drawn]
LOGIC 43          in = L41[hi] (42)                                 -> SH43, E43   [drawn]
BARREL SHIFTER 44 data = L41[lo], shift = SH43                      -> SH44        [drawn]
LATCH 45          D = SH44                                          -> L45         [drawn]
LATCH 46          D = E43                                           -> L46         [drawn]
BARREL SHIFTER 52 data = L45, control from LOGIC 49                 -> S52         [drawn]
BARREL SHIFTER 53 data = L47 (fed back from the output latch),      -> S53         [drawn]
                  control from LOGIC 49
ADDER 51          A = S52, B = S53                                  -> SUM51       [drawn]
MUX 48            in0 = L45   (second coefficient: pass through)    [drawn]
                  in1 = SUM51 (first coefficient: second + decoded difference)
                                                                    -> M48
                  ctl: first/second
LATCH 47          D = M48                                           -> L47         [drawn]
LOGIC 49          in = L46                                          -> LATCH 50 .D [drawn]
                  also controls BARREL SHIFTER 52 and 53            [drawn]
                  the previous exponent (L50) must also enter LOGIC 49 to
                  align the addition; that feedback is not drawn    [assumed]
LATCH 50          D = LOGIC 49 out                                  -> L50         [drawn]
```

Sequencing `[text 6:24-35]`: the second coefficient (B2' or A2') is decoded
first and passes through MUX 48 to LATCH 47 with its exponent passed
unchanged to LATCH 50. The first coefficient (B1' or A1') is decoded next as
a difference; ADDER 51 adds it to the value held in LATCH 47 after shifters
52 and 53 align the two exponents, and LOGIC 49 produces the sum's exponent.
This matches the reservation table's order B2', B1', A2', A1' and the
ARMAdillo paper's b1' = b2' + 4e^(-k1).
