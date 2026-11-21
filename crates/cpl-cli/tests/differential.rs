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
//!
//! The programs are split into `SHARDS` tests by their index in sorted
//! order, so the test runner spreads them over its threads. A sanitized
//! binary is kept under the C++ it was built from and reused while that
//! source, the flags and the compiler are unchanged; it is still run every
//! time.

use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

const SHARDS: usize = 8;

const GXX_FLAGS: [&str; 6] = [
    "-std=c++17",
    "-fsanitize=address,undefined",
    "-fno-sanitize-recover=all",
    "-Wall",
    "-Wextra",
    "-o",
];

/// The programs of `tests/parity`: `mut` receivers, fields and cells
/// read by closures and helpers while a call changes them, how often
/// and when `sort(by:)` works out a key, `print` arguments that print,
/// and stepped ranges at their edges. They are a test of their own,
/// so a break in either is named as that.
fn parity_programs() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/parity");
    let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("{} exists", dir.display()))
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cp"))
        .collect();
    found.sort();
    found
}

#[test]
fn aliasing_and_sort_keys_agree_between_interp_and_sanitized_cpp() {
    let programs = parity_programs();
    let names: Vec<_> = programs
        .iter()
        .map(|p| {
            p.file_stem()
                .expect("a name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        names,
        [
            "map_update_field",
            "mut_cell_receiver",
            "mut_field_after_closure",
            "mut_self_closure",
            "print_args_first",
            "sort_key_calls",
            "sort_key_one_traps",
            "sort_key_own_list",
            "step_at_int_max",
            "step_not_positive"
        ]
    );
    let failures: Vec<String> = programs
        .iter()
        .filter_map(|program| {
            check(program, "parity")
                .err()
                .map(|why| format!("{}: {why}", program.display()))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

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

/// What `g++ --version` prints, so a compiler upgrade rebuilds every binary.
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

/// The sanitized binary built from `cpp`, compiled now or reused from an
/// earlier run that compiled the same bytes with the same flags and compiler.
fn sanitized_binary(cpp: &[u8]) -> Result<PathBuf, String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (gxx_version(), GXX_FLAGS, cpp).hash(&mut hasher);
    let key = format!("{:016x}", hasher.finish());
    let cache = Path::new(env!("CARGO_TARGET_TMPDIR")).join("differential/cache");
    std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let source = cache.join(format!("{key}.cpp"));
    let binary = cache.join(&key);
    if binary.exists() && std::fs::read(&source).is_ok_and(|kept| kept == cpp) {
        return Ok(binary);
    }
    // Built under a name of its own and renamed, so a shard never runs a
    // binary another one is still writing.
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
    let warnings = String::from_utf8_lossy(&compiled.stderr);
    if !compiled.status.success() || !warnings.is_empty() {
        let _ = std::fs::remove_file(&tmp_binary);
        return Err(format!(
            "g++ complained about {}:\n{warnings}",
            tmp_source.display()
        ));
    }
    std::fs::rename(&tmp_binary, &binary).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_source, &source).map_err(|e| e.to_string())?;
    Ok(binary)
}

/// Builds `cpp` with the sanitizers on and runs it on `input`, with
/// `scratch` (a directory of the calling test's own) for the judge's files.
fn compile_and_run(
    name: &str,
    scratch: &str,
    cpp: &[u8],
    input: &[u8],
    interactor: Option<&Path>,
) -> Result<(Vec<u8>, Option<i32>), String> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("differential")
        .join(scratch);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let binary = sanitized_binary(cpp)?;
    let source = binary.with_extension("cpp");
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

fn check(program: &Path, group: &str) -> Result<(), String> {
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
    let (stdout, code) = compile_and_run(&name, group, &built.stdout, &input, interactor)?;
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

/// The programs whose index in sorted order is `shard` modulo `SHARDS`.
fn shard_of(programs: &[PathBuf], shard: usize) -> Vec<&PathBuf> {
    programs.iter().skip(shard).step_by(SHARDS).collect()
}

fn agree_on_shard(shard: usize) {
    let programs = programs();
    let mine = shard_of(&programs, shard);
    let failures: Vec<String> = mine
        .iter()
        .filter_map(|program| {
            check(program, &format!("shard-{shard}"))
                .err()
                .map(|why| format!("{}: {why}", program.display()))
        })
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} programs in shard {shard} failed:\n\n{}",
        failures.len(),
        mine.len(),
        failures.join("\n\n")
    );
}

macro_rules! shards {
    ($($test:ident = $shard:literal),* $(,)?) => {
        $(
            #[test]
            fn $test() {
                agree_on_shard($shard);
            }
        )*
        #[test]
        fn the_shards_cover_every_program_once() {
            let programs = programs();
            let mut covered: Vec<&PathBuf> =
                [$($shard),*].into_iter().flat_map(|k| shard_of(&programs, k)).collect();
            covered.sort();
            assert_eq!(covered, programs.iter().collect::<Vec<_>>());
            assert_eq!([$($shard),*].len(), SHARDS);
        }
    };
}

shards! {
    interpreter_and_sanitized_cpp_agree_shard_0 = 0,
    interpreter_and_sanitized_cpp_agree_shard_1 = 1,
    interpreter_and_sanitized_cpp_agree_shard_2 = 2,
    interpreter_and_sanitized_cpp_agree_shard_3 = 3,
    interpreter_and_sanitized_cpp_agree_shard_4 = 4,
    interpreter_and_sanitized_cpp_agree_shard_5 = 5,
    interpreter_and_sanitized_cpp_agree_shard_6 = 6,
    interpreter_and_sanitized_cpp_agree_shard_7 = 7,
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
        let (stdout, code) = compile_and_run(
            &format!("gap_{name}"),
            "lowering-gaps",
            &built.stdout,
            &[],
            None,
        )
        .expect(name);
        assert_eq!(code, Some(0), "{name}: sanitized C++ non-zero exit");
        assert_eq!(
            String::from_utf8_lossy(&stdout),
            *expected,
            "{name}: sanitized C++ output mismatch"
        );
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

/// Runs `command` against a judge that answers each `? x` with `x * 10`
/// only once it has read the line, and gives the lines the program said.
/// A program that keeps its questions to itself until it has an answer
/// never gets one: the test fails after ten seconds instead of hanging.
fn asked_live(mut command: Command) -> Vec<String> {
    use std::io::{BufRead, BufReader};
    let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("the program starts");
    let mut to_program = child.stdin.take().unwrap();
    let from_program = BufReader::new(child.stdout.take().unwrap());
    let (send, lines) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in from_program.lines().map_while(Result::ok) {
            if send.send(line).is_err() {
                break;
            }
        }
    });
    let mut said = Vec::new();
    while let Ok(line) = lines.recv_timeout(std::time::Duration::from_secs(10)) {
        if let Some(asked) = line.strip_prefix("? ") {
            // Both questions are out before the first answer goes in.
            if said.iter().filter(|l: &&String| l.starts_with("? ")).count() % 2 == 1 {
                let first: i64 = said.last().unwrap()[2..].parse().unwrap();
                let second: i64 = asked.parse().unwrap();
                writeln!(to_program, "{}\n{}", first * 10, second * 10).unwrap();
                to_program.flush().unwrap();
            }
        }
        let done = line.starts_with('!');
        said.push(line);
        if done {
            break;
        }
    }
    drop(to_program);
    let _ = child.kill();
    let _ = child.wait();
    said
}

/// `print` then `read()` with no `flush()` and no `ask`: a read that has
/// to wait shows what was printed first, in the interpreter and in the
/// C++, so a judge that answers only what it has seen is answered.
#[test]
fn a_read_that_waits_shows_what_was_printed_before_it() {
    let source = concat!(
        "var total = 0\n",
        "for round in 0..3:\n",
        "    print(\"?\", round + 1)\n",
        "    print(\"?\", round + 5)\n",
        "    let (a, b): (Int, Int) = read()\n",
        "    total += a + b\n",
        "print(\"!\", total)\n",
    );
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("differential/waits");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("waits.cp");
    std::fs::write(&path, source).unwrap();
    let path = path.to_string_lossy().into_owned();
    let expected = ["? 1", "? 5", "? 2", "? 6", "? 3", "? 7", "! 240"];

    let mut interpreted = Command::new(env!("CARGO_BIN_EXE_cpl"));
    interpreted.args(["run", &path]);
    assert_eq!(asked_live(interpreted), expected, "`cpl run`");

    let built = cpl_cli::run(&["build", &path, "-o", "-"]);
    assert_eq!(built.code, 0, "{}", built.stderr);
    // Nothing is flushed line by line: a program that reads and then
    // prints stays as fast as it was.
    let cpp = String::from_utf8_lossy(&built.stdout);
    let solve = cpp.split("void solve()").last().unwrap().split("int main").next().unwrap();
    assert!(!solve.contains("flush"), "{solve}");
    let binary = sanitized_binary(&built.stdout).unwrap();
    assert_eq!(asked_live(Command::new(binary)), expected, "the C++");
}
