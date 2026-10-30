//! The interpolating variable x of FIGs. 9 to 12: an 8-bit encoded
//! increment (Table I), a per-sample update with clamp and overflow
//! handling (FIG. 10), and the automatic increment computation from a
//! destination and an approach time (FIGs. 11, 12, Table II).
//!
//! x is 16 bits. An 8-bit destination (clamp) byte stands for the 16-bit
//! value made by duplicating it (0x5a -> 0x5a5a, 7:25), so the cutoff byte
//! c is x = c * 0x0101 and cutoff 255 is 0xFFFF.

/// Table I: the 8-bit increment code (S, shift2..0, d3..d0) as a signed
/// 16-bit linear value. Positive codes: F = 0 gives d, F >= 1 gives
/// (16 + d) << (F - 1). Negative codes: F = 0 gives d - 16, F >= 1 gives
/// (d - 32) << (F - 1). Range -2048 to 1984.
pub fn decode_increment(code: u8) -> i32 {
    let s = code >> 7;
    let f = ((code >> 4) & 7) as i32;
    let d = (code & 15) as i32;
    match (s, f) {
        (0, 0) => d,
        (0, f) => (16 + d) << (f - 1),
        (_, 0) => d - 16,
        (_, f) => (d - 32) << (f - 1),
    }
}

/// Largest Table I value not above `v` (floor onto the code lattice),
/// saturating at the ends of the range.
pub fn encode_increment_value(v: i32) -> u8 {
    let v = v.clamp(-2048, 1984);
    if v >= 0 {
        if v < 16 {
            return v as u8;
        }
        // first set bit decides the shift: value = (16 + d) << (f - 1)
        let f = (32 - (v as u32).leading_zeros()) as i32 - 4; // bit length - 4, >= 1
        let d = (v >> (f - 1)) - 16;
        ((f as u8) << 4) | d as u8
    } else {
        if v >= -16 {
            return 0x80 | (v + 16) as u8;
        }
        // smallest f with v >> (f - 1) >= -32
        let mut f = 1;
        while (v >> (f - 1)) < -32 {
            f += 1;
        }
        let d = (v >> (f - 1)) + 32;
        0x80 | ((f as u8) << 4) | d as u8
    }
}

/// Approach time index k: 32 << k samples (8:25: "eight possible values,
/// each a power of two greater, with the lowest being 32 sample periods").
pub fn approach_samples(k: u8) -> u32 {
    32 << (k & 7)
}

/// FIGs. 11 and 12: the increment code for a 17-bit difference
/// (destination minus current) and an approach time index. The difference
/// is shifted right arithmetically by 5 + k (a division by the approach
/// time rounded toward minus infinity) and floored onto the Table I
/// lattice; this reproduces every row of Table II.
pub fn encode_increment(diff: i32, k: u8) -> u8 {
    debug_assert!((-65535..=65535).contains(&diff));
    encode_increment_value(diff >> (5 + (k & 7)))
}

/// The interpolating variable with its clamp and encoded increment
/// (registers 70, 71, 72 of FIG. 10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interpolator {
    /// Register 70: the current value.
    pub x: u16,
    /// Register 71: the 8-bit clamp (destination).
    pub clamp: u8,
    /// Register 72: the 8-bit encoded increment.
    pub inc: u8,
}

impl Interpolator {
    /// At rest at `cutoff` (x = cutoff duplicated, clamp = cutoff). The
    /// increment is the smallest positive one; with x already at the clamp
    /// it is clamped away every sample.
    pub fn at(cutoff: u8) -> Self {
        Interpolator {
            x: dup(cutoff),
            clamp: cutoff,
            inc: 0,
        }
    }

    /// Jump to `cutoff` immediately.
    pub fn jump(&mut self, cutoff: u8) {
        self.x = dup(cutoff);
        self.clamp = cutoff;
        self.inc = 0;
    }

    /// FIG. 11: aim at `dest`, to be reached in about 32 << k samples. The
    /// increment is computed from the current value in one step.
    pub fn set_target(&mut self, dest: u8, k: u8) {
        let diff = dup(dest) as i32 - self.x as i32;
        self.inc = encode_increment(diff, k);
        self.clamp = dest;
    }

    /// Linear increment, with the forced carry of positive increments
    /// (Table I note): the step is value + 1 for positive codes, so a zero
    /// increment cannot be reached.
    pub fn step(&self) -> i32 {
        let v = decode_increment(self.inc);
        if self.inc & 0x80 == 0 {
            v + 1
        } else {
            v
        }
    }

    /// FIG. 10, both phases: add the increment, detect overflow, compare
    /// with the clamp in the direction of the increment, select.
    pub fn update(&mut self) -> u16 {
        let positive = self.inc & 0x80 == 0;
        let sum = self.x as i32 + self.step(); // phase 1
        let overflow = !(0..=0xFFFF).contains(&sum);
        let clamp = dup(self.clamp) as i32;
        let exceeded = if positive { sum > clamp } else { sum < clamp }; // phase 2
        self.x = if overflow || exceeded {
            clamp as u16
        } else {
            sum as u16
        };
        self.x
    }

    /// True once x sits on the clamp.
    pub fn settled(&self) -> bool {
        self.x == dup(self.clamp)
    }
}

/// An 8-bit value duplicated into 16 bits.
pub fn dup(v: u8) -> u16 {
    (v as u16) * 0x0101
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_i_examples() {
        // from the transcription's check of Table I against Table II
        assert_eq!(decode_increment(0x7f), 1984);
        assert_eq!(decode_increment(0xe0), -1024);
        assert_eq!(decode_increment(0x88), -8);
        assert_eq!(decode_increment(0x8f), -1);
        assert_eq!(decode_increment(0xf0), -2048);
        assert_eq!(decode_increment(0x00), 0);
        assert_eq!(decode_increment(0x0f), 15);
        assert_eq!(decode_increment(0x10), 16);
        assert_eq!(decode_increment(0x1f), 31);
        assert_eq!(decode_increment(0x20), 32);
    }

    #[test]
    fn lattice_is_distinct_and_encode_floors() {
        // Values are distinct (negative codes are not in code order: 0x8f is
        // -1 and 0x90 is -32), each value encodes back to its code, and a
        // value just above a lattice point still encodes to that point.
        let mut codes: Vec<u8> = (0..=255).collect();
        codes.sort_by_key(|&c| decode_increment(c));
        for w in codes.windows(2) {
            assert!(decode_increment(w[0]) < decode_increment(w[1]));
        }
        for code in 0..=255u8 {
            let v = decode_increment(code);
            assert_eq!(encode_increment_value(v), code, "code {code:02x}");
            if v < 1984 {
                assert!(decode_increment(encode_increment_value(v + 1)) <= v + 1);
            }
        }
        assert_eq!(encode_increment_value(2047), 0x7f);
        assert_eq!(encode_increment_value(-1985), 0xf0);
        assert_eq!(encode_increment_value(-9999), 0xf0);
    }

    #[test]
    fn update_reaches_the_clamp_and_stops() {
        let mut i = Interpolator::at(0);
        i.set_target(255, 0); // 65535 in 32 samples: 1984 + 1 per step
        let mut n = 0;
        while !i.settled() {
            i.update();
            n += 1;
            assert!(n < 100);
        }
        assert_eq!(i.x, 0xffff);
        assert!((32..=34).contains(&n), "{n} samples");
        // stays there
        i.update();
        assert_eq!(i.x, 0xffff);
        // and back down in 32 << 3 samples
        i.set_target(0, 3);
        assert_eq!(decode_increment(i.inc), -256);
        let mut n = 0;
        while !i.settled() {
            i.update();
            n += 1;
            assert!(n < 1000);
        }
        assert_eq!(i.x, 0);
        assert_eq!(n, 256);
    }

    #[test]
    fn small_differences_use_the_forced_carry() {
        // 0x1010 -> 0x1111 over 4096 samples: diff 257 >> 12 = 0 -> code 00,
        // step +1 per sample, reaches the clamp exactly
        let mut i = Interpolator::at(0x10);
        i.set_target(0x11, 7);
        assert_eq!(i.inc, 0x00);
        let mut n = 0;
        while !i.settled() {
            i.update();
            n += 1;
        }
        assert_eq!(n, 257);
        assert_eq!(i.x, 0x1111);
    }

    #[test]
    fn overflow_selects_the_clamp() {
        let mut i = Interpolator {
            x: 0xfff0,
            clamp: 0xff,
            inc: 0x7f, // +1985
        };
        i.update();
        assert_eq!(i.x, 0xffff);
    }
}
