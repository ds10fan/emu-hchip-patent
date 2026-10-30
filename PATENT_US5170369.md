# US 5,170,369 (Rossum): transcription, figures, and what it says about the H-chip

A fresh reading of E-mu's patent from the scanned PDF (`~/p/emu/emupatent.pdf`, 18 image-only pages),
done without relying on the earlier notes. Pages were rendered at 300 dpi, each text column was OCRed
separately with tesseract, and every equation, table, claim and doubtful line was checked against the page
image. Figures were read from the images directly. Crops of the key figures and tables are in
`docs/patent_figs/`.

Conventions in this file:

- `[c:l]` is column:line of the printed patent (columns 1 to 12, line numbers from the gutter; the line given
  is where the paragraph starts and may be off by one).
- `[sic]` marks a misprint kept as printed. Text in *[square brackets in italics]* is mine, not the patent's.
- Anything not legible is marked *unreadable*. Nothing in the transcription is filled in by guessing.

Contents: 1. bibliographic data and abstract; 2. full text; 3. claims; 4. figures; 5. answers to the
questions (a) to (f); 6. where the earlier notes are wrong or overstated.

---

## 1. Front page

| | |
|---|---|
| Patent | US 5,170,369, granted Dec. 8, 1992 |
| Title | Dynamic digital IIR audio filter and method which provides dynamic digital filtering for audio signals |
| Inventor | David P. Rossum, Aptos, Calif. |
| Assignee | E-mu Systems, Inc., Scotts Valley, Calif. |
| Appl. No. / filed | 576,203 / Aug. 29, 1990 |
| Related | Continuation-in-part of Ser. No. 411,450, Sep. 25, 1989, abandoned |
| Int. Cl.5 / U.S. Cl. | G06F 15/31 / 364/724.17 |
| Field of search | 364/724.01, 724.16, 724.17, 723, 748.5 |
| References cited | 4,305,133 (Amada et al.), 4,563,681 (Godard), 4,569,030 (Butterweck et al.), 4,573,135 (Dieterich), 4,736,333 (Mead et al.); J. Dattorro, "The Implementation of Recursive Digital Filters for High-Fidelity Audio", JAES vol. 36 no. 11, Nov. 1988, pp. 851-878 |
| Examiner / attorney | Long T. Nguyen / Heller, Ehrman, White & McAuliffe |
| Size | 7 claims, 11 drawing sheets |

The front-page drawing is FIG. 3.

**[57] Abstract.** An improved dynamic digital IIR (Infinite Impulse Response) audio filter and method
provides a remedy for the major problems in state of the art digital IIR filtering and dynamic digital
filtering for audio. In particular, the required multiplier coefficient size is reduced, such that 16 bit
coefficients are adequate for good frequency resolution. In addition, the data size of the multiplier is also
reduced, to be similar to the input and output data resolution without inferior noise performance.
Furthermore, the structure is easily modified to allow the fixing of the DC (Direct Current) gain of the IIR
filter to unity regardless of coefficient choice, eliminating any problems with filter dynamic range at low
frequency passbands. Additionally, a dynamic filter structure allows logarithmic filter sweeps and compresses
the coefficients further down to as few as 8 bits with adequate resolution for audio throughout the band.
The structure allows the alteration of sweep of the filter without excessive computation or the requirement
to read the current sweep value.

---

## 2. Full text (columns 1 to 10)

**DYNAMIC DIGITAL IIR AUDIO FILTER AND METHOD WHICH PROVIDES DYNAMIC DIGITAL FILTERING FOR AUDIO SIGNALS**

### Cross-reference to related applications

[1:9] The present invention is a continuation-in-part of copending patent application entitled DYNAMIC
DIGITAL IIR AUDIO FILTER, Ser. No. 411,450, filed Sep. 25, 1989, now abandoned, and which is assigned to the
same assignee as the present invention.

### Background of the invention

[1:16] The present invention relates to a dynamic digital IIR audio filter and corresponding method.

[1:18] The current state of the art for digital infinite impulse response (IIR) filters for audio
applications utilizes long wordlengths to obtain adequate noise performance for adequate sound quality.
Typically 24 bit or longer coefficients are used to obtain adequate resolution at low frequencies, and 24 bit
multiplier data paths or multiple precision techniques are used to improve noise performance. Finally,
extreme dynamic range problems exist when filters with very low frequency passbands are used. The article in
the Jan. 1989 Journal of the Audio Engineering Society provides a summary of the current state of the art.

[1:30] The state of the art in dynamic audio filtering, in which the coefficients are to be changing in time,
is even more primitive. Coefficients are typically linearly interpolated, producing filters whose
characteristics typically vary dramatically in audio [printed "andio"] quality over the sweep range from the
desired swept response. The coefficients are linearly encoded in the classical IIR filter forms, preventing
any logarithmic or audibly meaningful sweep, and requiring long words of typically 24 bits for adequate low
frequency resolution. Furthermore, any dynamic sweeping requires computation of the increment required to be
added each sample time. This complicates control of the filter during complicated sweeps, and makes it
impossible to change the sweep trajectory of the filter mid-sweep without reading back the current sweep
location.

### Summary of the invention

[1:49] It is an object of the present invention to provide an improved dynamic digital IIR audio filter and
method.

[1:51] It is another object of the present invention to remedy the major problems in state of the art
digital IIR filtering and dynamic digital filtering for audio. In particular, the required multiplier
coefficient size is reduced, such that 16-bit coefficients are adequate for good frequency resolution. In
addition, the data size of the multiplier is also reduced to be similar to the input and output data
resolution without inferior noise performance. Furthermore, the structure is easily modified to allow the
fixing of the DC gain of the IIR filter to unity regardless of coefficient choice, eliminating any problems
with filter dynamic range at low frequency passbands.

[1:63] Additionally, a dynamic filter structure is disclosed which allows logarithmic filter sweeps, and
compresses the coefficients further down to as few as 8 bits with adequate resolution for audio throughout
the band.

[1:67] Finally, a structure is disclosed which allows the alteration of sweep of the filter without excessive
computation or the requirement to read the current sweep value.

[2:3] All of these objectives can be accomplished by the present invention while allowing implementation of
up to 128th order filtering on a single silicon integrated circuit.

[2:7] Additional objects, advantages and novel features of the present invention will be set forth in part
in the description which follows and in part become apparent to those skilled in the art upon examination of
the following or may be learned by practice of the invention. The objects and advantages of the invention
may be realized and attained by means of the instrumentalities and combinations which are pointed out in the
appended claims.

### Brief description of the drawings

[2:18] The accompanying drawings which are incorporated in and form a part of this specification illustrate
an embodiment of the invention and, together with the description, serve to explain the principles of the
invention.

- FIG. 1 depicts a diagram of a prior art digital filter structure.
- FIG. 2 depicts another diagram of a prior art digital filter structure.
- FIG. 3 depicts a diagram of an improved digital filter structure according to the present invention.
- FIG. 4 depicts a diagram of another embodiment of a digital filter structure according to the present
  invention.
- FIG. 5 depicts a further feature of the digital filter according to the present invention.
- FIG. 6 depicts an implementation of the improved dynamic digital audio filter according to the present
  invention.
- FIG. 7 depicts a diagram of a multiplier block which forms a portion of FIG. 6.
- FIG. 8 depicts a diagram of a decoding technique which forms a part of the present invention.
- FIG. 9 depicts a preferred embodiment of the present invention.
- FIG. 10 depicts logic used to update an interpolating variable.
- FIG. 11 illustrates circuitry used to implement automatic computation of an encoded increment according to
  the present invention.
- FIGS. 12A and 12B show the logic diagram of a portion of FIG. 11.

*[Note: from the multiplier paragraph onward the detailed description cites figure numbers one higher than
the drawings: text "FIG. 8" = drawn FIG. 7, "FIG. 9" = drawn FIG. 8, "FIG. 10" = drawn FIG. 9,
"FIG. 11" = drawn FIG. 10, "FIG. 12" = drawn FIG. 11, "FIG. 13" = drawn FIGS. 12A/12B. There is no FIG. 13
sheet. Section 4 below maps each paragraph to its drawing.]*

### Detailed description of the preferred embodiment

[2:53] Reference will now be made in detail to the preferred embodiment of the invention, an example of
which is illustrated in the accompanying drawings. While the invention will be described in conjunction with
the preferred embodiment, it will be understood that it is not intended to limit the invention to that
embodiment. On the contrary, it is intended to cover alternatives, modifications and equivalents as may be
included within the spirit and scope of the invention as defined by the appended claims.

[2:63] The classical filter equation for a second order IIR digital filter is

```
Yn = Xn - B1*Yn-1 - B2*Yn-2
```

B1 and B2 form complex conjugate poles at locations:

```
R = sqrt(B2)    theta = arccos(-B1/2R)
```

[3:4] where R is the radius of the pole location and theta is the angle, with zero corresponding
[printed "coresponding", twice] to zero frequency and pi corresponding to half the sampling rate or
approximately 20 kHz. A classical filter structure to directly implement this equation is shown in FIG. 1.
Additionally, a second order FIR stage can be added to produce zeros to complete the classical structure as
shown in FIG. 2.

[3:12] The audio spectrum spans frequencies from 20 Hz to 20 kHz, and is typically logarithmically divided
into ten octaves by the ear. The ear's sensitivity to loudness is similarly logarithmic and is approximately
a range of 100 decibels (dB), where a decibel is defined as

```
20*log10(E/Eref)
```

where E is the signal level and Eref is a reference level.

[3:22] It is easily seen that the magnitude response of a resonance in a filter produced by a pole is
proportional to the difference between the pole's radius and unity. Hence the ear's sensitivity to the
position of the pole is tremendously amplified as the pole's radius approaches unity. Hence it would be
desirable to develop a filter whose coefficients were proportional to the distance between the pole's radius
and unity.

[3:30] Through a similar argument, emphasizing instead the ear's logarithmic frequency sensitivity, it can be
shown that the desirable coefficient relationship for the angle of poles should be such that small numbers
represent very small angles, whose cosines approach unity.

[3:36] If one examines the flow graph of an IIR filter from a noise point of view, one will observe that the
quantization of the signal at the input to the multiplier is the source of quantization noise. Reduction in
the required truncation at the multiplier input will reduce the noise produced by the filter. It is easily
observed that IIR filters become noisier as their poles tend towards unit radius and towards very low or
very high frequencies. This can be explained by realizing that in these cases, the recursion within the
filter is more severe, hence increasing the adverse effect of the multiplier input truncation noise. If the
filter structure can be altered such that in these cases there is less effective quantization noise,
improved performance in these interesting cases will result. A filter structure which accomplishes the above
structural requirements according to the present invention is shown in FIG. 3, and the extension to add FIR
terms in FIG. 4. Note that the coefficients have been modified as per the requirements:

```
B1' = B1 + 2
B2' = 1 - B2
```

[3:61] and hence all coefficients are always positive, and the multiplies unsigned.

[3:63] Another desirable feature for audio filters, in keeping with the objects of the invention, is to
force the gain of the filter at zero frequency to be unity. This feature can be accomplished by the
structure in FIG. 5. Note that the structures in FIGS. 4 and 5 are easily switched between.

[4:1] One implementation of the filter according to the present invention is shown in FIG. 6. In one
preferred embodiment, a total of four multiplies and eight additions are used. As will later be discussed,
this allows four multiplies to be used for interpolation of coefficients in dynamic filtering if the adder
and multiplier have the same speed, which is typically the case using modern VLSI components.

[4:9] In FIG. 6, accumulator 10 is used to accumulate the output signal. Auxilliary [sic] accumulator 11
accumulates the value to be saved in the delay memory 22. The filter operates in eight states. At the
beginning of the cycle, accumulator 10 is loaded with the input signal.

[4:15] During the first cycle, the adder 12 is supplied with negative delay memory 2's data from latch 13 at
input 14 via multiplexer 15, and at input 17 the input signal from accumulator 10, or zero, depending on the
DC gain stabilization desired, through multiplexer 16. The sum is latched into the multiplier input latch 18
to be multiplied by the associated unsigned coefficient B2' from coefficient memory 23, the output of which
will simultaneously be loaded into multiplier input latch 19.

[4:24] During the next cycle, negative delay memory 1's data from latch 20 is similarly routed to adder
input 14, while input 17 stays the same. The sum is latched again in multiplier input latch 18 for
multiplication by unsigned coefficient B1'.

[4:30] During the third cycle, adder input 14 is switched back to negative delay memory 2's data, while
adder input 17 is changed by multiplexer 16 to signal 21, which is positive delay memory 1's value shifted
left one bit to form twice delay memory 1's value. The resulting sum is stored at the end of the cycle in
auxilliary [sic] accumulator 11.

[4:37] During the fourth cycle, the output of the multiplier 24 is routed to added [sic] input 14. This now
is the product resulting from coefficient B2'. Adder input 17 will be selected as either negative the input
signal as routed from accumulator 10, or zero, depending on the DC gain stabilitzation [sic] requirement.
The resulting sum is latched into accumulator 10.

[4:45] During the fifth cycle, the output of the multiplier 24, containing the product resulting from
coefficient B1', is routed to adder input 10 [sic; input 14 is meant]. The negative of the contents of
accumulator 10 is routed to adder input 17. The resulting sum is stored in accumulator 10.

[4:50] During the sixth cycle, the contents of accumulator 11 is routed to adder input 14, and of
accumulator 10 to input 17. The resulting sum is saved in accumulator 11 to be stored in delay memory 22.

[4:54] During the seventh cycle, the multiplier output, which is the product resulting from coefficient A2',
is routed to adder input 14, and the negative of accumulator 10 routed to input 17. The resulting sum is
stored in accumulator 10.

[4:60] During the eighth and final cycle, the multiplier output, which is the product resulting from
coefficient A1', is routed to the adder input 14, and the negative of the signal accumulator is routed to
input 17. The resulting sum, which is the filter output and hence the input to the next stage, is stored in
accumulator 10.

[4:66] The reservation table for the above pipeline described in conjunction with FIG. 6 is presented below.

#### Reservation table (spans columns 5 and 6; crop: `patent_figs/reservation_table.png`)

Transcribed exactly, misprints included. A prime after a data symbol (`Z2'`, `Xn'`, `AR'`, `MA1'`) is the
printed prime; from the text it denotes the negated (complemented) value, e.g. `Z2'` is "negative delay
memory 2's data". `SA` is not defined anywhere; from [4:63] ("the negative of the signal accumulator") it is
accumulator 10. `CD` is the column headed "Adder CD"; from the notes below it is the adder carry input.
`DC` and `IDC` are not defined anywhere in the text.

| cycle *[my numbering]* | Adder Input 14 | Adder Input 17 | Adder CD | Result | A 10 contents | A 11 contents | Prod |
|---|---|---|---|---|---|---|---|
| 1 | `Z2'` | `(SA = Xn) & DC` | 1 | `DC & Xn − Z2` | `Xn` | xx | xx |
| 2 | `Z1'` | `(SA = Xn) & DC` | 1 | `DC & Xn − Z1` | `Xn` | xx | xx |
| 3 | `Z2'` | `2Z1` | 1 | `2Z1 − Z2` | `Xn` | xx | xx |
| 4 | `b2'(DC & Xn − Z2)` | `(SA = Xn') & IDC` | IDC | `−AR2` | `Xn` | `2Z1 − Z2` | `b2(DC & Xn − X2)` [sic] |
| 5 | `b1'(DC & Xn − Z1)` | `SA = −AR2'` | 1 | `b1'(DC & Xn − Z1) + AR2` | `−AR2` | `2Z1 − Z2` | `b1(DC & Xn − Z1)` |
| 6 | `2Z1 − ZX2` [sic] | `SA = AR` | 0 | `Yn` | `AR` | `2Z1 − Z2` | `a2'Z2` |
| 7 | `a2'Z2` | `SA' = AR'` | 1 | `a2'Z2 − AR` | `AR` | `Yn` | `a1'Z1` |
| 8 | `a1'Z1` | `SA' = −MA1'` | 1 | `MA1 + a1'Z1` | `−MA1` | `Yn` | xx |
| | | | | | `MAout = Xn` | | |

The "contents" columns are the register contents during the cycle (each row shows the previous row's
result), and "Prod" is the multiplier output available in that cycle. The last row, `MAout = Xn`, says the
section output becomes the next section's input.

Equations under the table, as printed:

```
AR2   = IDC & Xn − b2'(DC & Xn − X2)
AR    = IDC'Xn + b1'(DC & Xn − Z1) + b2'(Z2 − DC & Xn =AR2 + b1'(DC & Xn − Z1)
Yn    = IDC'Xn − b1'(Z1 − DC & Xn) + b2'(Z2 − DC & Xn) + 2Z1 − 2Z2 = AR + 2Z1 − 2Z2
MA1   = IDC'Xn − b1'(Z2 − DC & Xn) + b2'(Z2 − DC & Xn) − a2'Z2 = AR − a2'Z2
MAout = IDC'Xn − b1'(Z1 − DC & Xn) + b2'(Z2 − DC & Xn) + a1'Z1 − a2'Z2 = MA1 + a1'Z1
```

(The `2` before the first `Z1` in the `Yn` line is printed as a subscript, `₂Z1`. `X2` in the first line,
`Z2` in the `b1'` term of the `MA1` line, `2Z2` in the `Yn` line and the missing parenthesis in the `AR`
line are misprints; cycle 3 and the A 11 column give `2Z1 − Z2`.)

Notes, as printed:

> Yn must be positive. hence XL must be positive 2Z1 − Z2 or dodi must be complemented, dodi cannot always
> be complemented as both senses of product area required for MA portion and we cannot add two complements
> as a carry in of 2 is required. Hence, Z2 must be complemented, therefore Z1 should be complemented. We
> conclude that the B multiplies will be negative Z data.

("XL" and "dodi" are not defined anywhere in the patent.)

*[What the table computes, read with DC as a 1-bit signal and IDC as its inverse; this is my reading, the
patent states only the table: with IDC = 1 (DC = 0) the recursion is `Yn = Xn + 2Z1 − Z2 − b1'Z1 + b2'Z2`
(FIGS. 3/4); with DC = 1 it is `Yn = 2Z1 − Z2 − b1'(Z1 − Xn) + b2'(Z2 − Xn)` (FIG. 5), and `Xn` has no direct
path. `Yn` here is the value written to the delay memory (A 11). The section output is
`MAout = AR + a1'Z1 − a2'Z2`, where `AR = Yn − (2Z1 − Z2)`, i.e. the numerator taps the raw delays Z1, Z2,
never the differences with Xn.]*

[5:19] It should be noted that the mulitiplier [sic] 24 of FIG. 6 uses block floating point techniques on
the coefficients to achieve a greater range. The details of the multiplier block are illustrated in FIG. 8
*[drawn as FIG. 7]*. The signal is applied to the signed input 31 of the multiplier input stage 32 (which
consists of the Booth's recoders and wallace trees), and the coefficient mantissa is applied to the unsigned
input 33. The coefficient exponent is applied to the exponent pipeline latch 34, which is required due to
the multiplier pipeline latch 35 which feeds the second stage 36 of the multiplier consisting of the final
lookahead carry adder. This in turn feeds barrel shifter 37, which adjusts for the previous coefficient's
exponent. The final product, correctly scaled, is stored in product latch 38.

[5:34] In accordance with the objects of the current invention, logarithmic interpolation of the
coefficients is required to produce audibly meaningful sweeps. This is accomplished not through true
logarithmic interpolation, but rather through linear interpolation of approximately logarithmically encoded
coefficients.

[5:40] The poles of the filter are assumed to be, for important cases, near zero frequency and near unity
radius. Hence,

```
B2' = 1 − B2 = 1 − R2 = 1 − (1 − 2d + d2)    if R = 1 − d.
```

Since R is close to 1, d is small and d2 is very small, so B2' is approximately 2d, and hence proportional to
the distance from the pole to the unit circle. Consequently a nominally logarithmic coding for B2' will be
an audibly meaningful coding.

*["R2", "d2", "t2" are printed on the line; R², d², t² are meant.]*

[5:48] Similarly, if the poles angle t is near zero, cos(t) will be approximately 1 − t2, hence

```
B1' = 2 + B1 = 2 − 2Rcos(t) = 2 − 2(1 − d)(1 − t2) = 2d + 2(1 − d)t2
```

which is approximately B2' + 2t2. One can thus logarithmically encode the difference between B1' and B2' and
accomplish the objective of approximating logarithmic frequency and resonance control.

*[cos t ≈ 1 − t²/2 would be the usual approximation; the patent writes 1 − t2. Printed as is.]*

[5:55] The actual decoding can be accomplished by means of either a ROM or a shifting technique. The latter
is demonstrated in FIG. 9 *[drawn as FIG. 8]*.

[5:58] In FIG. 9 *[drawn FIG. 8]*, the coefficient to be decoded is held in coefficient latch 41. In the case
of a static filter, this would be supplied from a memory holding the coefficients; in the case of a dynamic
filter, this would be the output of the coefficient math unit, which would add the lower coefficient to the
product of the interpolating variable times the coefficient differences. The most significant bits 42, (in
this case 4 bits are used), are routed through the shift decoding logic 43, to determine the degree of shift
in the barrel shifter 44. The resulting decoded coefficient is stored in latch 45. If block floating point
coefficients are used, the shift logic can additionally or alternatively output an exponent into exponent
latch 46. If the coefficient is a second coefficient (B2' or A2'), the result is directly passed to the final
output latch 47 through multiplexer 48, and exponent control logic 49 passes the exponent unchanged into
exponent output latch 50 if block floating point is used.

[6:29] If the coefficient is a first coefficient (which must be processed second), then adder 51 adds the
previously computed second coefficient as stored in 47 to the decoded difference in latch 45, and the result
passes through multiplexer 48 into the output latch. If block floating point is used, the exponent logic 49
must control barrel shifters 52 and 53 to normalize the addition.

[6:36] The coefficients are interpolated according to the formula

```
C(x) = Ca + x(Cb − Ca)
```

where x varies from zero to unity. This requires one multiply per coefficient, and with two coefficients each
for poles and zeros one can use the available four multipliers to make complete use of the required
hardware in a time division multiplexed system.

[6:43] The interpolating variable x is updated in the known manner of incrementing its value each sample
period, and clamping the result to a final known value when the clamp value is exceeded. This works well
when both the increment and clamp are computed externally as the dynamic sweep rate and destination.
However, this method fails when a slow controlling processor is generating the sweep variables, and a real
time command requires the change to a new destination using a sweep while the current sweep is in progress.
In this case, the slower processor must calculate new new [sic] sweep increment based on the current value
of the interpolating variable x and the destination value. Unfortunately, the interpolating variable x
continues to be incremented according to the old sweep while the slow processor calculates its result, with
an inaccurate (or even worse, the wrong signed) increment saved when the computation is done.

[6:61] The solution to this problem is another aspect of the present invention. Rather than supplying the
interpolating hardware with both a destination and increment, the interpolator is supplied with a
destination and an approach time in samples. The interpolator computes within a single sample period the
difference between the destination and current value, divides this by the time, and stores the result in
the increment.

[7:1] The preferred embodiment of this solution will now be described. The increment is stored in a
compressed value as indicated in Table I. The 8-bit value S, shift2, shift1, shift0, d3, d2, d1, d0 is
translated into a signed 16-bit linear value from −20408 [sic] to 2047, using the logic circuitry shown in
FIG. 10 *[drawn as FIG. 9]*. Sign bit S and data bits d3, d2, d1 and d0 are shifted in a sign-extended
matter [sic] by barrel shifter 60. The control of the shifter is performed by logic 61 to conform to the
translation defined in Table I below.

#### Table I (spans columns 7 and 8; crop: `patent_figs/table1.png`)

The first column is headed "16bitl F" as printed; F is the 3-bit shift field. `S*` is printed with an
asterisk that the patent does not define (see the decode below).

| 16bitl F | d15 | d14 | d13 | d12 | d11 | d10 | d9 | d8 | d7 | d6 | d5 | d4 | d3 | d2 | d1 | d0 | shift |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 7 | S | S | S | S | S | S* | d3 | d2 | d1 | d0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 6 | S | S | S | S | S | S | S* | d3 | d2 | d1 | d0 | 0 | 0 | 0 | 0 | 0 | 1 |
| 5 | S | S | S | S | S | S | S | S* | d3 | d2 | d1 | d0 | 0 | 0 | 0 | 0 | 2 |
| 4 | S | S | S | S | S | S | S | S | S* | d3 | d2 | d1 | d0 | 0 | 0 | 0 | 3 |
| 3 | S | S | S | S | S | S | S | S | S | S* | d3 | d2 | d1 | d0 | 0 | 0 | 4 |
| 2 | S | S | S | S | S | S | S | S | S | S | S* | d3 | d2 | d1 | d0 | 0 | 5 |
| 1 | S | S | S | S | S | S | S | S | S | S | S | S* | d3 | d2 | d1 | d0 | 6 |
| 0 | S | S | S | S | S | S | S | S | S | S | S | S | d3 | d2 | d1 | d0 | 6 |

NOTE: Adder carry is set for increment add during positive increments. This means that a zero increment
cannot be reached (instead, set target to correct value), and that the increment ranges for positive and
negative are the same (0001-07c1 = ffff − f83f). This simplifies automatic decoding from target-value, as
zero cannot be reached on underflow.

*[Decode implied by Table I, checked against 20 rows of Table II: `S*` behaves as NOT S, a hidden leading
bit. Positive (S = 0): F = 0 gives d; F ≥ 1 gives (16 + d) << (F − 1); maximum 31 << 6 = 0x7C0, plus the
forced carry 0x7C1, matching the note's 0001-07c1. Negative (S = 1): F = 0 gives d − 16; F ≥ 1 gives
(d − 32) << (F − 1); range 0xF800 (−2048) to 0xFFFF (−1). Examples from Table II: difference 0x0F800 over 32
samples → code 0x7f → 1984 = 0xF800/32; difference −32768 (0x18000) over 32 → 0xe0 → −1024; over 4096 → 0x88
→ −8. The negative range per the table bits (down to 0xF800) does not match the note's "f83f"; I cannot
resolve that from the patent.]*

[7:25] The clamp value is similarly an 8-bit value, which is translated into a 16-bit linear value by simply
duplicating the bits twice. For example, 5a hexidecimal [sic] would translate to 5a5a hexidecimal. FIG. 11
*[drawn as FIG. 10]* illustrates the logic used to add update the 16-bit interpolating variable using the
8-bit increment and clamp.

[7:31] In FIG. 11 *[drawn FIG. 10]*, register 70 holds the current value of the 16-bit interpolating
variable. Register 71 holds the 8-bit clamp value, and register 72 holds the 8-bit encoded increment value.
Decoder 73 decodes the 8-bit increment into a 16-bit signed linear value. Signal 74 controls the two phases of
the operation. In the first phase, multiplexers 75 and 76 select as the inputs to the added [sic] 77 the
current value and linear increment, and multiplexer 78 is controlled by logic unit 79 so that the incremented
current value appears in output register 80. Logic 79 also notes the state of the adder carry signal 81 with
respect to the increment sign bit 82, such that an arithmetic overflow can be detected during this phase. In
the second phase, multiplexers 75 and 76 select the inverted clamp value and incremented current value
respectively, and hence adder 77 determines the difference between these values. The state of adder carry
signal 81 and increment sign bit 82 now allows control logic 79 to determine if the incremented current
value has exceeded the current value [sic; the clamp is meant, per the next clause]; that is, if the increment
sign is positive and the incremented current value is greater than the clamp, or if the increment sign is
negative and the incremented current value is less than the clamp. If the incremented current value exceeds
the clamp, or if overlow [sic] occurred during phase 1, then logic 79 causes multiplexer 78 to select the
16-bit clamp value; otherwise the incremented current value is selected. At the end of the second phase,
register 80 contains the properly computed new current value.

[8:8] FIG. 12 *[drawn as FIG. 11]* illustrates the circuitry required to implement the automatic computation
of the required encoded 8-bit increment, based on a destination (clamp) value, current value and approach
time. The approach time is selected from eight possible values, each a power or [sic] two greater, with the
lowest being 32 sample periods. This approach time is stored in register 90. The current value of the
interpolator is stored in register 91, and the destination is stored in register 92. Adder 93 computes the
difference between the destination and current value, which is then stored in register 94. Logic 95
computes the required increment to reach the destination in approach time samples. The 8-bit value is
output on signal lines 96 when enable signal 97 is active.

[8:36] FIG. 13 *[drawn as FIGS. 12A and 12B]* shows the detailed logic with block 95. Logic 100 determines
the first set bit, that is, the first bit different from the sign bit. Logic 101 determines the shift bits of
the required increment based on the first set bit and the selected approach time indicated by CUDR2N,
CUDR1N and CUDR0N. Signal 102 indicates that the first set bit is greater than or equal to the selected
approach time, which determines the function of data bit barrel shifter 103. If the first set bit is greater
than or equal to the selected approach time, the first set bit determines the shifting of the data bits.
Otherwise, the selected approach time determines the shifting of the bits. Enable signal 104 simply allows
the value to be driven onto the circuit outputs. When the enable is invalid, all outputs are logical 1's.
Table II below illustrates the resulting outputs as a function of the difference input and the selected
approach time.

#### Table II, "ARMAdillo Interpolator Increment Decoding" (column 8 bottom to column 10)

Difference and Binary are the 17-bit difference (destination − current); FSB is the first-set-bit position;
the `@` columns are the 8-bit increment code (hex) for each of the eight approach times. The patent gives no
unit for the `@` labels. `x` = don't care. Printed range typos are kept and flagged in the footnotes.

| Difference | Binary | FSB | @.6 | @1.2 | @2.4 | @4.8 | @9.6 | @20 | @40 | @80 |
|---|---|---|---|---|---|---|---|---|---|---|
| 0ffff-0f800 | 011111xxxxxxxxxxx | 15 | 7f | 6f | 5f | 4f | 3f | 2f | 1f | 0f |
| 0f7ff-0f000 | 011110xxxxxxxxxxx | 15 | 7e | 6e | 5e | 4e | 3e | 2e | 1e | 0f |
| 0efff-0e800 | 011101xxxxxxxxxxx | 15 | 7d | 6d | 5d | 4d | 3d | 2d | 1d | 0e |
| 087ff-08000 | 010000xxxxxxxxxxx | 15 | 70 | 60 | 50 | 40 | 30 | 20 | 10 | 08 |
| 07fff-07c00 | 0011111xxxxxxxxxx | 14 | 6f | 5f | 4f | 3f | 2f | 1f | 0f | 07 |
| 07bff-07800 | 0011110xxxxxxxxxx | 14 | 6e | 5e | 4e | 3e | 2e | 1e | 0f | 07 |
| 077ff-07400 | 0011101xxxxxxxxxx | 14 | 6d | 5d | 4d | 3d | 2d | 1d | 0e | 07 |
| 043ff-04000 | 0010000xxxxxxxxxx | 14 | 60 | 50 | 40 | 30 | 20 | 10 | 08 | 04 |
| 03fff-03e00 | 00011111xxxxxxxxx | 13 | 5f | 4f | 3f | 2f | 1f | 0f | 07 | 03 |
| 03dff-03c00 | 00011110xxxxxxxxx | 13 | 5e | 4e | 3e | 2e | 1e | 0f | 07 | 03 |
| 021ff-02000 | 00010000xxxxxxxxx | 13 | 50 | 40 | 30 | 20 | 10 | 08 | 04 | 02 |
| 01fff-01100 (1) | 000011111xxxxxxxx | 12 | 4f | 3f | 2f | 1f | 0f | 07 | 03 | 01 |
| 01eff-01e00 | 000011110xxxxxxxx | 12 | 4e | 3e | 2e | 1e | 0f | 07 | 03 | 01 |
| 010ff-01000 | 000010000xxxxxxxx | 12 | 40 | 30 | 20 | 10 | 08 | 04 | 02 | 01 |
| 00fff-00f80 | 0000011111xxxxxxx | 11 | 3f | 2f | 1f | 0f | 07 | 03 | 01 | 00 |
| 00f7f-00f00 | 0000011110xxxxxxx | 11 | 3e | 2e | 1e | 0f | 07 | 03 | 01 | 00 |
| 0087f-00800 | 0000010000xxxxxxx | 11 | 30 | 20 | 10 | 08 | 04 | 02 | 01 | 00 |
| 007ff-007c0 | 00000011111xxxxxx | 10 | 2f | 1f | 0f | 07 | 03 | 01 | 00 | 00 |
| 007bf-00780 | 00000011110xxxxxx | 10 | 2e | 1e | 0f | 07 | 03 | 01 | 00 | 00 |
| 0077f-00740 | 00000011101xxxxxx | 10 | 2d | 1d | 0e | 07 | 03 | 01 | 00 | 00 |
| 0073f-00700 | 00000011100xxxxxx | 10 | 2c | 1c | 0e | 07 | 03 | 01 | 00 | 00 |
| 006ff-006c0 | 00000011011xxxxxx | 10 | 2b | 1b | 0d | 06 | 03 | 01 | 00 | 00 |
| 006bf-00680 | 00000011010xxxxxx | 10 | 2a | 1a | 0d | 06 | 03 | 01 | 00 | 00 |
| 0067f-00640 | 00000011001xxxxxx | 10 | 29 | 19 | 0c | 06 | 03 | 01 | 00 | 00 |
| 0063f-00600 | 00000011000xxxxxx | 10 | 28 | 18 | 0c | 06 | 03 | 01 | 00 | 00 |
| 005ff-005c0 | 00000010111xxxxxx | 10 | 27 | 17 | 0b | 05 | 02 | 01 | 00 | 00 |
| 005bf-00580 | 00000010110xxxxxx | 10 | 26 | 16 | 0b | 05 | 02 | 01 | 00 | 00 |
| 0057f-00540 | 00000010101xxxxxx | 10 | 25 | 15 | 0a | 05 | 02 | 01 | 00 | 00 |
| 0053f-00500 | 00000010100xxxxxx | 10 | 24 | 14 | 0a | 05 | 02 | 01 | 00 | 00 |
| 004ff-004c0 | 00000010011xxxxxx | 10 | 23 | 13 | 09 | 04 | 02 | 01 | 00 | 00 |
| 004bf-00480 | 00000010010xxxxxx | 10 | 22 | 12 | 09 | 04 | 02 | 01 | 00 | 00 |
| 0047f-00440 | 00000010001xxxxxx | 10 | 21 | 11 | 08 | 04 | 02 | 01 | 00 | 00 |
| 0043f-00400 | 00000010000xxxxxx | 10 | 20 | 10 | 08 | 04 | 02 | 01 | 00 | 00 |
| 003ff-003e0 | 000000011111xxxxx | 9 | 1f | 0f | 07 | 03 | 01 | 00 | 00 | 00 |
| 0021f-00200 | 000000010000xxxxx | 9 | 10 | 08 | 04 | 02 | 01 | 00 | 00 | 00 |
| 001ff-001e0 | 000000001111xxxxx | 8 | 0f | 07 | 03 | 01 | 00 | 00 | 00 | 00 |
| 0011f-00100 | 000000001000xxxxx | 8 | 08 | 04 | 02 | 01 | 00 | 00 | 00 | 00 |
| 000ff-000e0 | 000000000111xxxxx | <8 | 07 | 03 | 01 | 00 | 00 | 00 | 00 | 00 |
| 0001f-00000 | 000000000000xxxxx | <8 | 00 | 00 | 00 | 00 | 00 | 00 | 00 | 00 |
| 1000-107ff (2) | 100000xxxxxxxxxxx | 15 | f0 | e0 | d0 | c0 | b0 | a0 | 90 | 80 |
| 10800-10fff | 100001xxxxxxxxxxx | 15 | f1 | e1 | d1 | c1 | b1 | a1 | 91 | 80 |
| 11000-117ff | 100010xxxxxxxxxxx | 15 | f2 | e2 | d2 | c2 | b2 | a2 | 92 | 81 |
| 17800-17fff | 101111xxxxxxxxxxx | 15 | ff | ef | df | cf | bf | af | 9f | 87 |
| 18000-183ff | 1100000xxxxxxxxxx | 14 | e0 | d0 | c0 | b0 | a0 | 90 | 80 | 88 |
| 18400-187ff | 1100001xxxxxxxxxx | 14 | e1 | d1 | c1 | b1 | a1 | 91 | 80 | 88 |
| 18800-18bff | 1100010xxxxxxxxxx | 14 | e2 | d2 | c2 | b2 | a2 | 92 | 81 | 88 |
| 1bc00-1bfff | 1101111xxxxxxxxxx | 14 | ef | df | cf | bf | af | 9f | 87 | 8b |
| 1c000-1c1ff | 11100000xxxxxxxxx | 13 | d0 | c0 | b0 | a0 | 90 | 80 | 88 | 8c |
| 1c200-1c3ff | 11100001xxxxxxxxx | 13 | d1 | c1 | b1 | a1 | 91 | 80 | 88 | 8c |
| 1deff-1dfff (3) | 11101111xxxxxxxxx | 13 | df | cf | bf | af | 9f | 87 | 8b | 8d |
| 1e000-1e0ff | 111100000xxxxxxxx | 12 | c0 | b0 | a0 | 90 | 80 | 88 | 8c | 8e |
| 1e100-1e1ff | 111100001xxxxxxxx | 12 | c1 | b1 | a1 | 91 | 80 | 88 | 8c | 8e |
| 1ef00-1efff | 111101111xxxxxxxx | 12 | cf | bf | af | 9f | 87 | 8b | 8d | 8e |
| 1f000-1f07f | 1111100000xxxxxxx | 11 | b0 | a0 | 90 | 80 | 88 | 8c | 8e | 8f |
| 1f800-1f0ff (4) | 1111100001xxxxxxx | 11 | b1 | a1 | 91 | 80 | 88 | 8c | 8e | 8f |
| 1f780-1f7ff | 1111101111xxxxxxx | 11 | bf | af | 9f | 87 | 8b | 8d | 8e | 8f |
| 1f800-1f83f | 11111100000xxxxxx | 10 | a0 | 90 | 80 | 88 | 8c | 8e | 8f | 8f |
| 1f840-1f87f | 11111100001xxxxxx | 10 | a1 | 91 | 80 | 88 | 8c | 8e | 8f | 8f |
| 1f880-1f8bf | 11111100010xxxxxx | 10 | a2 | 92 | 81 | 88 | 8c | 8e | 8f | 8f |
| 1f8c0-1f8ff | 11111100011xxxxxx | 10 | a3 | 93 | 81 | 88 | 8c | 8e | 8f | 8f |
| 1f900-1f93f | 11111100100xxxxxx | 10 | a4 | 94 | 82 | 89 | 8c | 8e | 8f | 8f |
| 1f940-1f97f | 11111100101xxxxxx | 10 | a5 | 95 | 82 | 89 | 8c | 8e | 8f | 8f |
| 1f980-1f9bf | 11111100110xxxxxx | 10 | a6 | 96 | 83 | 89 | 8c | 8e | 8f | 8f |
| 1f9c0-1f9ff | 11111100111xxxxxx | 10 | a7 | 97 | 83 | 89 | 8c | 8e | 8f | 8f |
| 1fa00-1fa3f | 11111101000xxxxxx | 10 | a8 | 98 | 84 | 8a | 8d | 8e | 8f | 8f |
| 1fa40-1fa7f | 11111101001xxxxxx | 10 | a9 | 99 | 84 | 8a | 8d | 8e | 8f | 8f |
| 1fa80-1fabf | 11111101010xxxxxx | 10 | aa | 9a | 85 | 8a | 8d | 8e | 8f | 8f |
| 1fac0-1faff | 11111101011xxxxxx | 10 | ab | 9b | 85 | 8a | 8d | 8e | 8f | 8f |
| 1fb00-1fb3f | 11111101100xxxxxx | 10 | ac | 9c | 86 | 8b | 8d | 8e | 8f | 8f |
| 1fb40-1fb7f | 11111101101xxxxxx | 10 | ad | 9d | 86 | 8b | 8d | 8e | 8f | 8f |
| 1fb80-1fbbf | 11111101110xxxxxx | 10 | ae | 9e | 87 | 8b | 8d | 8e | 8f | 8f |
| 1fbc0-1fbff | 11111101111xxxxxx | 10 | af | 9f | 87 | 8b | 8d | 8e | 8f | 8f |
| 1fc00-1fc1f | 111111100000xxxxx | 9 | 90 | 80 | 88 | 8c | 8e | 8f | 8f | 8f |
| 1fde0-1fdff | 111111101111xxxxx | 9 | 9f | 87 | 8b | 8d | 8e | 8f | 8f | 8f |
| 1fe00-1fe1f | 111111110000xxxxx | 8 | 80 | 88 | 8c | 8e | 8f | 8f | 8f | 8f |
| 1fee0-1feff | 111111110111xxxxx | 8 | 87 | 8b | 8d | 8e | 8f | 8f | 8f | 8f |
| 1ff00-1ff1f | 111111111000xxxxx | <8 | 88 | 8c | 8e | 8f | 8f | 8f | 8f | 8f |
| 1ffe0-1ffff | 111111111111xxxxx | <8 | 8f | 8f | 8f | 8f | 8f | 8f | 8f | 8f |

Range misprints (the Binary column is consistent and gives the intended range): (1) binary 000011111 means
01f00-01fff; (2) 10000-107ff; (3) 1de00-1dfff; (4) 1f080-1f0ff.

*[Reading of the table, mine: each column halves the increment, consistent with approach times of
32·2^k samples. Results are floor(difference / time) encoded per Table I, saturating at the largest code of
each sign (0x7f, and 0x8f = −1 for small negative differences).]*

[9:57] The foregoing description of the preferred embodiment of the invention has been presented for purposes
of illustration and description. It is not intended to be exhaustive or to limit the invention to the
precise form disclosed, and many modifications and variations are possible in light of the above teaching.
The preferred embodiment was chosen and described in order to best explain the principles of the invention
and its practical applications to thereby enable others skilled in the art to best utilize the invention and
various embodiments and with various modifications as are suited to the particular use contemplated. It is
intended that the scope of the invention be defined only by the claims appended hereto.

---

## 3. Claims (columns 10 to 12)

What is claimed is:

**1.** [10:61] A dynamic digital audio filter comprising
adding means for receiving an input audio signal,
first and second unit delay means connected to an output of said adding means for forming first and second
unit output delays each having a first bit width, respectively, wherein the first unit delay is one time unit
and wherein the second unit delay is two time units,
means for providing a first unsigned coefficient and a second unsigned coefficient,
unsigned multiplying means receiving said first and second output delays, wherein the inputs to said
multiplying means are a truncation of said first and second output delays each having a truncated bit width
such that the truncated bit width is less than said first bit width so as to form a first truncated delay and
a second truncated delay,
said unsigned multiplying means including means for providing said first unsigned coefficient times the first
truncated delay and second unsigned coefficient times the second truncated delay,
means for shifting the output of said first delay means to multiply by two,
means for forming a 2's complement of the output of said second delay means, and
said adding means including means for forming an output signal which is a sum of said input audio signal,
the output of said first delay means shifted to multiply by two, the 2's complement output of said second
delay means, the first unsigned coefficient times the first truncated delay, and the second unsigned
coefficient times the second truncated delay.

**2.** [11:26] A digital filter as in claim 1 including means for forming a difference between said first
delay and said input audio signal and wherein said sum includes the first unsigned coefficient times the
difference between said first delay and said input audio signal and including means for forming a difference
between said second delay and said input audio signal and wherein said sum includes the second unsigned
coefficient times the difference between said second delay and said input audio signal.

**3.** [11:36] A digital filter as in claim 1 further including a finite impulse response filter.

**4.** [11:38] A digital filter as in claim 1 including first storage means and wherein said second
coefficient is stored in a logarithmically compressed format in said first storage means.

**5.** [12:3] A digital filter as in claim 4 including second storage means and including a third coefficient
stored in said second storage means in a logarithmically compressed format representative of a difference
between said first and second coefficients.

**6.** [12:8] A digital filter as in claim 5 including means for interpolating between said first and second
coefficients.

**7.** [12:10] In a dynamic digital audio filter for filtering an input audio signal wherein said filter
includes first and second unit delays for forming first and second unit output delays having a first bit
width wherein the first unit delay is one time unit and wherein the second unit delay is two time units, an
unsigned multiplying means wherein an input to said multiplying means is a truncation of said first and second
output delays each having a truncated bit width such that the truncated bit widths are less than said first
bit width so as to form first and second truncated delays, an adding means, the method comprising the steps of
providing a first unsigned coefficient and a second unsigned coefficient, providing said first unsigned
coefficient times the first truncated delay and said second unsigned coefficient times the second truncated
delay,
receiving said first and second truncated delays,
shifting the output of said first unit delay to multiply by two,
forming a 2's complement of the output of said second unit delay, and
forming an output delayed signal which is a sum of said input audio signal, the output of said first unit
delay shifted to multiply by two, the 2's complement output of said second unit delay, a first unsigned
coefficient times the first truncated delay, and a second unsigned coefficient times the second truncated
delay.

*[Observations on the claim language, mine: claim 1 adds both products with a plus sign; the sign is carried
by the data (the reservation-table note: "the B multiplies will be negative Z data"). Claim 2 adds the
difference terms but does not say the direct input term of claim 1 is removed; the removal is shown only in
FIG. 5 and by the IDC gating in the reservation table. Claim 6 reads "interpolating between said first and
second coefficients", not between coefficient sets or frames. No claim covers the destination-plus-approach
-time interpolator, the increment code, or the coefficient decoder hardware.]*

---

## 4. Figures

Drawing sheets are pages 2 to 12 of the PDF. Sheets 4 to 6 and 8 to 11 are drawn landscape.

### Sheet 1: FIG. 1 and FIG. 2 (prior art)

- FIG. 1: direct-form all-pole biquad. `Xn` into a summing node whose output is `Yn`; `Yn` feeds a chain of two
  `Z^-1` blocks; the tap after the first delay is multiplied by `−B1`, after the second by `−B2`, both summed
  back into the node.
- FIG. 2: the same recursion followed by a second summing node: the node output (top line) plus `A1`×(first
  delay tap) plus `A2`×(second delay tap) gives `Yn`. Direct form II, the FIR taps share the recursion's
  delays.

### Sheet 2: FIG. 3 and FIG. 4 (`patent_figs/fig03_04.png`)

- FIG. 3 (also the front-page drawing): one summing node with inputs `Xn`; two separate unmultiplied lines
  from the first delay tap (2·Z1); the second delay tap on a `−` input (−Z2); `−B1'`×Z1; `B2'`×Z2. Output `Yn`
  is the node output, which also feeds the delay chain.
- FIG. 4: three summing nodes. Left node: `Xn` + (`−B1'`×Z1) + (`B2'`×Z2). Its output runs along the top to
  the right-hand output node and, through a branch, into a centre node that also receives two lines from the
  Z1 tap and Z2 on a `−` input; the centre node output enters the first `Z^-1`. Right-hand node: top line +
  (`A1'`×Z1) + (`−A2'`×Z2) = `Yn`. So the delay input is (left sum) + 2Z1 − Z2, while the output is (left sum)
  + A1'Z1 − A2'Z2; the FIR taps are added to the multiplier sum, not to the full delay input. This matches the
  reservation table (`MAout = AR + a1'Z1 − a2'Z2`).

### Sheet 3: FIG. 5 (`patent_figs/fig05.png`)

Same three-node layout as FIG. 4, with two extra two-input adders in front of the B multipliers. Each extra
adder takes a delay tap on its `+` input (Z1 for the `−B1'` multiplier, Z2 for the `B2'` multiplier) and
`Xn` on its `−` input, so the multipliers see `Z1 − Xn` and `Z2 − Xn`. `Xn` enters the drawing at the left
and goes only to those two `−` inputs (the vertical line it crosses near the left edge, from the `B2'`
product to the top-left node, has no junction dot). The top-left node therefore sums only the two products.
The `A1'` and `−A2'` taps still read Z1 and Z2 directly. No switch or control signal is drawn; the figure is
one fixed structure.

### Sheet 4: FIG. 6, datapath (`patent_figs/fig06.png`)

Blocks and connections as drawn:

- ADDER 12 with inputs 14 (from MUX 15) and 17 (from MUX 16). Its output feeds the MUX in front of ACC 10,
  the MUX in front of ACC 11, and (via a long line along the top) the unnumbered MUX in front of multiplier
  input LATCH 18.
- ACC 10 has a true and an inverted (bubble) output. Its input MUX selects among INPUT SIGNAL, the adder
  output, and a line from ACC 10's own output node.
- ACC 11 with its own input MUX (adder output and its own output fed back).
- DELAY MEMORY 22: its output loops round to the inputs of LATCH 13 and LATCH 20.
- LATCH 13: inverted output to MUX 15 (this is "negative delay memory 2's data"); a true output line also
  leaves the latch towards the multiplier-input MUX.
- LATCH 20: inverted output to MUX 15 ("negative delay memory 1's data"); true output to MUX 16 as line 21
  and to the multiplier-input MUX. Line 21 is described in the text as "delay memory 1's value shifted left
  one bit"; the shift is not drawn.
- MUX 15 inputs: inverted LATCH 13, inverted LATCH 20, the ACC 11 output bus, and the multiplier 24
  output. MUX 16 inputs: line 21 and two lines from the ACC 10 true and inverted outputs (input signal or
  accumulator, and its negative). The ACC 11 output bus also feeds the DELAY MEMORY 22 input and ACC 11's
  own input MUX.
- Unnumbered MUX → LATCH 18 → multiplier 24 signal input; COEF. MEMORY 23 → LATCH 19 → multiplier 24
  coefficient input. The multiplier output goes to MUX 15.

No mux select lines, clock, control signals, address generators, or sequencer are drawn anywhere in FIG. 6.
The text never mentions the MUX in front of LATCH 18; from the drawing it is the path that lets the raw delay
values reach the multiplier for the `a2'Z2`, `a1'Z1` products of cycles 7 and 8 (my reading). How the delay
and coefficient memories are addressed is not shown.

### Sheet 5: FIG. 7, multiplier block (`patent_figs/fig07.png`; text calls it "FIG. 8")

SIGNAL INPUT (31) → SIGNED INPUT and COEFFICIENT MANTISSA (33) → UNSIGNED INPUT of the multiplier first
stage 32; pipeline latch 35; second stage 36; BARREL SHIFTER 37; LATCH 38 → MULTIPLIER PRODUCT. COEFFICIENT
EXPONENT → LATCH 34 → shift control of barrel shifter 37. No bit widths are marked.

### Sheet 6: FIG. 8, coefficient decoder (`patent_figs/fig08.png`; text calls it "FIG. 9")

ENCODED CO-EFFICIENT → LATCH 41. The high bits (42) go to LOGIC 43, which controls BARREL SHIFTER 44 and
writes an exponent to LATCH 46; the remaining latch-41 bits are the shifter's data input. Shifter 44 →
LATCH 45. LATCH 45 feeds MUX 48 directly and also BARREL SHIFTER 52 → ADDER 51. LATCH 47 (output) feeds back
through BARREL SHIFTER 53 to the other ADDER 51 input. ADDER 51 → MUX 48 → LATCH 47 (coefficient output).
LATCH 46 → LOGIC 49 → LATCH 50 (exponent output); LOGIC 49 also controls shifters 52 and 53. No bit widths
are marked except the text's "4 bits" for 42.

### Sheet 7: FIG. 9, increment decoder (`patent_figs/fig09.png`; text calls it "FIG. 10")

Gate-level. Dashed block 61 (shift control): inputs `SHIFT2=D6`, `SHIFT1=D5`, `SHIFT0=D4`, `ENB`, `ENBN`,
`S=D7`; internal `D6N`, `D6B`, `D5N`, `D5B`, `SH1N`, `SH1`, `SN`, `ENBB`; outputs `ESH0`, `ESH1` (written
`E5H0`, `E5H1` where they enter block 60). Dashed block 60 (barrel shifter 60): data inputs `D3`, `D2`, `D1`,
`D0` and the sign; a column of ten 2:1 cells labelled `D5N`/`D5`; an `SXA` term and a gate labelled `5C`;
outputs `Z15N` to `Z11N` (one gate labelled `Z11N–Z15N`, the sign extension) and `Z10N` down to `Z0N`. So the
code byte is S = bit 7, shift = bits 6..4, d = bits 3..0, as in the text. The gate-level detail was not
traced further.

### Sheet 8: FIG. 10, interpolating-variable update (`patent_figs/fig10.png`; text calls it "FIG. 11")

REG. 71 (clamp) → MUX 75 through two inverted (bubble) inputs, i.e. the byte duplicated and inverted, and also
twice into MUX 78. REG. 70 (current value) → MUX 75. REG. 72 (increment) → DECODER 73 → MUX 76; the sign line
82 is taken from REG. 72 to CONTROL LOGIC CIRCUIT 79. MUX 75 and MUX 76 → ADDER 77; carry 81 → logic 79;
ADDER 77 → MUX 78 → OUTPUT REG. 80; REG. 80 output feeds back to MUX 76 and MUX 78. CONTROL SIGNAL 74 (the
phase) goes to MUX 75, MUX 76 and logic 79; logic 79 drives the MUX 78 select.

### Sheet 9: FIG. 11, increment computation (`patent_figs/fig11.png`; text calls it "FIG. 12")

REG. 91 (current value; inverted output) and REG. 92 (destination; its output also tied to a second adder
input, plausibly the carry-in for the subtraction) → ADDER 93 → REG. 94 (two lines in, the extra one from the
adder's top, plausibly the carry/sign bit) → LOGIC CIRCUIT 95 → output 96. REG. 90 (approach time) and ENABLE
SIGNAL 97 → logic 95.

### Sheets 10 and 11: FIGS. 12A and 12B, logic 95 (`patent_figs/fig12a.png`, `fig12b.png`; text calls them "FIG. 13")

- FIG. 12A: inputs `15N` to `9N`; XOR gates produce `C15` to `C9` (each bit compared with the next, i.e.
  with the sign). Block 100 (first set bit) produces `FSB2`, `FSB1`, `FSB0`. Block 101 combines these with the
  approach-time select, drawn as `CUDRN2`, `CUDRN`, `CUDR0N` (the text says `CUDR2N`, `CUDR1N`, `CUDR0N`).
  Gate 102 produces `FGEUDR` (first set bit greater than or equal to the approach time). Outputs `VP17N`,
  `VP16N`, `VP15N`, `VP14N` (the high nibble of the code).
- FIG. 12B: block 103, the data barrel shifter, inputs `15N` to `5N`, controls `SHFT4`/`SHFT4N`,
  `SHFT2`/`SHFT2N`, `SHFT1N`, `SHFT1E`, `D1FT1E`; outputs `VPI3N` to `VPI0N` (the four data bits). The shift
  controls come from `FGEUDR`, `F5B2`/`CUDR2`, `F5B1`/`CUDR1`, `F5B0`/`CUDR0` and `EVDI`. Numeral 104 points
  at a line near the `FGEUDR` buffers; I could not trace which line the text's "enable signal 104" is.

---

## 5. Answers

### (a) What selects the DC-forced routing (FIG. 5, claim 2)?

The patent shows a 1-bit signal and nothing about where it comes from.

- Text: in cycle 1 adder input 17 gets "the input signal from accumulator 10, or zero, depending on the DC
  gain stabilization desired, through multiplexer 16" [4:15-19]; cycle 2 "input 17 stays the same"
  [4:24-27]; cycle 4 input 17 is "either negative the input signal as routed from accumulator 10, or zero,
  depending on the DC gain stabilitzation requirement" [4:37-44]. "The structures in FIGS. 4 and 5 are easily
  switched between" [3:66-68].
- Reservation table: the selection is written as the signals `DC` and `IDC`: `(SA = Xn) & DC` on input 17
  in cycles 1 and 2, `(SA = Xn') & IDC` with carry-in `IDC` in cycle 4. With DC = 1 the input goes into both
  multiplier operands and is removed from the direct path; with DC = 0 the operands are `−Z2`, `−Z1` and the
  input is added once in cycle 4. So it is one bit acting on mux 16 in three of the eight cycles plus the
  carry-in of cycle 4.
- Not stated anywhere: whether DC is a mode register, a per-section flag, part of the coefficient word, or
  microcode. The patent has no control word, control store, or register map at all (see c). It is not said to
  be per section. Since the table describes one section's eight cycles and the signal enters inside them,
  per-section control would be possible in this pipeline, but that is an inference, not something the patent
  says.
- Claim 2 only adds the difference terms; it does not mention a selector or the removal of the direct input.

### (b) Normalisation of the zero section, gain stage, output scaling word?

None in the patent.

- The only gain normalisation is the DC-forced pole recursion: "the fixing of the DC gain of the IIR filter
  to unity regardless of coefficient choice" [abstract; 1:58-62]; "force the gain of the filter at zero
  frequency to be unity ... FIG. 5" [3:63-66].
- The numerator is never normalised. In FIG. 5 the `A1'`, `−A2'` taps read Z1, Z2 directly, and in the
  reservation table cycles 7 and 8 multiply `a2'Z2`, `a1'Z1` with no DC term. Worked out from the table (not
  stated in the patent): in DC mode at DC, Z1 = Z2 = Xn, AR = 0 and `MAout = (a1' − a2')·Xn`, so the zeros'
  DC gain is `A1' − A2'`, and the text's "gain of the filter at zero frequency ... unity" holds only for the
  recursive part.
- There is no gain multiply and no scaling word: the table has exactly four products (`b2'`, `b1'`, `a2'`,
  `a1'`) [4:1-4], and coefficient memory 23 supplies only those. The only "scaling" is the block-floating
  -point exponent of the multiplier ("adjusts for the previous coefficient's exponent", [5:19-33]) and the
  coefficient decoder's exponent [5:65-6:35], both internal to forming a product.
- The patent never defines `A1'`, `A2'` in words. The structure (FIG. 4 and the `MAout` formula, which adds
  `a1'Z1 − a2'Z2` to `AR = Yn − 2Z1 + Z2`) makes the numerator `1 + (A1' − 2)z^-1 + (1 − A2')z^-2`, i.e. the
  same transform as the poles; and the decoder handles "(B2' or A2')" and the corresponding "first
  coefficient" identically [6:24-33].

### (c) Microcode, sequencer, control store?

None is described.

- Everything about sequencing is: "The filter operates in eight states" [4:12-13], the cycle-by-cycle
  description [4:15-65], and the reservation table. The output of cycle 8 "is the filter output and hence
  the input to the next stage" [4:63-65], and the table's last row `MAout = Xn`. Up to 128th order on one IC
  [2:3-6]. The four multiplies serve interpolation "in a time division multiplexed system" [6:39-42].
- FIG. 6 draws no mux select lines and no controller. FIG. 10 has "control signal 74" for the two phases of
  the interpolator update [7:35-36] and control logic 79; FIG. 11 has "enable signal 97" [8:35]. Those are the
  only control signals named, besides DC/IDC in the table.
- The words "voice", "channel", "microcode", "sequencer" and "control word" do not occur. How sections or
  voices are interleaved, how memories are addressed, and whether there is one interpolating variable per
  section, per filter or per coefficient pair are not described.
- Coefficient interpolation: the "coefficient math unit ... would add the lower coefficient to the product
  of the interpolating variable times the coefficient differences" [5:61-64]; `C(x) = Ca + x(Cb − Ca)`, one
  multiply per coefficient, four coefficients per section filling the four free multiplier slots [4:3-8;
  6:36-42]. The decoder needs the second coefficient (`B2'`/`A2'`) before the first [6:24-33], consistent with
  the table's order B2', B1', A2', A1'. The interpolating variable is updated "each sample period" [6:43-44] by
  the two-phase logic of FIG. 10 [7:31-8:7].

### (d) Saturation, clipping, overflow handling in the patent

- Audio path: none. No saturation, clipping, limiting or overflow handling of the accumulators, the delay
  memory or the output is described. The only related statements are about dynamic range: the background's
  "extreme dynamic range problems ... with very low frequency passbands" [1:24-27], which the DC-forced form
  is said to eliminate [1:58-62], and block floating point coefficients "to achieve a greater range"
  [5:19-21].
- The reservation-table notes concern sign and carry conventions ("we cannot add two complements as a carry
  in of 2 is required"), not overflow.
- Interpolator only: overflow of the 16-bit update is detected from adder carry 81 and increment sign 82
  [7:40-43]; on overflow, or when the clamp is passed, mux 78 selects the 16-bit clamp value [8:1-5]. The
  variable is "clamp[ed] ... to a final known value when the clamp value is exceeded" [6:43-46]. Table I
  note: a zero increment is unreachable and "zero cannot be reached on underflow". In Table II the code
  saturates at 0x7f and 0x8f. When the increment encoder's enable is invalid, all outputs are 1's [8:48-50].
- Saturation placement (into the delays, extended-headroom accumulator) is from Rossum's paper, not from this
  patent.

### (e) Word widths stated

| Item | Width in the patent | Where |
|---|---|---|
| Linear multiplier coefficient | "16 bit coefficients are adequate" | abstract; [1:54-56] |
| Compressed coefficient | "as few as 8 bits" | abstract; [1:63-66] |
| Coefficient decoder shift field | "4 bits" (the most significant bits 42) | [5:64-66] |
| Multiplier data | "similar to the input and output data resolution" (no number); multiplier inputs are truncated to less than the delay width (no numbers) | abstract; [1:56-58]; claims 1, 7 |
| Interpolating variable x | 16 bits | [7:29-32] |
| Clamp / destination | 8 bits, expanded to 16 by duplication | [7:25-28] |
| Increment | 8 bits (S, shift2..0, d3..d0), decoded to 16-bit signed, stated range "−20408 [sic] to 2047" | [7:2-7]; Table I |
| Difference in increment computation | 17 bits (from the Table II binary column) | Table II |
| Approach time | 8 values, 32 · 2^k samples, 3-bit select CUDR2N..CUDR0N | [8:25-28; 8:40-41] |
| Prior art, for comparison | 24-bit coefficients, 24-bit multiplier data | [1:18-24; 1:36-39] |
| Delay memory, accumulators, product, output | not stated | |
| Total code width of the interpolated coefficient, mantissa width, exponent width of the multiplier | not stated | |

### (f) Coefficient interpolation

- On the encoded coefficients: "not through true logarithmic interpolation, but rather through linear
  interpolation of approximately logarithmically encoded coefficients" [5:34-39]; in a dynamic filter the
  decoder's input latch 41 receives "the output of the coefficient math unit", i.e. interpolation precedes
  decoding [5:58-64].
- Linear: `C(x) = Ca + x(Cb − Ca)`, x from 0 to 1, one multiply per coefficient [6:36-42].
- What is encoded: `B2'` and the difference `B1' − B2'` (and likewise `A2'`, `A1' − A2'`), "nominally
  logarithmic" [5:40-54; 6:24-33; claims 4, 5]. The decoding is "either a ROM or a shifting technique"
  [5:55-57]; the patent gives the shifter hardware but no formula for the code-to-value mapping.
- Rate: x is updated each sample period [6:43-44]. The host gives a destination and an approach time; the
  increment is computed in hardware within one sample period [6:61-68; 8:8-35].
- Not stated: whether Ca/Cb are per section, whether all four coefficients share one x, and how often the
  coefficient math unit runs relative to the filter's eight states.

---

## 6. Where `~/p/emu/docs/us5170369-reverse-engineering-notes.md` is wrong or overstated

Checked against the patent only; statements the notes attribute to Rossum's papers or correspondence are not
judged here.

1. **Increment code (§7.3) is wrong, and Table I is not illegible.** The notes call Table I "mangled" and
   guess `m = d << F; S ? −m : m + 1`. The scan is clear: it is a hidden-bit format, F = 0 → d, F ≥ 1 →
   (16 + d) << (F − 1), negatives as the sign-extended two's-complement field (d − 32) << (F − 1). Table II
   confirms it (code 0x7f = 1984, not 1920; code 0xf0 = −2048, not 0). The notes' encoder is wrong to match.
2. **Overflow in the interpolator (§7.2) is wrong.** The notes force 0xFFFF / 0x0000 on overflow; the patent
   selects the clamp value on overflow [8:1-5].
3. **Multiplier widths (§6) are not in the patent.** "16-bit code", "exponent 4b", "mantissa 13b",
   "truncate 16b", "27 bits", "accumulator is plain fixed point throughout" are unsourced. The patent gives
   only 16-bit linear coefficients as adequate, as few as 8-bit compressed codes, and a 4-bit shift field in
   the decoder. (`HCHIP_ARITHMETIC.md` §3 repeats these widths tagged [P]; the patent does not state them.)
4. **Claim 6 is misdescribed (§12).** It claims "interpolating between said first and second coefficients",
   not interpolation between coefficient sets.
5. **Claim 2 is overstated (§12, §13).** It adds the coefficient-times-difference terms; it does not say the
   direct input is removed. The removal is in FIG. 5 and the reservation table (IDC gating) only.
6. **"One mux selects between 3.1 and 3.2" (§3.2)** simplifies the patent. The table gates mux 16 with DC
   in cycles 1, 2 and 4 and also sets the cycle-4 carry-in to IDC. The patent never says what drives DC or
   whether it is per section.
7. **Approach-time labels as milliseconds at 48 kHz (§7.2)** are an inference. The patent gives no unit and no
   sample rate for `@.6 … @80`; only "lowest being 32 sample periods", each a power of two greater.
8. **Figure numbering.** The notes cite "FIGs. 10–13" for the interpolator. There is no FIG. 13 drawing; the
   text's figure numbers from the multiplier on are one higher than the drawings (text FIG. 8 to 13 = drawn
   FIG. 7 to 12A/B).
9. **`A1' = A1 + 2`, `A2' = 1 − A2` "identically for the numerator" (§2)** is right in effect but is not
   stated in the patent; it follows from FIG. 4 and the `MAout` equation. It should be labelled as derived.
10. **The numerator's DC gain.** The notes state the DC-forced form makes `H(1) = 1` "exactly"; that is the
    pole part only. Per the reservation table the section output carries a further `A1' − A2'` at DC, and
    the patent provides no normalisation for it.
11. **"ARMADILLO = H-chip" (§1 table)** is not in the patent; the patent's only use of the name is the Table
    II heading "ARMAdillo Interpolator Increment Decoding".

Also: `HCHIP_PATENT_REVIEW.md` calls the shipped codec "the patent's octave-shift log"; the patent never uses
"octave shift" and gives no code-to-value formula, only "nominally logarithmic" coding decoded by "a ROM or a
shifting technique" [5:46; 5:55-57].
