//! The interpreter against the C++ back end, end to end.
//!
//! Every program under `tests/programs` and `crates/cpl-cli/tests/cases/program` is run by `cpl run`, and built by
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
/// stepped ranges at their edges, arrays of two lengths given for one
/// `D`, `argmin` on arrays that do not start at 0, `?` inside a larger
/// expression, and how a `Float` prints. They are a test of their own,
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
            "argmin_own_index",
            "dead_field",
            "dead_field_store_traps",
            "domain_step",
            "domain_step_not_positive",
            "float_digits",
            "grid_parts_generic",
            "interval_default_on_old",
            "map_get_lend",
            "map_update_field",
            "mut_cell_receiver",
            "mut_field_after_closure",
            "mut_self_closure",
            "part_below_first",
            "part_from_one",
            "part_overlap",
            "part_past_last",
            "print_args_first",
            "range_int_sum",
            "range_int_sum_bad_step",
            "range_int_sum_near_limit",
            "range_int_sum_overflow",
            "rec_fn_param",
            "rec_local_closure",
            "same_domain_lengths",
            "same_domain_longer_first",
            "same_domain_mut",
            "sort_cmp",
            "sort_key_calls",
            "sort_key_one_traps",
            "sort_key_own_list",
            "sorted_ord",
            "step_at_int_max",
            "step_not_positive",
            "str_rfind_in",
            "tested_folds",
            "try_in_index",
            "try_inside_expr",
            "try_inside_stmt",
            "unit_generic_local",
            "unread_fn_field",
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
    let structure = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/program");
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

    let built = cpl_cli::run(&["build", &path, "--checks", "traps", "-o", "-"]);
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
    if built.stdout != cpl_cli::run(&["build", &path, "--checks", "traps", "-o", "-"]).stdout {
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
let pre = Prefix<I, I, Int>(a)
let row = [I] i => a[I(2), i]
let p1 = Prefix(row)
let cnt = counts([1, 2, 1, 3, 1])
print(pre.fold(I(1)..=I(2), I(2)..=I(3)))
print(p1.fold(I(1)..=I(3)))
print(cnt[1], cnt[2], cnt[99])
",
            "16\n15\n3 1 0\n",
        ),
        (
            "integer_printing_extremes",
            "\
let n = 9223372036854775807
let u: U64 = 18446744073709551615
print(0, -1, n, -n, n - 1, -n + 1, 1234567890123456789)
print(u, U64(0), U64(10))
",
            "0 -1 9223372036854775807 -9223372036854775807 9223372036854775806 \
             -9223372036854775806 1234567890123456789\n18446744073709551615 0 10\n",
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
print(total, pairs.adjacent.count((a, b) => a != b))
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
            interpreted.code, 0,
            "{name}: cpl run failed:\n{}",
            interpreted.stderr
        );
        assert_eq!(
            String::from_utf8_lossy(&interpreted.stdout),
            *expected,
            "{name}: interpreter output mismatch"
        );
        let built = cpl_cli::run(&["build", &path, "--checks", "traps", "-o", "-"]);
        assert_eq!(built.code, 0, "{name}: cpl build failed:\n{}", built.stderr);
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
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the program starts");
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
            if said
                .iter()
                .filter(|l: &&String| l.starts_with("? "))
                .count()
                % 2
                == 1
            {
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

    let built = cpl_cli::run(&["build", &path, "--checks", "traps", "-o", "-"]);
    assert_eq!(built.code, 0, "{}", built.stderr);
    // Nothing is flushed line by line: a program that reads and then
    // prints stays as fast as it was.
    let cpp = String::from_utf8_lossy(&built.stdout);
    let solve = cpp
        .split("void solve()")
        .last()
        .unwrap()
        .split("}  // namespace prog")
        .next()
        .unwrap();
    assert!(!solve.contains("flush"), "{solve}");
    let binary = sanitized_binary(&built.stdout).unwrap();
    assert_eq!(asked_live(Command::new(binary)), expected, "the C++");
}

/// `@!interactive`: every `print` is written out as it is said, with no
/// `flush()` and no read after it, in the interpreter and in the C++.
/// It stands with `@!multitest` in either order, once, at the top level.
#[test]
fn interactive_writes_each_print_out_at_once() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("differential/interactive");
    std::fs::create_dir_all(&dir).unwrap();
    let write = |name: &str, source: &str| {
        let path = dir.join(name);
        std::fs::write(&path, source).unwrap();
        path.to_string_lossy().into_owned()
    };
    // The program never reads and never ends: the line is seen only if
    // `print` itself wrote it out.
    let busy = "print(\"! ready\")\nvar k = 0\nwhile k >= 0: k = (k + 1) % 7\n";
    let path = write("busy.cp", &format!("@!interactive\n{busy}"));
    let mut interpreted = Command::new(env!("CARGO_BIN_EXE_cpl"));
    interpreted.args(["run", &path]);
    assert_eq!(asked_live(interpreted), ["! ready"], "`cpl run`");
    let built = cpl_cli::run(&["build", &path, "--checks", "traps", "-o", "-"]);
    assert_eq!(built.code, 0, "{}", built.stderr);
    let binary = sanitized_binary(&built.stdout).unwrap();
    assert_eq!(asked_live(Command::new(binary)), ["! ready"], "the C++");

    let rounds = "let n: Int = read()\nprint(\"?\", n)\nlet x: Int = read()\nprint(\"!\", x + n)\n";
    for head in [
        "@!interactive\n@!multitest\n",
        "@!multitest\n@!interactive\n",
    ] {
        let path = write("rounds.cp", &format!("{head}{rounds}"));
        let ran = cpl_cli::run_with_input(&["run", &path], b"2\n5\n7\n3\n1\n");
        assert_eq!(
            (ran.code, String::from_utf8_lossy(&ran.stdout).as_ref()),
            (0, "? 5\n! 12\n? 3\n! 4\n"),
            "{head}"
        );
        let formatted = cpl_cli::run(&["fmt", "--check", &path]);
        assert_eq!(formatted.code, 0, "{}", formatted.stderr);
    }
    for (source, said) in [
        (
            "@!interactive\nprint(1)\n@!interactive\nprint(2)\n",
            "a second `@!interactive`",
        ),
        (
            "fn f():\n    @!interactive\n    print(1)\nf()\n",
            "`@!interactive` stands at the top level of the file",
        ),
        (
            "@!interactive(fast)\nprint(1)\n",
            "`@!interactive` takes nothing",
        ),
    ] {
        let checked = cpl_cli::run(&["check", &write("refused.cp", source)]);
        assert_ne!(checked.code, 0, "{source}");
        assert!(checked.stderr.contains(said), "{}", checked.stderr);
    }
}

/// Two arrays given for one `D` that start at different indexes are
/// refused in both back ends, in the program's own words: the `D` the
/// fn wrote, never the name the compiler made for it.
#[test]
fn arrays_for_one_domain_that_start_apart_are_refused_by_its_written_name() {
    let dir = std::env::temp_dir().join(format!("cpl-one-domain-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("apart.cp");
    let source = "fn dot<D: Index>(a: [D]Int, b: [D]Int) -> Int:\n    var s = 0\n    for i in D: s += a[i] * b[i]\n    s\nlet a: [0..3]Int = 1\nlet b: [1..=3]Int = 2\nprint(dot(a, b))\n";
    std::fs::write(&path, source).unwrap();
    let path = path.to_string_lossy().into_owned();
    for command in [
        vec!["run", &path],
        vec!["build", &path, "--checks", "traps", "-o", "-"],
    ] {
        let ran = cpl_cli::run(&command);
        assert_ne!(ran.code, 0, "{command:?}");
        assert!(
            ran.stderr.contains("indexed by the same `D` as an argument before it, which starts at 0; this one starts at 1"),
            "{}",
            ran.stderr
        );
        assert!(!ran.stderr.contains("D_0"), "{}", ran.stderr);
    }
}

/// Partial-injection `a.inverse()` traps on duplicate images in both
/// the interpreter and sanitized C++17.
#[test]
fn partial_inverse_duplicate_image_traps_in_interp_and_cpp() {
    let dir = std::env::temp_dir().join(format!("cpl-partial-inv-dup-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("dup.cp");
    let source = "\
domain P = 1..=4
domain V = 0..3
var a: [P]?V = none
a[P(1)] = V(2)
a[P(3)] = V(2)
let inv = a.inverse()
print(inv[V(2)]!.label)
";
    std::fs::write(&path, source).unwrap();
    let path_str = path.to_string_lossy().into_owned();
    let ran = cpl_cli::run(&["run", &path_str]);
    assert_ne!(
        ran.code,
        0,
        "expected interpreter trap, got stdout: {}",
        String::from_utf8_lossy(&ran.stdout)
    );
    assert!(
        ran.stderr.contains("duplicate image"),
        "expected duplicate image message, got: {}",
        ran.stderr
    );
    let built = cpl_cli::run(&["build", &path_str, "--checks", "traps", "-o", "-"]);
    assert_eq!(built.code, 0, "{}", built.stderr);
    let binary = sanitized_binary(&built.stdout).unwrap();
    let cpp = Command::new(&binary).output().unwrap();
    assert!(
        !cpp.status.success(),
        "expected sanitized C++ trap on duplicate image"
    );
    assert!(
        String::from_utf8_lossy(&cpp.stderr).contains("duplicate image"),
        "expected duplicate image message in C++, got: {}",
        String::from_utf8_lossy(&cpp.stderr)
    );
}

/// A `[D]Char` read skips spaces between chars like one `read()` per cell,
/// across the input's 64 KB reads, and fails the same way when chars run out.
#[test]
fn char_arrays_read_across_spaces_and_refills_agree() {
    let dir = std::env::temp_dir().join(format!("cpl-read-chars-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("chars.cp");
    std::fs::write(
        &path,
        "\
let n: 1..=1_000_000 = read()
let s: [0..n]Char = read()
domain Row = 0..2
domain Col = 0..3
let g: [Row, Col]Char = read()
var h = 0
for c in s:
    h = (h * 31 + Int(c)) % 1_000_000_007
print(h, s[0], s[n - 1], g[Row(1), Col(2)])
",
    )
    .unwrap();
    let path_str = path.to_string_lossy().into_owned();
    let n = 150_000;
    let mut long = String::new();
    for i in 0..n {
        long.push((b'a' + (i % 26) as u8) as char);
        if i % 7919 == 0 {
            long.push_str(" \n\t");
        }
    }
    let good = format!("{n}\n{long}\nab c\ndef\n");
    let short = format!("{n}\n{}\n", &long[..1000]);
    let built = cpl_cli::run(&["build", &path_str, "--checks", "traps", "-o", "-"]);
    assert_eq!(built.code, 0, "{}", built.stderr);
    let binary = sanitized_binary(&built.stdout).unwrap();
    for (input, ok) in [(good, true), (short, false)] {
        let input_path = dir.join("in.txt");
        std::fs::write(&input_path, &input).unwrap();
        let interp = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(["run", &path_str])
            .stdin(std::fs::File::open(&input_path).unwrap())
            .output()
            .unwrap();
        let cpp = Command::new(&binary)
            .stdin(std::fs::File::open(&input_path).unwrap())
            .output()
            .unwrap();
        assert_eq!(
            interp.status.success(),
            ok,
            "{}",
            String::from_utf8_lossy(&interp.stderr)
        );
        assert_eq!(
            cpp.status.success(),
            ok,
            "{}",
            String::from_utf8_lossy(&cpp.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&interp.stdout),
            String::from_utf8_lossy(&cpp.stdout)
        );
    }
}

/// Separate `compress` call sites get distinct fresh domains in checker, IR,
/// interpreter, `--checked`, and `--release` C++ (top-level, inside a fn called
/// twice, wrapper fn returning `compress`, and `@!multitest` loop).
#[test]
fn compress_call_sites_get_fresh_domains_in_interp_checked_and_release() {
    let dir = std::env::temp_dir().join(format!("cpl-compress-fresh-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    // 1. Top-level, inside a fn called twice, and a wrapper fn called twice.
    let main_cp = dir.join("main.cp");
    let main_src = "\
let (domain X, xs) = compress([40, -7, 40, 1_000_000_000, 3])
let (domain Y, ys) = compress([5, 6])
let ix = xs.at(40)
let iy = ys.at(6)
print(xs.len, ys.len, xs[ix], ys[iy])

fn work(a: List<Int>, b: List<Int>):
    let (domain U, u) = compress(a)
    let (domain W, v) = compress(b)
    let iu = u.at(a[0])
    let iv = v.at(b[0])
    print(u.len, v.len, u[iu], v[iv])

work([10, 20, 30, 10], [99])
work([5, 5], [1, 2, 3, 4, 5])
";
    std::fs::write(&main_cp, main_src).unwrap();
    let main_str = main_cp.to_string_lossy().into_owned();
    let expected_main = "4 2 40 6\n3 1 10 99\n1 5 5 1\n";

    let interp = cpl_cli::run(&["run", &main_str]);
    assert_eq!(interp.code, 0, "interp stderr: {}", interp.stderr);
    assert_eq!(String::from_utf8_lossy(&interp.stdout), expected_main);

    let main_cpp = dir.join("main.cpp");
    let main_cpp_str = main_cpp.to_string_lossy().into_owned();
    let built_chk = cpl_cli::run(&["build", &main_str, "-o", &main_cpp_str, "--checked"]);
    assert_eq!(
        built_chk.code, 0,
        "--checked build stderr: {}",
        built_chk.stderr
    );
    let chk_out = Command::new(dir.join("main.checked")).output().unwrap();
    assert!(
        chk_out.status.success(),
        "--checked stderr: {}",
        String::from_utf8_lossy(&chk_out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&chk_out.stdout), expected_main);

    let rel = cpl_cli::run(&["run", "--release", &main_str]);
    assert_eq!(rel.code, 0, "--release stderr: {}", rel.stderr);
    assert_eq!(String::from_utf8_lossy(&rel.stdout), expected_main);

    // 2. Same pattern inside a `@!multitest` loop.
    let multi_cp = dir.join("multi.cp");
    let multi_src = "\
@!multitest
let n: Int = read()
let m: Int = read()
let a: List<Int> = [read() for _ in 0..n]
let b: List<Int> = [read() for _ in 0..m]
let (domain X, xs) = compress(a)
let (domain Y, ys) = compress(b)
let ix = xs.at(a[0])
let iy = ys.at(b[0])
print(xs.len, ys.len, xs[ix], ys[iy])
";
    std::fs::write(&multi_cp, multi_src).unwrap();
    let multi_str = multi_cp.to_string_lossy().into_owned();
    let multi_in = dir.join("multi.in");
    let multi_in_str = multi_in.to_string_lossy().into_owned();
    let input_bytes = b"2\n4 2\n40 -7 40 3\n5 6\n1 3\n9\n10 20 30\n";
    std::fs::write(&multi_in, input_bytes).unwrap();
    let expected_multi = "3 2 40 5\n1 3 9 10\n";

    let m_interp = cpl_cli::run(&["run", &multi_str, &multi_in_str]);
    assert_eq!(m_interp.code, 0, "multi interp stderr: {}", m_interp.stderr);
    assert_eq!(String::from_utf8_lossy(&m_interp.stdout), expected_multi);

    let multi_cpp = dir.join("multi.cpp");
    let multi_cpp_str = multi_cpp.to_string_lossy().into_owned();
    let m_built_chk = cpl_cli::run(&["build", &multi_str, "-o", &multi_cpp_str, "--checked"]);
    assert_eq!(
        m_built_chk.code, 0,
        "multi --checked stderr: {}",
        m_built_chk.stderr
    );
    let mut m_chk_cmd = Command::new(dir.join("multi.checked"));
    m_chk_cmd.stdin(std::fs::File::open(&multi_in).unwrap());
    let m_chk_out = m_chk_cmd.output().unwrap();
    assert!(
        m_chk_out.status.success(),
        "multi --checked run stderr: {}",
        String::from_utf8_lossy(&m_chk_out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&m_chk_out.stdout), expected_multi);

    let m_rel = cpl_cli::run(&["run", "--release", &multi_str, &multi_in_str]);
    assert_eq!(m_rel.code, 0, "multi --release stderr: {}", m_rel.stderr);
    assert_eq!(String::from_utf8_lossy(&m_rel.stdout), expected_multi);

    // 3. Cross-domain mixing between two `compress` call sites is rejected.
    let err_cp = dir.join("err.cp");
    std::fs::write(
        &err_cp,
        "let (domain X, xs) = compress([1, 2, 3, 4])\nlet (domain Y, ys) = compress([10, 20])\nlet i: X = ys.at(10)\nprint(i)\n",
    )
    .unwrap();
    let err_str = err_cp.to_string_lossy().into_owned();
    let checked = cpl_cli::run(&["check", &err_str]);
    assert_ne!(
        checked.code, 0,
        "expected domain mismatch between separate compress calls"
    );
}

#[test]
fn nth_element_introselect_handles_median_of_3_killer_in_budget() {
    let dir = std::env::temp_dir().join(format!("cpl-nth-killer-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let cp_path = dir.join("killer.cp");
    let src = "\
let n = 200_000
var a: List<Int> = [i for i in 0..n]
var lo = n - 2
while lo >= 0:
    let mid = lo + ((n - 1 - lo) >> 1)
    a.swap(lo + 1, mid)
    lo -= 2

var b = a.copy()
a.select(n - 1)
assert a[n - 1] == n - 1
for i in 0..n - 1: assert a[i] <= n - 1

b.select(n / 2)
assert b[n / 2] == n / 2
for i in 0..n / 2: assert b[i] <= n / 2
for i in n / 2 + 1..n: assert b[i] >= n / 2

print(a[n - 1], b[n / 2])
";
    std::fs::write(&cp_path, src).unwrap();
    let cp_str = cp_path.to_string_lossy().into_owned();
    let cpp_path = dir.join("killer.cpp");
    let cpp_str = cpp_path.to_string_lossy().into_owned();
    let built = cpl_cli::run(&["build", &cp_str, "--checks", "traps", "-o", &cpp_str]);
    assert_eq!(built.code, 0, "build stderr: {}", built.stderr);

    let bin_path = dir.join("killer");
    let gxx = Command::new("g++")
        .args(["-std=c++17", "-O2", &cpp_str, "-o"])
        .arg(&bin_path)
        .output()
        .expect("run g++");
    assert!(
        gxx.status.success(),
        "g++ stderr: {}",
        String::from_utf8_lossy(&gxx.stderr)
    );

    let t0 = std::time::Instant::now();
    let run = Command::new(&bin_path).output().expect("run killer binary");
    let elapsed = t0.elapsed();
    assert!(
        run.status.success(),
        "killer stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "199999 100000\n");
    assert!(
        elapsed < std::time::Duration::from_millis(1000),
        "introselect median-of-3 killer took {:?}, expected < 1s (O(n^2) regression?)",
        elapsed
    );
}
