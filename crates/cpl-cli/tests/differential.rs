//! The interpreter against the C++ back end, end to end.
//!
//! Every program under `tests/programs` and `proto/tests/program` is run by `cpl run`, and built by
//! `cpl build`, compiled with `g++ -std=c++17 -fsanitize=address,undefined`
//! and run. Both must print the bytes in the program's `.out` file and
//! exit with the same status, and the sanitizers must stay silent: C++
//! that only happens to work is a failure here.
//!
//! A program reads the `.in` file beside it, if there is one. To record
//! the `.out` of a new program, run the tests with `CPL_BLESS=1`; the
//! interpreter writes it, and the C++ is then held to it.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn programs() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    // The prototype's program-structure cases (decision 20: the
    // `@!multitest` divider and top-level `return`) are held to the same
    // `.out` files here.
    let structure = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../proto/tests/program");
    let mut found: Vec<PathBuf> = [&dir, &structure]
        .into_iter()
        .flat_map(|dir| {
            std::fs::read_dir(dir).unwrap_or_else(|_| panic!("{} exists", dir.display()))
        })
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cp"))
        .collect();
    found.sort();
    assert!(
        found.len() >= 10,
        "only {} programs in {}",
        found.len(),
        dir.display()
    );
    found
}

fn input_of(program: &Path) -> Vec<u8> {
    std::fs::read(program.with_extension("in")).unwrap_or_default()
}

/// Compiles `cpp` with the sanitizers on and runs it on `input`.
/// Runs the built solution against `cpl judge`, the `@interactor` of
/// `program`: each reads what the other prints, and `input` is the
/// hidden test. The output is what the solution printed.
fn against_judge(
    binary: &Path,
    program: &Path,
    hidden: &Path,
    input: &[u8],
) -> Result<std::process::Output, String> {
    use std::io::{BufRead, BufReader};
    std::fs::write(hidden, input).map_err(|e| e.to_string())?;
    let mut judge = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("judge")
        .arg(program)
        .arg(hidden)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut solution = Command::new(binary)
        .stdin(judge.stdout.take().expect("piped"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    // Every line the solution prints is kept, and passed on to the judge.
    let mut to_judge = judge.stdin.take().expect("piped");
    let mut said = Vec::new();
    for line in BufReader::new(solution.stdout.take().expect("piped")).split(b'\n') {
        let mut line = line.map_err(|e| e.to_string())?;
        line.push(b'\n');
        said.extend_from_slice(&line);
        // A judge that has ruled no longer listens.
        let _ = to_judge.write_all(&line).and_then(|()| to_judge.flush());
    }
    drop(to_judge);
    let mut ran = solution.wait_with_output().map_err(|e| e.to_string())?;
    let judged = judge.wait_with_output().map_err(|e| e.to_string())?;
    ran.stdout = said;
    if ran.status.success() && !judged.status.success() {
        return Err(format!(
            "the interactor rejected the C++:\n{}",
            String::from_utf8_lossy(&judged.stderr)
        ));
    }
    Ok(ran)
}

fn compile_and_run(
    name: &str,
    cpp: &[u8],
    input: &[u8],
    interactor: Option<&Path>,
) -> Result<(Vec<u8>, Option<i32>), String> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("differential");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let source = dir.join(format!("{name}.cpp"));
    let binary = dir.join(name);
    std::fs::write(&source, cpp).map_err(|e| e.to_string())?;
    let compiled = Command::new("g++")
        .args([
            "-std=c++17",
            "-fsanitize=address,undefined",
            "-fno-sanitize-recover=all",
            "-Wall",
            "-Wextra",
            "-o",
        ])
        .arg(&binary)
        .arg(&source)
        .output()
        .map_err(|e| format!("g++ did not start ({e}); these tests need it"))?;
    let warnings = String::from_utf8_lossy(&compiled.stderr);
    if !compiled.status.success() || !warnings.is_empty() {
        return Err(format!(
            "g++ complained about {}:\n{warnings}",
            source.display()
        ));
    }
    let ran = match interactor {
        None => {
            let mut child = Command::new(&binary)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())?;
            let mut stdin = child.stdin.take().expect("piped");
            stdin.write_all(input).map_err(|e| e.to_string())?;
            drop(stdin);
            child.wait_with_output().map_err(|e| e.to_string())?
        }
        Some(program) => against_judge(&binary, program, &dir.join(format!("{name}.in")), input)?,
    };
    let stderr = String::from_utf8_lossy(&ran.stderr);
    if stderr.contains("Sanitizer") || stderr.contains("runtime error:") {
        return Err(format!(
            "the sanitizers stopped {}:\n{stderr}",
            source.display()
        ));
    }
    Ok((ran.stdout, ran.status.code()))
}

fn check(program: &Path) -> Result<(), String> {
    let name = program
        .file_stem()
        .expect("a file name")
        .to_string_lossy()
        .into_owned();
    let path = program.to_string_lossy().into_owned();
    let input = input_of(program);

    let interpreted = cpl_cli::run_with_input(&["run", &path], &input);
    if interpreted.code != 0 && interpreted.code != cpl_cli::EXIT_FAILED {
        return Err(format!(
            "`cpl run` exited with {}:\n{}",
            interpreted.code, interpreted.stderr
        ));
    }
    let golden = program.with_extension("out");
    if std::env::var_os("CPL_BLESS").is_some() {
        std::fs::write(&golden, &interpreted.stdout).map_err(|e| e.to_string())?;
    }
    let expected = std::fs::read(&golden)
        .map_err(|_| format!("no {}: run with CPL_BLESS=1 to record it", golden.display()))?;
    if interpreted.stdout != expected {
        return Err(format!(
            "`cpl run` printed\n{}\nbut {} holds\n{}",
            String::from_utf8_lossy(&interpreted.stdout),
            golden.display(),
            String::from_utf8_lossy(&expected)
        ));
    }

    let built = cpl_cli::run(&["build", &path, "-o", "-"]);
    if built.code != 0 {
        return Err(format!(
            "`cpl build` exited with {}:\n{}",
            built.code, built.stderr
        ));
    }
    if built.stdout.len() >= cpl_cli::MAX_CPP_BYTES {
        return Err(format!(
            "the C++ is {} bytes, over the 64 KB a judge takes",
            built.stdout.len()
        ));
    }
    if built.stdout != cpl_cli::run(&["build", &path, "-o", "-"]).stdout {
        return Err("two builds of one source differ".into());
    }
    let source = std::fs::read_to_string(program).map_err(|e| e.to_string())?;
    let cpp = String::from_utf8_lossy(&built.stdout);
    for line in source.lines().filter(|line| !line.trim().is_empty()) {
        if !cpp.contains(&format!("//     {line}")) {
            return Err(format!(
                "the C++ does not open with the source: `{line}` is missing"
            ));
        }
    }

    // An interactive problem is run against the judge in its file.
    let interactor = source.contains("\n@interactor").then_some(program);
    let (stdout, code) = compile_and_run(&name, &built.stdout, &input, interactor)?;
    if stdout != interpreted.stdout {
        return Err(format!(
            "the C++ printed\n{}\nbut the interpreter printed\n{}",
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&interpreted.stdout)
        ));
    }
    if code != Some(i32::from(interpreted.code)) {
        return Err(format!(
            "the C++ exited with {code:?}, the interpreter with {}",
            interpreted.code
        ));
    }
    Ok(())
}

#[test]
fn interpreter_and_sanitized_cpp_agree_on_every_program() {
    let programs = programs();
    let workers = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(programs.len());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let failures = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(program) = programs.get(index) else {
                    break;
                };
                if let Err(why) = check(program) {
                    failures
                        .lock()
                        .expect("no panic while held")
                        .push(format!("{}: {why}", program.display()));
                }
            });
        }
    });
    let mut failures = failures.into_inner().expect("no panic while held");
    failures.sort();
    assert!(
        failures.is_empty(),
        "{} of {} programs failed:\n\n{}",
        failures.len(),
        programs.len(),
        failures.join("\n\n")
    );
}

#[test]
fn lowering_gaps_agree_between_interp_and_sanitized_cpp() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "prefix_counts_span",
            "\
domain I = 1..=3
let a: [I, I]Int = [I, I] (r, c) => r.ord * 3 + c.ord + 1
let pre = prefix(a)
let row = [I] i => a[I(2), i]
let p1 = prefix(row)
let cnt = counts([1, 2, 1, 3, 1])
print(pre.sum(span(I(1).l, I(2)), span(I(2).l, I(3))))
print(p1.sum(span(I(1), I(3))))
print(cnt[1], cnt[2], cnt[99])
",
            "16\n15\n3 1 0\n",
        ),
        (
            "subset_alias_and_submasks",
            "\
domain Item = 0..3
type Group = Subset<Item>
var total = 0
for s in Group:
    for g in s.subsets:
        let rest = s - g
        total += g.len + rest.len
let full = Group.full
let one = Group.empty.with(Item(1))
print(total, full.len, full.without(Item(1)).len, one.first!.ord)
",
            "54 3 2 1\n",
        ),
        (
            "filled_none_inference",
            "\
fn make_grid(n: Int) -> List<List<?Int>>:
    var out = List.filled(n, List.filled(n, none))
    out[0][1] = 7
    out

let g = make_grid(2)
var dp = List.filled(3, none)
dp[2] = 42
print(g[0][0] ?? -1, g[0][1] ?? -1, dp[0] ?? -1, dp[2] ?? -1)
",
            "-1 7 -1 42\n",
        ),
        (
            "adjacent_and_domain_list_index",
            "\
domain Bug = 0..4
let bs = List(Bug)
var total = 0
for i in Bug:
    total += bs[i.to_idx].ord
let pairs = [10, 20, 35, 35]
print(total, count(a != b for (a, b) in pairs.adjacent))
var known: [Bool]List<Int> = []
known[true].push(5)
known[false].push(9)
let flag = true
print(known[if flag: true else: false][0])
",
            "6 2\n5\n",
        ),
    ];

    let tmp = std::env::temp_dir().join(format!("cpl_lowering_gaps_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    for (name, src, expected) in cases {
        let cp = tmp.join(format!("{name}.cp"));
        std::fs::write(&cp, src).expect("write test program");
        let path = cp.to_string_lossy().into_owned();
        let interpreted = cpl_cli::run(&["run", &path]);
        assert_eq!(
            interpreted.code,
            0,
            "{name}: cpl run failed:\n{}",
            interpreted.stderr
        );
        assert_eq!(
            String::from_utf8_lossy(&interpreted.stdout),
            *expected,
            "{name}: interpreter output mismatch"
        );
        let built = cpl_cli::run(&["build", &path, "-o", "-"]);
        assert_eq!(
            built.code,
            0,
            "{name}: cpl build failed:\n{}",
            built.stderr
        );
        let (stdout, code) =
            compile_and_run(&format!("gap_{name}"), &built.stdout, &[], None).expect(name);
        assert_eq!(code, Some(0), "{name}: sanitized C++ non-zero exit");
        assert_eq!(
            String::from_utf8_lossy(&stdout),
            *expected,
            "{name}: sanitized C++ output mismatch"
        );
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
