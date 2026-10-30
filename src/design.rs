//! Filter design and measurement helpers (floating point, not hardware).

use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }
    pub fn expj(w: f64) -> Self {
        Complex::new(w.cos(), w.sin())
    }
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    pub fn scale(self, k: f64) -> Complex {
        Complex::new(self.re * k, self.im * k)
    }
}

impl std::ops::Add for Complex {
    type Output = Complex;
    fn add(self, o: Complex) -> Complex {
        Complex::new(self.re + o.re, self.im + o.im)
    }
}

impl std::ops::Mul for Complex {
    type Output = Complex;
    fn mul(self, o: Complex) -> Complex {
        Complex::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }
}

/// Denominator coefficients (b1, b2) of a two-pole lowpass with the given
/// cutoff and Q at sample rate `fs` (the bilinear-transform denominator of
/// the RBJ cookbook; this section has no zeros, so only the poles are used).
pub fn lowpass_poles(fc: f64, q: f64, fs: f64) -> (f64, f64) {
    let w0 = 2.0 * PI * fc / fs;
    let alpha = w0.sin() / (2.0 * q);
    let a0 = 1.0 + alpha;
    let b1 = -2.0 * w0.cos() / a0;
    let b2 = (1.0 - alpha) / a0;
    (b1, b2)
}

/// Denominator coefficients for poles at radius `r` and angle `theta`.
pub fn poles_from_rt(r: f64, theta: f64) -> (f64, f64) {
    (-2.0 * r * theta.cos(), r * r)
}

/// The patent's transformed coefficients: B1' = B1 + 2, B2' = 1 - B2.
pub fn primes(b1: f64, b2: f64) -> (f64, f64) {
    (b1 + 2.0, 1.0 - b2)
}

/// Back from the primed coefficients to (B1, B2).
pub fn unprimes(b1p: f64, b2p: f64) -> (f64, f64) {
    (b1p - 2.0, 1.0 - b2p)
}

/// Pole radius and angle for (b1, b2); the angle is NaN for real poles.
pub fn pole_rt(b1: f64, b2: f64) -> (f64, f64) {
    let r = b2.sqrt();
    (r, (-b1 / (2.0 * r)).acos())
}

/// Magnitude response of the section at angular frequency `w` for the
/// effective coefficients [B2', B1', A2', A1'] and the DC mode bit.
///
/// H(z) = g * (1 + (A1'-2) z^-1 + (1-A2') z^-2) / (1 + (B1'-2) z^-1 + (1-B2') z^-2),
/// g = B1' - B2' when dc, else 1.
pub fn ideal_response(coefs: [f64; 4], dc: bool, w: f64) -> Complex {
    let [b2p, b1p, a2p, a1p] = coefs;
    let z1 = Complex::expj(-w);
    let z2 = Complex::expj(-2.0 * w);
    let num = Complex::new(1.0, 0.0) + z1.scale(a1p - 2.0) + z2.scale(1.0 - a2p);
    let den = Complex::new(1.0, 0.0) + z1.scale(b1p - 2.0) + z2.scale(1.0 - b2p);
    let g = if dc { b1p - b2p } else { 1.0 };
    let d2 = den.re * den.re + den.im * den.im;
    let inv = Complex::new(den.re / d2, -den.im / d2);
    (num * inv).scale(g)
}

/// DFT of a real sequence at angular frequency `w`.
pub fn dft(h: &[i64], w: f64) -> Complex {
    let mut acc = Complex::new(0.0, 0.0);
    for (n, &v) in h.iter().enumerate() {
        acc = acc + Complex::expj(-w * n as f64).scale(v as f64);
    }
    acc
}

pub fn db(x: f64) -> f64 {
    20.0 * x.max(1e-30).log10()
}
