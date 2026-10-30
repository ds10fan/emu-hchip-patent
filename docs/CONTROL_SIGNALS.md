# Control signals

The patent draws no control lines at all. The reservation table (columns 5
and 6) and the cycle-by-cycle text (4:9 to 4:65) fix what every mux must
select and what the carry-in must be in each of the eight states; the latch
loads and the multiplier pipeline follow from which product the adder
consumes in which row. This file lists the resulting control word and how
each field was derived. The source of truth is `src/microcode.rs`; the table
below is `hfilter microcode` output.

## Control word fields

| Field | Drives | Values |
|---|---|---|
| `mux15` | MUX 15 → adder input 14 | `Z2N` (~Z2), `Z1N` (~Z1), `Acc11`, `Prod` |
| `mux16` | MUX 16 → adder input 17 | `Z1x2` (line 21), `Acc10`, `Acc10N` (~ACC 10) |
| `gate` | zero gate on input 17 | `Always` (pass), `Dc` (pass only if DC = 1), `Idc` (pass only if DC = 0), `Never` (zero; gain rows) |
| `cin` | adder carry-in ("Adder CD") | `Zero`, `One`, `Idc` (= NOT DC) |
| `acc10` | MUX10 → ACC 10 | `Hold`, `Adder`, `Input` |
| `acc11` | MUX11 → ACC 11 | `Hold`, `Adder`, `Mux16` (the direct line; unused by this microcode) |
| `mux_m` | MUXM → LATCH 18 data | `Adder`, `Z2`, `Z1` |
| `load18` | LATCH 18 and LATCH 19 load, with the coefficient to fetch | `-`, `B2`, `B1`, `A2`, `A1`, `G` (gain) |
| `dmem_write` | DELAY MEMORY 22 write of ACC 11 | `-`, `yes` |
| `latch_z` | LATCH 13 / LATCH 20 load from DELAY MEMORY 22 | `-`, `yes` |

`DC` is a mode bit (the choice between the FIG. 4 and FIG. 5 forms, 3:66);
the patent does not say where it lives. In the emulator it is a per-section
register (`Section::dc`).

All register updates happen at the end of the cycle. A mux select is valid
during the cycle; `load18`, `acc10`, `acc11`, `dmem_write` and `latch_z`
name what is clocked in at the end of it.

## The microcode

```
cycle | mux15 | mux16  | gate   | cin  | acc10 | acc11 | mux_m | load18 | dmem_w | latch_z
------|-------|--------|--------|------|-------|-------|-------|--------|--------|--------
0 load | Z2N   | Acc10  | Dc     | One  | Input | Hold  | Adder | -      | -      | yes
1     | Z2N   | Acc10  | Dc     | One  | Hold  | Hold  | Adder | B2     | -      | -
2     | Z1N   | Acc10  | Dc     | One  | Hold  | Hold  | Adder | B1     | -      | -
3     | Z2N   | Z1x2   | Always | One  | Hold  | Adder | Adder | -      | -      | -
4     | Prod  | Acc10N | Idc    | Idc  | Adder | Hold  | Z2    | A2     | -      | -
5     | Prod  | Acc10N | Always | One  | Adder | Hold  | Z1    | A1     | -      | -
6     | Acc11 | Acc10  | Always | Zero | Hold  | Adder | Adder | -      | -      | -
7     | Prod  | Acc10N | Always | One  | Adder | Hold  | Adder | -      | yes    | -
8     | Prod  | Acc10N | Always | One  | Adder | Hold  | Adder | -      | -      | -
```

Rows 1 to 6 are the poles, rows 7 and 8 the zeros on the same delay
values; a section without zeros runs the same rows with A2' = 1, A1' = 2.

### With an output gain (`MICROCODE_GAIN`)

Rows 0 to 7 as above, then:

```
8g    | Prod  | Acc10N | Always | One  | Adder | Hold  | Adder | G      | -      | -
9     | Prod  | Acc10  | Never  | Zero | Hold  | Hold  | Adder | -      | -      | -
10    | Prod  | Acc10  | Never  | Zero | Hold  | Hold  | Adder | -      | -      | -
11    | Prod  | Acc10  | Never  | Zero | Adder | Hold  | Adder | -      | -      | -
```

Row 8g is row 8 plus a LATCH 18/19 load of the adder output (MAout) with
the gain coefficient; rows 9 and 10 wait for the multiplier; row 11 passes
g·MAout through the adder (input 17 gated to zero, carry 0) into ACC 10.
This is not in the patent; see VOICE.md.

## Derivation, row by row

Reservation table rows are quoted as printed; `'` is the one's complement
(bubble output), "SA" is ACC 10. Negation is always `~v + 1`, the complement
input plus carry-in 1, which is why the note under the table says two
complements cannot be added in one cycle.

| Row | Table: input 14 / input 17 / CD / result | Control |
|---|---|---|
| 1 | `Z2'` / `(SA = Xn) & DC` / 1 / `DC & Xn − Z2` | `mux15 = Z2N`, `mux16 = Acc10`, `gate = Dc`, `cin = One`; the result is the B2' operand: `mux_m = Adder`, `load18 = B2` |
| 2 | `Z1'` / `(SA = Xn) & DC` / 1 / `DC & Xn − Z1` | same with `Z1N`; `load18 = B1` |
| 3 | `Z2'` / `2Z1` / 1 / `2Z1 − Z2` → A 11 | `mux15 = Z2N`, `mux16 = Z1x2`, `cin = One`, `acc11 = Adder` |
| 4 | `b2'(…)` / `(SA = Xn') & IDC` / IDC / `−AR2` → A 10 | `mux15 = Prod`, `mux16 = Acc10N`, `gate = Idc`, `cin = Idc`, `acc10 = Adder`. Also `mux_m = Z2`, `load18 = A2` so that a2'Z2 is ready in row 7 |
| 5 | `b1'(…)` / `SA = −AR2'` / 1 / `AR` → A 10 | `mux15 = Prod`, `mux16 = Acc10N`, `cin = One`, `acc10 = Adder`. Also `mux_m = Z1`, `load18 = A1` for row 8 |
| 6 | `2Z1 − Z2` / `SA = AR` / 0 / `Yn` → A 11 | `mux15 = Acc11`, `mux16 = Acc10`, `cin = Zero`, `acc11 = Adder` |
| 7 | `a2'Z2` / `SA' = AR'` / 1 / `−MA1` → A 10 | `mux15 = Prod`, `mux16 = Acc10N`, `cin = One`, `acc10 = Adder`; `dmem_write` (ACC 11 holds Yn from the end of row 6) |
| 8 | `a1'Z1` / `SA' = −MA1'` / 1 / `MAout` → A 10 | `mux15 = Prod`, `mux16 = Acc10N`, `cin = One`, `acc10 = Adder` |

Row 0 is not in the table. It is "At the beginning of the cycle, accumulator
10 is loaded with the input signal" (4:13) plus the load of LATCH 13 and
LATCH 20 from the delay memory, which must precede row 1. In a chip that
runs many sections back to back this state is the previous section's row 8:
MAout lands in ACC 10 and is the next section's Xn ("MAout = Xn" under the
table), and the delay latches are loaded with the next section's Z1, Z2.
For a lone section row 0 costs one extra cycle per sample.

## Multiplier pipeline

LATCH 18 (operand) and LATCH 19 (coefficient mantissa, with the exponent
from LATCH 50) load together at the end of a `load18` cycle. The product
passes LATCH 35 at the end of the next cycle and LATCH 38 at the end of the
one after, and is on the `PROD` net in the third cycle:

| Operand latched at end of | Coefficient | Product used in |
|---|---|---|
| row 1 | B2' | row 4 |
| row 2 | B1' | row 5 |
| row 4 | A2' | row 7 |
| row 5 | A1' | row 8 |

This leaves rows 3, 6, 7, 8 free at LATCH 18: the four multiplier slots the
patent reserves for coefficient interpolation (4:1-8, 6:39-42).

The printed "Prod" column shows a2'Z2 in row 6 and a1'Z1 in row 7, one row
before the "Adder Input 14" column consumes them, while it shows the b2' and
b1' products in the same rows that consume them. One product latch cannot
hold a2'Z2 through row 7 while also presenting a1'Z1 in row 7, so the Prod
column cannot be read literally; the constant latency above is used.

## Coefficient fetch order

`load18` fetches B2', B1', A2', A1' (and G) in that order. The FIG. 8 decoder needs
it: a first coefficient (B1', A1') is a difference added to the second
coefficient held in LATCH 47, so the second must be decoded first (6:24-33).
The emulator's `Coef::kind()` maps B2/A2 to `Second` and B1/A1 to `First`.

## Delay memory

`dmem_write` in row 7 stores sat(ACC 11) as the new Y[n−1] and moves the old
Y[n−1] to Y[n−2]; the latches keep the old values until `latch_z` in row 0
of the next sample. Row 7 is a choice: ACC 11 holds Yn from the end of row 6
to the end of the frame, so the write could be in row 7 or 8. Address
generation for a multi-section chip is not modelled; the emulator keeps a
two-word memory per section.

## The direct MUX 16 → MUX11 line

`acc11 = Mux16` loads ACC 11 with the gated MUX 16 output without passing the
adder. No row of the reservation table uses it; it is in the netlist and the
control word because the drawing is missing that line, not because the
patent's microcode needs it.
