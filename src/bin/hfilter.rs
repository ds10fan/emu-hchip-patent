//! Command line front end: print the microcode, trace cycles, measure the
//! frequency response, dump impulse responses.

use hchip_filter::coef::{decode_code, decode_linear};
use hchip_filter::design::{db, dft, ideal_response, lowpass_poles, pole_rt, primes, unprimes};
use hchip_filter::mult::Rounding;
use hchip_filter::{impulse_response, microcode, sine_gain, Params, Section};
use std::f64::consts::PI;

const FS: f64 = 48000.0;

fn usage() -> ! {
    eprintln!(
        "usage: hfilter <command> [options]
  microcode                      print the control words
  coefs                          print the coefficient code table
  trace    [--dc] [--x N] [--samples N]
                                 per-cycle trace of the first samples
  response [--dc] [--fc HZ] [--q Q] [--amp A] [--sine-amp A] [--n N]
                                 impulse-response DFT and sine measurement vs. the ideal response
  width options (all commands): --acc-bits B --acc-sat --delay-bits B --mult-bits B --drop N
                                --mant-bits B --exact-sum --round
  impulse  [--dc] [--fc HZ] [--q Q] [--amp A] [--n N]
                                 dump the impulse response as CSV
  sine     [--dc] [--fc HZ] [--q Q] [--f HZ] [--sine-amp A]
                                 gain of one sine with the saturation counters"
    );
    std::process::exit(2)
}

struct Opts {
    dc: bool,
    fc: f64,
    q: f64,
    amp: i64,
    sine_amp: i64,
    n: usize,
    x: i64,
    samples: usize,
    f: f64,
    params: Params,
}

fn parse(args: &[String]) -> Opts {
    let mut o = Opts {
        dc: false,
        fc: 1000.0,
        q: std::f64::consts::FRAC_1_SQRT_2,
        amp: 0,
        sine_amp: 0,
        n: 16384,
        x: 10000,
        samples: 2,
        f: 1000.0,
        params: Params::default(),
    };
    let mut i = 0;
    let val = |i: &mut usize| -> &str {
        *i += 1;
        args.get(*i).map(String::as_str).unwrap_or_else(|| usage())
    };
    while i < args.len() {
        match args[i].as_str() {
            "--dc" => o.dc = true,
            "--fc" => o.fc = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--q" => o.q = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--amp" => o.amp = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--sine-amp" => o.sine_amp = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--mant-bits" => o.params.mant_bits = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--exact-sum" => o.params.exact_coef_sum = true,
            "--n" => o.n = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--x" => o.x = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--f" => o.f = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--samples" => o.samples = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--acc-sat" => o.params.acc_saturate = true,
            "--round" => o.params.product_round = Rounding::Nearest,
            "--mult-bits" => {
                o.params.mult_in_bits = val(&mut i).parse().unwrap_or_else(|_| usage())
            }
            "--drop" => o.params.mult_lsb_drop = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--delay-bits" => o.params.delay_bits = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--acc-bits" => o.params.acc_bits = val(&mut i).parse().unwrap_or_else(|_| usage()),
            _ => usage(),
        }
        i += 1;
    }
    if o.amp == 0 {
        o.amp = if o.dc { 32767 } else { 2048 };
    }
    o
}

fn design(o: &Opts) -> Section {
    let (b1, b2) = lowpass_poles(o.fc, o.q, FS);
    let (b1p, b2p) = primes(b1, b2);
    let mut s = Section::new(o.params);
    s.dc = o.dc;
    s.set_coefs(b2p, b1p, 1.0, 2.0);
    s
}

fn print_design(o: &Opts, s: &Section) {
    let (b1, b2) = lowpass_poles(o.fc, o.q, FS);
    let (b1p, b2p) = primes(b1, b2);
    let eff = s.effective_coefs();
    let effv = s.effective_coefs_f64();
    let (eb1, eb2) = unprimes(effv[1], effv[0]);
    let (r, th) = pole_rt(eb1, eb2);
    println!(
        "design: fc = {} Hz, Q = {}, fs = {} Hz, DC mode = {}",
        o.fc, o.q, FS, o.dc
    );
    println!("wanted:  B1' = {b1p:.6}  B2' = {b2p:.6}  (B1 = {b1:.6}, B2 = {b2:.6})");
    println!(
        "codes:   B2' = {}  B1'-B2' = {}  A2' = {}  A1'-A2' = {}",
        s.coef_mem[0], s.coef_mem[1], s.coef_mem[2], s.coef_mem[3]
    );
    println!(
        "decoded: B2' = {:.6} ({}<<{})  B1' = {:.6} ({}<<{})  A2' = {:.6}  A1' = {:.6}",
        effv[0], eff[0].mant, eff[0].exp, effv[1], eff[1].mant, eff[1].exp, effv[2], effv[3]
    );
    println!(
        "poles:   R = {r:.6}  theta = {th:.6} rad = {:.1} Hz;  DC gain = {:.3} ({:.2} dB)",
        th * FS / (2.0 * PI),
        ideal_response(effv, o.dc, 0.0).abs(),
        db(ideal_response(effv, o.dc, 0.0).abs())
    );
    println!("params:  {:?}", s.p);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else { usage() };
    let o = parse(&args[1..]);
    match cmd.as_str() {
        "microcode" => print!("{}", microcode::table()),
        "coefs" => {
            println!("code  c  mant exp   linear   value");
            for code in 0..=255u8 {
                let d = decode_code(code);
                println!(
                    "{code:4} {:3} {:4} {:3} {:8} {:.6}",
                    code as u32 + 1,
                    d.mant,
                    d.exp,
                    decode_linear(code),
                    d.to_f64()
                );
            }
        }
        "trace" => {
            let mut s = design(&o);
            print_design(&o, &s);
            for k in 0..o.samples {
                let x = if k == 0 { o.x } else { 0 };
                let (y, tr) = s.process_traced(x);
                println!("--- sample {k}: x = {x}");
                println!(
                    "{:<7} {:>9} {:>9} {:>3} {:>9} | {:>9} {:>9} {:>9} {:>9}  (z1={} z2={})",
                    "cycle",
                    "in14",
                    "in17",
                    "cin",
                    "sum",
                    "acc10",
                    "acc11",
                    "prod",
                    "latch18",
                    tr[1].z1,
                    tr[1].z2
                );
                for t in &tr {
                    println!(
                        "{:<7} {:>9} {:>9} {:>3} {:>9} | {:>9} {:>9} {:>9} {:>9}",
                        t.name, t.a14, t.a17, t.cin, t.sum, t.acc10, t.acc11, t.prod, t.latch18
                    );
                }
                println!(
                    "output = {y}, delay memory = {:?}, stats = {:?}",
                    s.dmem, s.stats
                );
            }
        }
        "response" => {
            let mut s = design(&o);
            print_design(&o, &s);
            let eff = s.effective_coefs_f64();
            let h = impulse_response(&mut s, o.amp, o.n);
            let peak = h.iter().map(|v| v.abs()).max().unwrap_or(0);
            println!(
                "impulse: amp = {}, n = {}, peak |h| = {peak}, stats = {:?}",
                o.amp, o.n, s.stats
            );
            println!(
                "{:>8} {:>10} {:>10} {:>8} {:>10} {:>8}",
                "f [Hz]", "ideal dB", "dft dB", "err", "sine dB", "err"
            );
            let freqs = [
                20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 1500.0, 2000.0, 3000.0, 5000.0, 7000.0,
                10000.0, 15000.0, 20000.0, 23000.0,
            ];
            // Keep the sine below the delay memory's range at the loudest point
            // of the ideal response (the measurement must not saturate).
            let max_gain = (0..=2400)
                .map(|k| ideal_response(eff, o.dc, PI * k as f64 / 2400.0).abs())
                .fold(1.0f64, f64::max);
            let sine_amp = if o.sine_amp > 0 {
                o.sine_amp as f64
            } else {
                (o.amp as f64).min((30000.0 / max_gain).floor())
            };
            println!("sine: amp = {sine_amp} (max ideal gain {:.2})", max_gain);
            let mut sine_stats = Default::default();
            for f in freqs {
                let w = 2.0 * PI * f / FS;
                let ideal = db(ideal_response(eff, o.dc, w).abs());
                let meas = db(dft(&h, w).abs() / o.amp as f64);
                let mut s2 = design(&o);
                let g = sine_gain(&mut s2, f, FS, sine_amp, 4096, 8192);
                sine_stats = s2.stats;
                let sine = db(g);
                println!(
                    "{f:8.0} {ideal:10.2} {meas:10.2} {:8.2} {sine:10.2} {:8.2}",
                    meas - ideal,
                    sine - ideal
                );
            }
            println!("sine stats (last frequency): {:?}", sine_stats);
        }
        "sine" => {
            let mut s = design(&o);
            print_design(&o, &s);
            let eff = s.effective_coefs_f64();
            let amp = if o.sine_amp > 0 { o.sine_amp } else { o.amp } as f64;
            let w = 2.0 * PI * o.f / FS;
            let g = sine_gain(&mut s, o.f, FS, amp, 4096, 8192);
            println!(
                "f = {} Hz, amp = {amp}: measured {:.2} dB, ideal {:.2} dB, stats = {:?}",
                o.f,
                db(g),
                db(ideal_response(eff, o.dc, w).abs()),
                s.stats
            );
        }
        "impulse" => {
            let mut s = design(&o);
            let h = impulse_response(&mut s, o.amp, o.n);
            println!("n,h");
            for (i, v) in h.iter().enumerate() {
                println!("{i},{v}");
            }
        }
        _ => usage(),
    }
}
