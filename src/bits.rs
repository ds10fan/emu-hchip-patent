//! Fixed-width two's complement helpers. Everything in the datapath is kept
//! in an `i64` and brought back to its hardware width with these.

/// Keep the low `bits` bits of `v`, sign-extended (a plain adder/register
/// that overflows silently).
pub fn wrap(v: i64, bits: u32) -> i64 {
    debug_assert!((1..=63).contains(&bits));
    let s = 64 - bits;
    (v << s) >> s
}

/// Largest value representable in `bits` signed bits.
pub fn max_val(bits: u32) -> i64 {
    (1i64 << (bits - 1)) - 1
}

/// Smallest value representable in `bits` signed bits.
pub fn min_val(bits: u32) -> i64 {
    -(1i64 << (bits - 1))
}

/// Clamp `v` to `bits` signed bits.
pub fn sat(v: i64, bits: u32) -> i64 {
    v.clamp(min_val(bits), max_val(bits))
}

/// One's complement of `v` as a `bits`-wide word (the "bubble" outputs of
/// latches 13 and 20 and of accumulator 10 in FIG. 6).
pub fn not(v: i64, bits: u32) -> i64 {
    wrap(!v, bits)
}

/// True when `v` does not fit in `bits` signed bits.
pub fn overflows(v: i64, bits: u32) -> bool {
    v < min_val(bits) || v > max_val(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_and_sat() {
        assert_eq!(wrap(0x7fff, 16), 32767);
        assert_eq!(wrap(0x8000, 16), -32768);
        assert_eq!(wrap(0x1_0000, 16), 0);
        assert_eq!(sat(40000, 16), 32767);
        assert_eq!(sat(-40000, 16), -32768);
        assert_eq!(not(0, 16), -1);
        assert_eq!(not(5, 16) + 1, -5);
    }
}
