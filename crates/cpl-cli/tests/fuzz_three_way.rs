//! Three-way differential fuzzing on degenerate inputs.
//!
//! Each family is one small program run on seeded inputs chosen from the
//! edges: empty domains, empty and reversed ranges, `none`, the ends of
//! `Int`, `I128` and `U64`, `0` and `P - 1` in `Mod<P>`. Every input is run
//! three ways, and all three must agree:
//!
//! - `cpl run`, the interpreter;
//! - the C++ of `cpl build`, compiled with `-fsanitize=address,undefined`;
//! - a plain reference written here in Rust, which says what the program
//!   must print, or that it must stop with a runtime error (status 3).
//!
//! A sanitizer report is a failure even when the output matches. A
//! sanitized binary is kept under a hash of its C++, the flags and the
//! compiler, as in `differential.rs`, so a warm run compiles nothing.
//!
//! An input that fails on HEAD is its own test, marked
//! `#[ignore = "KNOWN WA-TRAP: ..."]` and written up in
//! `research/fuzz-three-way-findings.md`; run those with
//! `cargo test -p cpl-cli --test fuzz_three_way -- --ignored`.

use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// Seeds per family in every run.
const SEEDS: u64 = 6;

const GXX_FLAGS: [&str; 5] = [
    "-std=c++17",
    "-fsanitize=address,undefined",
    "-fno-sanitize-recover=all",
    "-w",
    "-o",
];

/// What a program must do on one input.
#[derive(Clone, Debug, PartialEq)]
enum Expect {
    Prints(String),
    Traps,
}

/// xorshift64*: the same cases on every machine.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + self.below((hi - lo + 1) as u64) as i64
    }
    fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[self.below(xs.len() as u64) as usize]
    }
}

fn dir() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("fuzz_three_way")
}

/// Writes `src` as `name.cp`, leaving the file alone when it is unchanged.
fn program(name: &str, src: &str) -> PathBuf {
    std::fs::create_dir_all(dir()).unwrap();
    let path = dir().join(format!("{name}.cp"));
    if std::fs::read_to_string(&path).ok().as_deref() != Some(src) {
        std::fs::write(&path, src).unwrap();
    }
    path
}

fn run(mut command: Command, input: &str) -> std::process::Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("starts");
    // A program that stops early closes its input; that is not an error.
    let _ = child
        .stdin
        .take()
        .expect("piped")
        .write_all(input.as_bytes());
    child.wait_with_output().expect("exits")
}

fn gxx_version() -> &'static [u8] {
    static VERSION: OnceLock<Vec<u8>> = OnceLock::new();
    VERSION.get_or_init(|| {
        Command::new("g++")
            .arg("--version")
            .output()
            .map(|out| out.stdout)
            .unwrap_or_default()
    })
}

/// The sanitized binary of `cpp`, compiled now or kept from an earlier run.
fn sanitized_binary(cpp: &[u8]) -> Result<PathBuf, String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (gxx_version(), GXX_FLAGS, cpp).hash(&mut hasher);
    let key = format!("{:016x}", hasher.finish());
    let cache = dir().join("cache");
    std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let (source, binary) = (cache.join(format!("{key}.cpp")), cache.join(&key));
    if binary.exists() && std::fs::read(&source).is_ok_and(|kept| kept == cpp) {
        return Ok(binary);
    }
    let unique = format!(
        "{key}.{}.{:?}",
        std::process::id(),
        std::thread::current().id()
    )
    .replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "");
    let (tmp_source, tmp_binary) = (cache.join(format!("{unique}.cpp")), cache.join(&unique));
    std::fs::write(&tmp_source, cpp).map_err(|e| e.to_string())?;
    let compiled = Command::new("g++")
        .args(GXX_FLAGS)
        .arg(&tmp_binary)
        .arg(&tmp_source)
        .output()
        .map_err(|e| format!("g++ did not start ({e}); these tests need it"))?;
    if !compiled.status.success() {
        return Err(format!(
            "g++ failed on {}:\n{}",
            tmp_source.display(),
            String::from_utf8_lossy(&compiled.stderr)
        ));
    }
    std::fs::rename(&tmp_binary, &binary).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_source, &source).map_err(|e| e.to_string())?;
    Ok(binary)
}

/// Runs `name` on every input three ways; returns one line per disagreement.
fn three_way(name: &str, src: &str, cases: &[(String, Expect)]) -> Vec<String> {
    let path = program(name, src);
    let built = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args([
            OsStr("build"),
            path.as_os_str(),
            OsStr("--checks"),
            OsStr("traps"),
            OsStr("-o"),
            OsStr("-"),
        ])
        .output()
        .expect("cpl build runs");
    if !built.status.success() {
        return vec![format!(
            "{name}: cpl build failed:\n{}",
            String::from_utf8_lossy(&built.stderr)
        )];
    }
    let binary = match sanitized_binary(&built.stdout) {
        Ok(binary) => binary,
        Err(message) => return vec![format!("{name}: {message}")],
    };
    let mut failures = Vec::new();
    for (input, expect) in cases {
        let mut interp = Command::new(env!("CARGO_BIN_EXE_cpl"));
        interp.arg("run").arg(&path);
        let by_interp = run(interp, input);
        let by_cpp = run(Command::new(&binary), input);
        let said = String::from_utf8_lossy(&by_cpp.stderr);
        let sanitizer = said.contains("Sanitizer") || said.contains("runtime error:");
        let show = |out: &std::process::Output| {
            format!(
                "status {:?}, printed {:?}",
                out.status.code(),
                String::from_utf8_lossy(&out.stdout)
            )
        };
        let agrees = |out: &std::process::Output| match expect {
            Expect::Prints(text) => out.status.code() == Some(0) && out.stdout == text.as_bytes(),
            Expect::Traps => out.status.code() == Some(3),
        };
        if !agrees(&by_interp) || !agrees(&by_cpp) || sanitizer {
            failures.push(format!(
                "{name} on input {input:?}: expected {expect:?}\n  cpl run: {}\n  C++:     {}{}",
                show(&by_interp),
                show(&by_cpp),
                if sanitizer {
                    format!("\n  sanitizer: {}", said.lines().next().unwrap_or(""))
                } else {
                    String::new()
                }
            ));
        }
    }
    failures
}

#[allow(non_snake_case)]
fn OsStr(text: &str) -> &std::ffi::OsStr {
    text.as_ref()
}

fn assert_agree(failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{} disagreement(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ---- Euclidean `/` and `%` on `Int`, `I128` and `U64` ----

const INT_DIVISION: &str = "\
let q: Int = read()
for _ in 0..q:
    let (a, b): (Int, Int) = read()
    print(a / b, a % b)
";

const INT_EDGES: [i64; 13] = [
    0,
    1,
    -1,
    2,
    -2,
    3,
    -3,
    7,
    -7,
    1_000_000_000,
    -1_000_000_000,
    i64::MAX,
    -i64::MAX,
];

/// Divisors: every edge but 0.
const INT_DIVISORS: [i64; 12] = [
    1,
    -1,
    2,
    -2,
    3,
    -3,
    7,
    -7,
    1_000_000_000,
    -1_000_000_000,
    i64::MAX,
    -i64::MAX,
];

#[test]
fn euclidean_division_on_int_edges() {
    let cases: Vec<_> = (0..SEEDS)
        .map(|seed| {
            let mut rng = Rng::new(seed);
            let pairs: Vec<(i64, i64)> = (0..12)
                .map(|_| (rng.pick(&INT_EDGES), rng.pick(&INT_DIVISORS)))
                .collect();
            let input = pairs
                .iter()
                .fold(format!("{}\n", pairs.len()), |s, (a, b)| {
                    s + &format!("{a} {b}\n")
                });
            let out = pairs
                .iter()
                .map(|(a, b)| format!("{} {}\n", a.div_euclid(*b), a.rem_euclid(*b)))
                .collect();
            (input, Expect::Prints(out))
        })
        .collect();
    assert_agree(three_way("int_division", INT_DIVISION, &cases));
}

#[test]
fn int_min_over_minus_one_and_mod_int_min_are_defined() {
    let pairs = [
        (-1, -i64::MAX),
        (i64::MAX, -i64::MAX),
        (-i64::MAX, -i64::MAX),
        (0, -i64::MAX),
        (-i64::MAX, -1),
    ];
    let input = pairs
        .iter()
        .fold(format!("{}\n", pairs.len()), |s, (a, b)| {
            s + &format!("{a} {b}\n")
        });
    let out = pairs
        .iter()
        .map(|(a, b): &(i64, i64)| format!("{} {}\n", a.div_euclid(*b), a.rem_euclid(*b)))
        .collect();
    assert_agree(three_way(
        "int_division",
        INT_DIVISION,
        &[(input, Expect::Prints(out))],
    ));
}

#[test]
fn int_division_by_zero_stops() {
    let cases = vec![
        ("1\n5 0\n".to_string(), Expect::Traps),
        ("1\n0 0\n".to_string(), Expect::Traps),
    ];
    assert_agree(three_way("int_division", INT_DIVISION, &cases));
}

/// `-2^63` is not an `Int`: both reject it as input.
#[test]
fn minus_two_to_the_63_in_the_input_stops() {
    let cases = vec![
        (format!("1\n{} -1\n", i64::MIN), Expect::Traps),
        (format!("1\n5 {}\n", i64::MIN), Expect::Traps),
    ];
    assert_agree(three_way("int_division", INT_DIVISION, &cases));
}

const SHIFTS: &str = "\
let q: Int = read()
for _ in 0..q:
    let (op, a, k): (Int, Int, Int) = read()
    if op == 0: print(a << k)
    elif op == 1: print(a >> k)
    elif op == 2: print(a ** k)
    elif op == 3: print(U64(a) << k, U64(a) >> k)
    else: print(I128(a) >> k)
";

#[test]
fn shifts_and_powers_inside_their_range() {
    let rows = [
        (0, 1, 0, "1"),
        (0, 3, 61, "6917529027641081856"),
        (1, -8, 63, "-1"),
        (1, 9, 2, "2"),
        (2, 2, 0, "1"),
        (2, -3, 3, "-27"),
        (3, 1, 63, "9223372036854775808 0"),
        (4, -8, 127, "-1"),
    ];
    let input = rows
        .iter()
        .fold(format!("{}\n", rows.len()), |s, (op, a, k, _)| {
            s + &format!("{op} {a} {k}\n")
        });
    let out = rows.iter().map(|r| format!("{}\n", r.3)).collect();
    assert_agree(three_way("shifts", SHIFTS, &[(input, Expect::Prints(out))]));
}

#[test]
fn a_shift_outside_its_width_or_a_negative_power_stops() {
    let rows = [
        (0, 1, 64),
        (0, 1, -1),
        (1, 1, 64),
        (1, 1, -1),
        (2, 2, -1),
        (3, 1, 64),
        (3, 1, -1),
        (4, 1, 128),
        (4, 1, -1),
    ];
    let cases: Vec<_> = rows
        .iter()
        .map(|(op, a, k)| (format!("1\n{op} {a} {k}\n"), Expect::Traps))
        .collect();
    assert_agree(three_way("shifts", SHIFTS, &cases));
}

const LIST_EDITS: &str = "\
let n: Int = read()
var xs = [10 * i for i in 0..n]
let q: Int = read()
for _ in 0..q:
    let (op, a, b): (Int, Int, Int) = read()
    if op == 0: xs.swap(a, b)
    elif op == 1: xs.truncate(a)
    elif op == 2: xs.fill(a)
    else: print(\"ab\".repeat(a), \"|\")
    print(xs)
";

/// `LIST_EDITS` on a list of `n` after `ops`: what it prints, or `None` when an edit stops it.
fn list_edits(n: i64, ops: &[(i64, i64, i64)]) -> Option<String> {
    let mut xs: Vec<i64> = (0..n).map(|i| 10 * i).collect();
    let mut out = String::new();
    for &(op, a, b) in ops {
        match op {
            0 => {
                let inside = |i: i64| (0..xs.len() as i64).contains(&i);
                if !inside(a) || !inside(b) {
                    return None;
                }
                xs.swap(a as usize, b as usize);
            }
            1 if a < 0 => return None,
            1 => xs.truncate(a as usize),
            2 => xs.fill(a),
            _ if a < 0 => return None,
            _ => out += &format!("{} |\n", "ab".repeat(a as usize)),
        }
        let row: Vec<String> = xs.iter().map(|x| x.to_string()).collect();
        out += &format!("{}\n", row.join(" "));
    }
    Some(out)
}

#[test]
fn swaps_truncations_fills_and_repeats_at_the_edges() {
    let mut rng = Rng::new(0x5eed);
    let mut cases = Vec::new();
    for n in 0..5i64 {
        for _ in 0..12 {
            let ops: Vec<(i64, i64, i64)> = (0..3)
                .map(|_| {
                    (
                        rng.below(4) as i64,
                        rng.below(n as u64 + 3) as i64 - 1,
                        rng.below(n as u64 + 3) as i64 - 1,
                    )
                })
                .collect();
            let input = ops
                .iter()
                .fold(format!("{n}\n{}\n", ops.len()), |s, (op, a, b)| {
                    s + &format!("{op} {a} {b}\n")
                });
            cases.push((
                input,
                list_edits(n, &ops).map_or(Expect::Traps, Expect::Prints),
            ));
        }
    }
    assert!(
        cases.iter().any(|c| matches!(c.1, Expect::Traps))
            && cases.iter().any(|c| matches!(c.1, Expect::Prints(_)))
    );
    assert_agree(three_way("list_edits", LIST_EDITS, &cases));
}

const BIG_MOD: &str = "\
type Z = Mod<1000000000000000003>
let m: Int = read()
type Y = Mod<m>
let q: Int = read()
for _ in 0..q:
    let (a, b, e): (Int, Int, Int) = read()
    print(Z(a) * Z(b), Z(a) ** e, Y(a) * Y(b), Y(a) ** e)
";

#[test]
fn a_modulus_whose_square_does_not_fit() {
    let big = 1_000_000_000_000_000_003i128;
    let pow = |x: i128, e: i64, p: i128| (0..e).fold(1 % p, |acc, _| acc * x.rem_euclid(p) % p);
    for m in [
        4_000_000_007i128,
        3_037_000_500,
        3_037_000_499,
        4_611_686_018_427_387_847,
    ] {
        let rows = [
            (big as i64 - 1, big as i64 - 2, 5),
            (i64::MAX, i64::MAX, 3),
            (-1, i64::MIN + 1, 7),
            (m as i64 - 1, m as i64 - 1, 2),
            (0, 5, 0),
        ];
        let input = rows
            .iter()
            .fold(format!("{m}\n{}\n", rows.len()), |s, (a, b, e)| {
                s + &format!("{a} {b} {e}\n")
            });
        let out = rows
            .iter()
            .map(|&(a, b, e)| {
                let (a, b) = (a as i128, b as i128);
                let mul = |p: i128| a.rem_euclid(p) * b.rem_euclid(p) % p;
                format!(
                    "{} {} {} {}\n",
                    mul(big),
                    pow(a, e, big),
                    mul(m),
                    pow(a, e, m)
                )
            })
            .collect();
        assert_agree(three_way(
            "big_mod",
            BIG_MOD,
            &[(input, Expect::Prints(out))],
        ));
    }
}

const INVERSE: &str = "\
type Z = Mod<998244353>
let q: Int = read()
for _ in 0..q:
    let a: Int = read()
    print(Z(a).inv(), Z(a).inv, Z(a).inv() * Z(a), Z(a) ** -1)
";

#[test]
fn the_inverse_reads_with_and_without_parentheses() {
    let p = 998_244_353i64;
    let rows = [1, 2, -1, p - 1, p + 1, i64::MAX, i64::MIN + 1, 123_456_789];
    let input = rows
        .iter()
        .fold(format!("{}\n", rows.len()), |s, a| s + &format!("{a}\n"));
    let out = rows
        .iter()
        .map(|a| {
            let inv = pow_mod(a.rem_euclid(p), p - 2, p);
            format!("{inv} {inv} 1 {inv}\n")
        })
        .collect();
    assert_agree(three_way(
        "inverse",
        INVERSE,
        &[(input, Expect::Prints(out))],
    ));
    let none = vec![
        ("1\n0\n".to_string(), Expect::Traps),
        (format!("1\n{p}\n"), Expect::Traps),
    ];
    assert_agree(three_way("inverse", INVERSE, &none));
}

const WIDE: &str = "\
let q: Int = read()
for _ in 0..q:
    let (a, b): (I128, I128) = read()
    let (u, v): (U64, U64) = read()
    print(a / b, a % b, u / v, u % v, u + v, u - v)
";

const I128_EDGES: [i128; 12] = [
    0,
    1,
    -1,
    2,
    -7,
    7,
    i128::MAX,
    i128::MIN,
    i64::MAX as i128,
    i64::MIN as i128,
    10i128.pow(30),
    -(10i128.pow(30)),
];
const U64_EDGES: [u64; 6] = [0, 1, 2, 7, u64::MAX, u64::MAX - 1];

#[test]
fn euclidean_division_on_i128_and_wrapping_u64() {
    let cases: Vec<_> = (0..SEEDS)
        .map(|seed| {
            let mut rng = Rng::new(seed);
            let mut rows = Vec::new();
            while rows.len() < 10 {
                let (a, b) = (rng.pick(&I128_EDGES), rng.pick(&I128_EDGES));
                let (u, v) = (rng.pick(&U64_EDGES), rng.pick(&U64_EDGES));
                if b != 0 && v != 0 && !(a == i128::MIN && b == -1) {
                    rows.push((a, b, u, v));
                }
            }
            let input = rows
                .iter()
                .fold(format!("{}\n", rows.len()), |s, (a, b, u, v)| {
                    s + &format!("{a} {b}\n{u} {v}\n")
                });
            let out = rows
                .iter()
                .map(|(a, b, u, v)| {
                    format!(
                        "{} {} {} {} {} {}\n",
                        a.div_euclid(*b),
                        a.rem_euclid(*b),
                        u / v,
                        u % v,
                        u.wrapping_add(*v),
                        u.wrapping_sub(*v)
                    )
                })
                .collect();
            (input, Expect::Prints(out))
        })
        .collect();
    assert_agree(three_way("wide", WIDE, &cases));
}

#[test]
fn i128_min_over_minus_one_stops() {
    let cases = vec![(format!("1\n{} -1\n1 1\n", i128::MIN), Expect::Traps)];
    assert_agree(three_way("wide", WIDE, &cases));
}

// ---- `Mod<P>` at 0, P - 1, negative literals and `**` ----

const MODULAR: &str = "\
type Z = Mod<998244353>
type S = Mod<7>
let q: Int = read()
for _ in 0..q:
    let (x, e): (Int, Int) = read()
    let z = Z.new(x)
    let s = S.new(x)
    print(z, z * z, z - Z(1), z + Z(-1), z ** e, s, s * s - s, s ** e)
";

fn pow_mod(x: i64, e: i64, p: i64) -> i64 {
    let (mut base, mut acc, mut k) = (x.rem_euclid(p) as i128, 1i128 % p as i128, e);
    let base_inv = |b: i128| pow_mod(b as i64, p - 2, p) as i128;
    if k < 0 {
        base = base_inv(base);
        k = -k;
    }
    while k > 0 {
        if k % 2 == 1 {
            acc = acc * base % p as i128;
        }
        base = base * base % p as i128;
        k /= 2;
    }
    acc as i64
}

#[test]
fn mod_p_at_its_edges() {
    const P: i64 = 998_244_353;
    let xs = [0, 1, -1, 6, 7, -7, P - 1, P, -P, P + 1, i64::MAX, -i64::MAX];
    let cases: Vec<_> = (0..SEEDS)
        .map(|seed| {
            let mut rng = Rng::new(seed);
            let rows: Vec<(i64, i64)> = (0..10)
                .map(|_| {
                    let x = rng.pick(&xs);
                    // A negative power needs an inverse in both moduli.
                    let invertible = x.rem_euclid(P) != 0 && x.rem_euclid(7) != 0;
                    let e = if invertible {
                        rng.range(-3, 40)
                    } else {
                        rng.range(0, 40)
                    };
                    (x, e)
                })
                .collect();
            let input = rows.iter().fold(format!("{}\n", rows.len()), |s, (x, e)| {
                s + &format!("{x} {e}\n")
            });
            let out = rows
                .iter()
                .map(|&(x, e)| {
                    let (z, s) = (x.rem_euclid(P), x.rem_euclid(7));
                    format!(
                        "{z} {} {} {} {} {s} {} {}\n",
                        (z as i128 * z as i128 % P as i128),
                        (z - 1).rem_euclid(P),
                        (z - 1).rem_euclid(P),
                        pow_mod(z, e, P),
                        (s * s - s).rem_euclid(7),
                        pow_mod(s, e, 7)
                    )
                })
                .collect();
            (input, Expect::Prints(out))
        })
        .collect();
    assert_agree(three_way("modular", MODULAR, &cases));
}

// ---- Empty domains ----

const EMPTY_DOMAIN: &str = "\
let n: 0..=5 = read()
domain D = 0..n
var a: [D]Int = 7
var count = 0
for x in D:
    count += 1
print(D.len, count, a.len, sum(a[x] for x in D))
";

#[test]
fn a_domain_of_every_size_down_to_empty() {
    let cases: Vec<_> = (0..=5)
        .map(|n| {
            (
                format!("{n}\n"),
                Expect::Prints(format!("{n} {n} {n} {}\n", 7 * n)),
            )
        })
        .collect();
    assert_agree(three_way("empty_domain", EMPTY_DOMAIN, &cases));
}

const DOMAIN_ENDS: &str = "\
let n: 0..=5 = read()
domain D = 0..n
print(D.first!, D.last!)
";

#[test]
fn the_ends_of_a_domain_that_has_some() {
    let cases: Vec<_> = (1..=5)
        .map(|n| (format!("{n}\n"), Expect::Prints(format!("0 {}\n", n - 1))))
        .collect();
    assert_agree(three_way("domain_ends", DOMAIN_ENDS, &cases));
}

#[test]
fn the_ends_of_an_empty_domain_stop() {
    assert_agree(three_way(
        "domain_ends",
        DOMAIN_ENDS,
        &[("0\n".to_string(), Expect::Traps)],
    ));
}

// ---- Slices: negative, reversed, empty and past the end ----

const SLICES: &str = "\
let n: 0..=6 = read()
var a: List<Int> = []
for _ in 0..n:
    let x: Int = read()
    a.push(x)
let (lo, hi): (Int, Int) = read()
print(a[lo..hi].len)
print(a[lo..hi])
";

#[test]
fn slices_in_and_out_of_range() {
    let mut cases = Vec::new();
    for seed in 0..SEEDS * 3 {
        let mut rng = Rng::new(seed);
        let n = rng.range(0, 6);
        let a: Vec<i64> = (0..n).map(|_| rng.range(-9, 9)).collect();
        let (lo, hi) = (rng.range(-2, n + 2), rng.range(-2, n + 2));
        let input = format!(
            "{n}\n{}\n{lo} {hi}\n",
            a.iter().map(i64::to_string).collect::<Vec<_>>().join(" ")
        );
        let expect = if 0 <= lo && lo <= hi && hi <= n {
            let part = &a[lo as usize..hi as usize];
            Expect::Prints(format!(
                "{}\n{}\n",
                part.len(),
                part.iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ))
        } else {
            Expect::Traps
        };
        cases.push((input, expect));
    }
    assert_agree(three_way("slices", SLICES, &cases));
}

// ---- Optionals: `!` on `None`, `??`, `max` and `min` over all-`none` ----

const OPTIONALS: &str = "\
let n: 0..=4 = read()
var xs: List<?Int> = []
for _ in 0..n:
    let r: Int = read()
    if r == 0:
        xs.push(none)
    else:
        xs.push(r)
print(xs.max() ?? -100, xs.min() ?? 100)
let k: Int = read()
print(xs[k]!)
";

#[test]
fn optionals_with_and_without_values() {
    let mut cases = Vec::new();
    for seed in 0..SEEDS * 2 {
        let mut rng = Rng::new(seed);
        let n = rng.range(0, 4);
        let raw: Vec<i64> = (0..n)
            .map(|_| {
                if rng.below(2) == 0 {
                    0
                } else {
                    rng.range(-9, 9)
                }
            })
            .collect();
        let k = rng.range(0, (n - 1).max(0));
        let values: Vec<i64> = raw.iter().copied().filter(|&r| r != 0).collect();
        let input = format!(
            "{n}\n{}\n{k}\n",
            raw.iter().map(i64::to_string).collect::<Vec<_>>().join(" ")
        );
        let expect = match raw.get(k as usize) {
            Some(&r) if r != 0 => Expect::Prints(format!(
                "{} {}\n{r}\n",
                values.iter().max().copied().unwrap_or(-100),
                values.iter().min().copied().unwrap_or(100)
            )),
            _ => Expect::Traps,
        };
        cases.push((input, expect));
    }
    assert_agree(three_way("optionals", OPTIONALS, &cases));
}

// ---- Fenwick, SegTree, LazySegTree and Dsu on n = 0, 1, 2 and 5 ----

const STRUCTURES: &str = "\
use std.segtree.{SegTree}
use std.fenwick.{Fenwick}
use std.dsu.{Dsu}
let n: 0..=5 = read()
let q: Int = read()
domain V = 0..n
var f = Fenwick<V, Int>()
var s = SegTree<V, Int>.of(min, _ => Int.MAX)
var d = Dsu<V>()
print(d.components)
for _ in 0..q:
    let (t, i, j, x): (Int, Int, Int, Int) = read()
    if t == 0:
        f.add(V.nth(i), x)
        s.set(V.nth(i), x)
    elif t == 1:
        print(f.fold(V.nth(i)..=V.nth(j)), s.fold(V.nth(i)..=V.nth(j)))
    else:
        print(yes(d.unite(V.nth(i), V.nth(j))), yes(d.same(V.nth(i), V.nth(j))), d.components)
";

/// The structures' answers, from plain arrays. A range `i..=j` with
/// `j < i` is empty: it folds to the identity and an update to it does nothing.
fn structures_case(rng: &mut Rng, n: i64) -> (String, Expect) {
    let q = if n == 0 { 0 } else { 14 };
    let mut fen = vec![0i64; n as usize];
    let mut seg = vec![i64::MAX; n as usize];
    let mut label: Vec<usize> = (0..n as usize).collect();
    let mut input = format!("{n}\n{q}\n");
    let mut out = format!("{n}\n");
    let mut stops = false;
    for _ in 0..q {
        let (t, i, j, x) = (
            rng.range(0, 2),
            rng.range(0, n - 1),
            rng.range(0, n - 1),
            rng.range(-9, 9),
        );
        input += &format!("{t} {i} {j} {x}\n");
        let (iu, ju) = (i as usize, j as usize);
        let span = iu..=ju;
        // A range query written backwards stops the program. `i..=i - 1`
        // is in order and empty.
        stops |= t == 1 && j + 1 < i;
        match t {
            0 => {
                fen[iu] += x;
                seg[iu] = x;
            }
            1 => {
                let sum: i64 = fen.get(span.clone()).map_or(0, |part| part.iter().sum());
                let min = seg
                    .get(span)
                    .and_then(|part| part.iter().min().copied())
                    .unwrap_or(i64::MAX);
                out += &format!("{sum} {min}\n");
            }
            _ => {
                let (a, b) = (label[iu], label[ju]);
                let united = a != b;
                label.iter_mut().filter(|l| **l == b).for_each(|l| *l = a);
                let mut seen = label.clone();
                seen.sort_unstable();
                seen.dedup();
                out += &format!("{} YES {}\n", if united { "YES" } else { "NO" }, seen.len());
            }
        }
    }
    (
        input,
        if stops {
            Expect::Traps
        } else {
            Expect::Prints(out)
        },
    )
}

#[test]
fn structures_on_tiny_and_empty_domains() {
    let mut cases = Vec::new();
    for seed in 0..SEEDS {
        let mut rng = Rng::new(seed);
        for n in [0, 1, 2, 5] {
            cases.push(structures_case(&mut rng, n));
        }
    }
    assert_agree(three_way("structures", STRUCTURES, &cases));
}
