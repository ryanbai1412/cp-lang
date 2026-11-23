//! Differential fuzzing of the optimizer on generated effectful programs.
//!
//! Each seed builds one program from statements that sit where perf's
//! rewrites act: map views read twice, `.rev` chains, tuples of lambdas,
//! rows lent or cloned before a mutation, sort keys, searches, short
//! circuits, comprehensions and a division that traps on one input. Every
//! call goes through fns that count calls in a global or print, so a
//! rewrite that drops, repeats or reorders a call changes the output.
//! `cpl run` and the C++ of `cpl build`, compiled with
//! `-fsanitize=address,undefined`, must print the same bytes and stop with
//! the same status, and the sanitizers must stay silent. This is
//! performance/R9 in handbook/guidelines/performance-checklist.md, checked
//! on programs nobody wrote, as Csmith checks GCC.

use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/sanitized.rs"]
mod sanitized;
use sanitized::{run, sanitized_binary, Rng};

/// Programs per run; each is compiled once and kept in the cache.
const SEEDS: u64 = 32;

fn dir() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("effect_fuzz")
}

/// One statement of the body, as source lines at one indent, for a call
/// to `f` (counts), `g` (prints) or either.
fn statement(rng: &mut Rng, i: usize) -> Vec<String> {
    let one = |rng: &mut Rng| rng.pick(&["f", "g"]);
    let (a, b) = (one(rng), one(rng));
    let k = rng.range(-3, 12);
    let r = rng.range(0, 1);
    match rng.below(14) {
        0 => vec![
            format!("let m{i} = xs.map({a})"),
            format!("print(m{i})"),
            format!("print(m{i}.len, m{i}.sum(), calls)"),
        ],
        1 => vec![format!("print(xs.rev.map({a}).to_list())")],
        2 => vec![
            format!("let (p{i}, q{i}) = ((i: Int) => {a}(i + {k}), (i: Int) => {b}(i))"),
            format!("print(p{i}({r}), q{i}({k}), p{i}(2))"),
        ],
        3 => vec![
            format!("let row{i} = rows[{r}].clone()"),
            format!("rows[{r}].push({a}({k}))"),
            format!("print(row{i}, rows[{r}].len)"),
        ],
        4 => vec![
            format!("let row{i} = rows[{r}]"),
            format!("print(row{i}.len + (row{i}.find(x => {a}(x) > {k}) ?? 0))"),
        ],
        5 => vec![format!("xs.sort(by: x => {a}(x) % 5)"), "print(xs)".into()],
        6 => vec![format!("print(xs.find(x => {a}(x) > {k}) ?? -1)")],
        7 => vec![format!("print([{a}(x) for x in xs if x % 2 == {r}])")],
        8 => vec![format!(
            "print(yes({a}({k}) > 4 and {b}({r}) > 0), yes({b}({k}) > 9 or {a}(1) > 3))"
        )],
        9 => vec![
            format!("let ys{i} = xs.clone()"),
            format!("xs.push({a}({k}))"),
            format!("print(ys{i}.len, xs.len, ys{i})"),
        ],
        10 => vec![format!("xs = [{a}(x) % 50 for x in xs]")],
        11 => vec!["xs.push(h(n))".into(), "print(xs.len, calls)".into()],
        12 => vec![format!(
            "print((0..xs.len).rev.find(j => {a}(xs[j]) % 3 == 0) ?? -1)"
        )],
        _ => vec![format!("print(xs.map({a}).sum(), calls)")],
    }
}

/// The program for `seed` and the inputs it is run on: `trap` makes `h`
/// divide by zero.
fn generate(seed: u64) -> (String, Vec<String>) {
    let mut rng = Rng::new(seed);
    let (mul, add, trap) = (rng.range(1, 3), rng.range(-2, 4), rng.range(0, 9));
    let len = rng.range(0, 6);
    let mut src = format!(
        "var calls = 0\n\n\
         fn f(x: Int) -> Int:\n    calls += 1\n    x * {mul} + {add}\n\n\
         fn g(x: Int) -> Int:\n    print(\"g\", x)\n    x - 2\n\n\
         fn h(x: Int) -> Int:\n    calls += 1\n    100 / (x - {trap})\n\n\
         fn body(n: Int) -> Int:\n    \
         var xs = [i * 2 + n for i in 0..{len}]\n    \
         var rows: List<List<Int>> = [[n, 2], [3, n, 5]]\n"
    );
    for i in 0..rng.range(6, 14) as usize {
        for line in statement(&mut rng, i) {
            src += &format!("    {line}\n");
        }
    }
    src += "    print(xs, rows)\n    calls\n\nprint(body(read()))\nprint(calls)\n";
    let inputs = [trap, trap + 1, rng.range(-5, 20)]
        .iter()
        .map(|n| format!("{n}\n"))
        .collect();
    (src, inputs)
}

/// Runs the program for `seed` both ways; one line per disagreement.
fn check(seed: u64) -> Vec<String> {
    let (src, inputs) = generate(seed);
    let path = dir().join(format!("seed{seed}.cp"));
    std::fs::write(&path, &src).unwrap();
    let cpl = env!("CARGO_BIN_EXE_cpl");
    let built = Command::new(cpl)
        .arg("build")
        .arg(&path)
        .args(["--checks", "traps", "-o", "-"])
        .output()
        .expect("cpl build runs");
    if !built.status.success() {
        return vec![format!(
            "seed {seed}: the generated program doesn't build:\n{}\n{src}",
            String::from_utf8_lossy(&built.stderr)
        )];
    }
    let binary = match sanitized_binary(&dir().join("cache"), &built.stdout) {
        Ok(binary) => binary,
        Err(message) => return vec![format!("seed {seed}: {message}")],
    };
    let mut failures = Vec::new();
    for input in inputs {
        let mut interp = Command::new(cpl);
        interp.arg("run").arg(&path);
        let (by_interp, by_cpp) = (run(interp, &input), run(Command::new(&binary), &input));
        let said = String::from_utf8_lossy(&by_cpp.stderr);
        let sanitizer = said.contains("Sanitizer") || said.contains("runtime error:");
        let status_ok = matches!(by_interp.status.code(), Some(0 | 3));
        if by_interp.stdout != by_cpp.stdout
            || by_interp.status.code() != by_cpp.status.code()
            || sanitizer
            || !status_ok
        {
            failures.push(format!(
                "seed {seed} ({}) on input {input:?}:\n  cpl run: status {:?}, printed {:?}\n  C++:     status {:?}, printed {:?}\n  stderr:  {}",
                path.display(),
                by_interp.status.code(),
                String::from_utf8_lossy(&by_interp.stdout),
                by_cpp.status.code(),
                String::from_utf8_lossy(&by_cpp.stdout),
                said.lines().next().unwrap_or(""),
            ));
        }
    }
    failures
}

#[test]
fn generated_effectful_programs_agree_between_interp_and_sanitized_cpp() {
    std::fs::create_dir_all(dir()).unwrap();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()) as u64;
    let failures: Vec<String> = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    (0..SEEDS)
                        .filter(|seed| seed % threads == t)
                        .flat_map(check)
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
