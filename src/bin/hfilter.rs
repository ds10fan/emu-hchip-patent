//! Command line front end: print the microcode, trace cycles, measure the
//! frequency response, dump impulse responses.

use hchip_filter::coef::{decode_code, decode_linear};
use hchip_filter::design::{db, dft, ideal_response, lowpass_poles, pole_rt, primes, unprimes};
use hchip_filter::mult::Rounding;
use hchip_filter::voice::{stock_type, word_info, FLAG_GAIN, FLAG_UNNORMALISED};
use hchip_filter::{impulse_response, microcode, sine_gain, Params, Section, Voice};
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
                                 gain of one sine with the saturation counters
  voice    --type lp2|lp6|hp4 [--vq 0|64|127] --cutoff N   or   --words W0,..,W9 --flags F --cutoff N
                                 decode the words of one voice and measure its response
           [--sweep-to N --time K]  instead: sweep the cutoff to N in 32<<K samples and print x and word 0 on the way"
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
    vtype: String,
    vq: u8,
    cutoff: u8,
    sweep_to: Option<u8>,
    time: u8,
    words: Option<[u32; 10]>,
    flags: u8,
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
        vtype: "lp2".into(),
        vq: 64,
        cutoff: 128,
        sweep_to: None,
        time: 5,
        words: None,
        flags: 0x40,
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
            "--type" => o.vtype = val(&mut i).to_string(),
            "--vq" => o.vq = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--cutoff" => o.cutoff = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--sweep-to" => o.sweep_to = Some(val(&mut i).parse().unwrap_or_else(|_| usage())),
            "--time" => o.time = val(&mut i).parse().unwrap_or_else(|_| usage()),
            "--flags" => {
                let v = val(&mut i);
                o.flags =
                    u8::from_str_radix(v.trim_start_matches("0x"), 16).unwrap_or_else(|_| usage());
            }
            "--words" => {
                let v: Vec<u32> = val(&mut i)
                    .split(',')
                    .map(|w| {
                        u32::from_str_radix(w.trim_start_matches("0x"), 16)
                            .unwrap_or_else(|_| usage())
                    })
                    .collect();
                if v.len() != 10 {
                    usage();
                }
                o.words = Some(v.try_into().unwrap());
            }
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
        ideal_response(effv, o.dc, false, 0.0).abs(),
        db(ideal_response(effv, o.dc, false, 0.0).abs())
    );
    println!("params:  {:?}", s.p);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else { usage() };
    let o = parse(&args[1..]);
    match cmd.as_str() {
        "microcode" => {
            println!("section (MICROCODE):");
            print!("{}", microcode::table());
            println!("\nsection with output gain (MICROCODE_GAIN):");
            print!("{}", microcode::table_of(&microcode::MICROCODE_GAIN));
        }
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
                .map(|k| ideal_response(eff, o.dc, false, PI * k as f64 / 2400.0).abs())
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
                let ideal = db(ideal_response(eff, o.dc, false, w).abs());
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
                db(ideal_response(eff, o.dc, false, w).abs()),
                s.stats
            );
        }
        "voice" => {
            let (words, flags) = match o.words {
                Some(w) => (w, o.flags),
                None => stock_type(&o.vtype, o.vq).unwrap_or_else(|| {
                    eprintln!("unknown stock type {} at Q {}", o.vtype, o.vq);
                    std::process::exit(2)
                }),
            };
            let mut v = Voice::new(o.params);
            v.set(words, o.cutoff, flags);
            if let Some(dest) = o.sweep_to {
                v.sweep_to(dest, o.time);
                println!(
                    "sweep: cutoff {} -> {} in 32<<{} = {} samples; increment code {:02x} = {:+} per sample",
                    o.cutoff,
                    dest,
                    o.time,
                    hchip_filter::interp::approach_samples(o.time),
                    v.interp.inc,
                    v.interp.step()
                );
                println!(
                    "{:>7} {:>6} {:>8} {:>8} {:>10} {:>9} {:>6}",
                    "sample", "x", "X0 code", "F0 code", "B2'", "f0 [Hz]", "Q"
                );
                let total = hchip_filter::interp::approach_samples(o.time) as usize + 8;
                let every = (total / 16).max(1);
                for n in 0..=total {
                    if n % every == 0 || v.interp.settled() {
                        let x = v.x();
                        let (x0, f0, x255, f255) = hchip_filter::voice::word_codes(words[0]);
                        let xc = hchip_filter::voice::interp_x(x0, x255, x);
                        let fc = hchip_filter::voice::interp_x(f0, f255, x);
                        let e = v.sections[0].effective_coefs_f64();
                        let (b1, b2) = hchip_filter::design::unprimes(e[1], e[0]);
                        let (fhz, q) = hchip_filter::design::pole_fq(b1, b2, FS);
                        println!(
                            "{n:7} {x:6x} {:8.2} {:8.2} {:10.6} {fhz:9.1} {q:6.2}",
                            xc as f64 / 256.0,
                            fc as f64 / 256.0,
                            e[0]
                        );
                        if v.interp.settled() {
                            break;
                        }
                    }
                    v.process(0);
                }
                return;
            }
            println!(
                "voice: words {:08X?}, cutoff {}, flags 0x{:02X} (gain word {}, unnormalised {})",
                words,
                o.cutoff,
                flags,
                flags & FLAG_GAIN != 0,
                flags & FLAG_UNNORMALISED != 0
            );
            println!(
                "{:>4} {:>8} {:>8} {:>10} {:>10} {:>9} {:>7}  role",
                "word", "X code", "F code", "B2'", "f^2", "f0 [Hz]", "Q"
            );
            for (i, w) in words.iter().enumerate() {
                let role = match i {
                    0 | 4 | 8 => "pole",
                    1 | 5 => "zero",
                    9 if flags & FLAG_GAIN != 0 => "gain",
                    9 => "zero",
                    _ => "unused",
                };
                if role == "unused" {
                    continue;
                }
                let info = word_info(*w, o.cutoff, FS);
                if role == "gain" {
                    let g = v.sections[2].effective_coefs_f64()[4];
                    println!(
                        "{i:>4} {:>8} {:>8.2} {:>10} {:>10} {:>9} {:>7}  gain = {g:.4} ({:.2} dB)",
                        "-",
                        info.f_code,
                        "-",
                        "-",
                        "-",
                        "-",
                        db(g)
                    );
                } else {
                    println!(
                        "{i:>4} {:>8.2} {:>8.2} {:>10.6} {:>10.6} {:>9.1} {:>7.2}  {role}",
                        info.x_code, info.f_code, info.b2p, info.f2, info.f0, info.q
                    );
                }
            }
            for (k, s) in v.sections.iter().enumerate() {
                let e = s.effective_coefs_f64();
                println!(
                    "section {k}: dc = {}, B2' = {:.6}, B1' = {:.6}, A2' = {:.6}, A1' = {:.6}, gain rows = {}, {} cycles",
                    s.dc, e[0], e[1], e[2], e[3], s.with_gain, s.program().len()
                );
            }
            let max_gain = v.max_internal_gain().max(1e-9);
            let amp = (30000.0 / max_gain).floor().min(30000.0);
            println!(
                "cycles per sample: {}; sine amp {amp} (max internal gain {max_gain:.3})",
                v.cycles_per_sample()
            );
            println!(
                "{:>8} {:>10} {:>10} {:>8}",
                "f [Hz]", "ideal dB", "sine dB", "err"
            );
            let freqs = [
                20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0,
            ];
            for f in freqs {
                let w = 2.0 * PI * f / FS;
                let ideal = db(v.ideal_response(w).abs());
                let mut v2 = v.clone();
                v2.reset_state();
                let g = {
                    let mut acc = hchip_filter::design::Complex::new(0.0, 0.0);
                    let (settle, n) = (4096usize, 8192usize);
                    for i in 0..settle + n {
                        let x = (amp * (w * i as f64).sin()).round() as i64;
                        let y = v2.process(x) as f64;
                        if i >= settle {
                            acc = acc + hchip_filter::design::Complex::expj(-w * i as f64).scale(y);
                        }
                    }
                    acc.abs() / (amp * n as f64 / 2.0)
                };
                println!("{f:8.0} {ideal:10.2} {:10.2} {:8.2}", db(g), db(g) - ideal);
            }
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
