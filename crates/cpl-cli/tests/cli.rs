use std::process::Command;

#[test]
fn binary_reports_its_version() {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("--version")
        .output()
        .expect("cpl runs");
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("cpl {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn binary_rejects_unknown_commands() {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("frobnicate")
        .output()
        .expect("cpl runs");
    assert_eq!(out.status.code(), Some(2));
}

/// Frames one LSP message.
fn frame(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}

#[test]
fn lsp_serves_diagnostics_and_hover_over_stdio() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["lsp", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cpl lsp starts");
    let text =
        "let n: 1..=2e5 = read()\\ndomain V = 1..=n\\nvar dist: [V]?Int = none\\nprint(dist[n])\\n";
    let session = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#.to_string(),
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file:///a.cp","languageId":"cp","version":1,"text":"{text}"}}}}}}"#
        ),
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///a.cp"},"position":{"line":3,"character":7}}}"#.to_string(),
        r#"{"jsonrpc":"2.0","id":3,"method":"shutdown","params":null}"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"exit","params":null}"#.to_string(),
    ];
    let mut stdin = child.stdin.take().unwrap();
    for message in &session {
        stdin.write_all(frame(message).as_bytes()).unwrap();
    }
    drop(stdin);
    let out = child.wait_with_output().expect("cpl lsp exits");
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(r#""hoverProvider":true"#), "{stdout}");
    assert!(stdout.contains(r#""code":"E0301""#), "{stdout}");
    assert!(
        stdout.contains("`dist` is indexed by V, but `n` is a 1..=2e5"),
        "{stdout}"
    );
    assert!(
        stdout.contains(r"dist: [V]?Int\n  domain V = 1..=n, n <= 2e5\n  storage: 8 bytes per element, 1.6 MB max"),
        "{stdout}"
    );
}

#[test]
fn check_exits_nonzero_and_prints_the_error() {
    let dir = std::env::temp_dir().join(format!("cpl-cli-bin-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("typo.cp");
    std::fs::write(&path, "var total = 0\nprint(totl)\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("check")
        .arg(&path)
        .output()
        .expect("cpl runs");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("error[E0201]: no `totl` in scope"),
        "{stderr}"
    );
    assert!(stderr.contains("= help: did you mean `total`?"), "{stderr}");
}

#[test]
fn a_read_that_names_a_domain_inside_a_tuple_is_an_error() {
    let dir = std::env::temp_dir().join(format!("cpl-cli-tuple-read-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for ty in ["([domain Pos]Char, Int)", "([domain Pos]Int, Int)"] {
        let path = dir.join("tuple_read.cp");
        std::fs::write(&path, format!("let (s, k): {ty} = read()\nprint(Pos.K, k)\n")).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_cpl")).arg("check").arg(&path).output().expect("cpl runs");
        assert_eq!(out.status.code(), Some(1));
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(stderr.contains("error[E0501]: `read()` can't size a domain named inside"), "{stderr}");
    }
}

/// A fresh directory holding `name` with `source` in it.
fn scratch(test: &str, name: &str, source: &[u8]) -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, source).unwrap();
    path
}

/// Runs `cpl` with `args` and `input` on stdin.
fn cpl(args: &[&std::ffi::OsStr], input: &[u8]) -> std::process::Output {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cpl starts");
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().expect("cpl exits")
}

#[test]
fn run_reads_stdin_and_prints_the_answer() {
    let path = scratch(
        "run-answer",
        "sum.cp",
        b"let n: Int = read()\nlet a: [n]Int = read()\nprint(sum(a))\n",
    );
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"3\n10 20 12\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"42\n");
    assert!(out.stderr.is_empty());
}

#[test]
fn run_exits_3_where_the_program_fails_and_keeps_what_it_printed() {
    let path = scratch(
        "run-trap",
        "trap.cp",
        b"let xs: List<Int> = [1, 2]\nprint(\"before\")\nprint(xs[5])\n",
    );
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"");
    assert_eq!(out.status.code(), Some(3));
    assert_eq!(out.stdout, b"before\n");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("error[R0001]: index 5 is outside a list of length 2"),
        "{stderr}"
    );
    assert!(stderr.contains("trap.cp:3:7"), "{stderr}");
}

#[test]
fn strings_are_bytes_and_literals_are_ascii() {
    // Bytes that are not ASCII pass from input to output untouched, one
    // `Char` each.
    let path = scratch(
        "bytes",
        "echo.cp",
        b"let s: Str = read()\nprint(s, s.len)\n",
    );
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"\xc3\xa9\xff\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"\xc3\xa9\xff 3\n");

    // A literal can't hold them: there is no encoding to pick.
    let path = scratch(
        "non-ascii",
        "accent.cp",
        "print(\"h\u{e9}llo\")\n".as_bytes(),
    );
    for command in ["run", "build"] {
        let out = cpl(&[command.as_ref(), path.as_os_str()], b"");
        assert_eq!(out.status.code(), Some(1), "{command}");
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(stderr.contains("error[E0502]"), "{stderr}");
        assert!(out.stdout.is_empty());
    }
}

#[test]
fn build_writes_cpp_that_opens_with_the_source() {
    let source = "// Doubles a number. \\\nlet n: Int = read()\nprint(n * 2)\n";
    let path = scratch("build-out", "double.cp", source.as_bytes());

    // Beside the source by default.
    let out = cpl(&["build".as_ref(), path.as_os_str()], b"");
    assert_eq!(out.status.code(), Some(0));
    let cpp = std::fs::read_to_string(path.with_extension("cpp")).unwrap();
    assert!(
        cpp.starts_with("// double.cp: C++17 written by cpl"),
        "{cpp}"
    );
    assert!(
        cpp.contains("//     let n: Int = read()\n//     print(n * 2)\n"),
        "{cpp}"
    );
    // A source line ending in `\` must not swallow the C++ line after it.
    assert!(!cpp.lines().any(|line| line.ends_with('\\')), "{cpp}");
    assert!(cpp.contains("int main()"), "{cpp}");

    // Where `-o` says, and to stdout for `-o -`: the same bytes.
    let target = path.with_file_name("elsewhere.cc");
    let out = cpl(
        &[
            "build".as_ref(),
            path.as_os_str(),
            "-o".as_ref(),
            target.as_os_str(),
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(std::fs::read_to_string(&target).unwrap(), cpp);
    let out = cpl(
        &[
            "build".as_ref(),
            path.as_os_str(),
            "-o".as_ref(),
            "-".as_ref(),
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8(out.stdout).unwrap(), cpp);
}

#[test]
fn build_drops_the_source_header_before_it_passes_64_kb() {
    // 60 KB of source: the C++ fits, the C++ and the source do not.
    let mut source = String::new();
    for i in 0..1500 {
        source += &format!("// {i}: {}\n", "padding ".repeat(4));
    }
    source += "print(1 + 1)\n";
    let path = scratch("build-headerless", "long.cp", source.as_bytes());
    let out = cpl(
        &[
            "build".as_ref(),
            path.as_os_str(),
            "-o".as_ref(),
            "-".as_ref(),
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    let cpp = String::from_utf8(out.stdout).unwrap();
    assert!(cpp.starts_with("// long.cp: C++17 written by cpl"), "{cpp}");
    assert!(
        !cpp.contains("padding") && cpp.contains("int main()"),
        "{cpp}"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("the header is left out"), "{stderr}");
}

#[test]
fn build_refuses_cpp_over_64_kb() {
    let mut source = String::new();
    for i in 0..6000 {
        source += &format!("print({i} + {i})\n");
    }
    let path = scratch("build-big", "big.cp", source.as_bytes());
    let out = cpl(
        &[
            "build".as_ref(),
            path.as_os_str(),
            "-o".as_ref(),
            "-".as_ref(),
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("over the 65536-byte limit"), "{stderr}");
}

/// A guessing game with its judge in the file: the solution halves the
/// range, and the judge holds the secret.
const GUESS: &str = "\
let n: Int = read()
var lo = 1
var hi = n
while lo < hi:
    let mid = (lo + hi) / 2
    let answer: Str = ask(\"?\", mid, limit: LIMIT)
    if answer == \"<\": hi = mid
    else: lo = mid + 1
ask(\"!\", ANSWER)

@interactor
fn judge(mut sol: Interactor):
    let n: 1..=1000 = read()
    let secret: 1..=n = read()
    sol.send(n)
    while true:
        let kind: Str = sol.recv()
        let x: Int = sol.recv()
        if kind == \"!\":
            assert x == secret
            return
        sol.send(if secret <= x: \"<\" else: \">=\")
";

fn guess(test: &str, limit: &str, answer: &str) -> std::path::PathBuf {
    let source = GUESS.replace("LIMIT", limit).replace("ANSWER", answer);
    scratch(test, "guess.cp", source.as_bytes())
}

#[test]
fn run_plays_the_solution_against_the_interactor_in_its_file() {
    let path = guess("interactor-accepts", "10", "lo");
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"8 3\n");
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    // The hidden test is the judge's: the solution hears `8` from it.
    assert_eq!(out.stdout, b"? 4\n? 2\n? 3\n! 3\n");
    assert!(String::from_utf8_lossy(&out.stderr).contains("the interactor `judge` accepted the run"));
}

#[test]
fn run_reports_the_interactor_rejecting_a_wrong_answer() {
    let path = guess("interactor-rejects", "10", "lo + 1");
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"8 3\n");
    assert_eq!(out.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the interactor `judge` rejected the run"), "{stderr}");
    assert!(stderr.contains("assertion failed: x == secret"), "{stderr}");
}

#[test]
fn run_stops_a_solution_that_asks_more_than_its_limit() {
    let path = guess("interactor-limit", "2", "lo");
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"1000 637\n");
    assert_eq!(out.status.code(), Some(3));
    assert_eq!(out.stdout, b"? 500\n? 750\n");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("query limit exceeded: this is query 3, and the limit is 2"),
        "{stderr}"
    );
}

#[test]
fn run_ends_a_conversation_where_both_sides_wait() {
    // The solution reads a second number the judge never sends.
    let source = GUESS.replace("LIMIT", "10").replace("ask(\"!\", ANSWER)", "let more: Int = read()");
    let path = scratch("interactor-deadlock", "guess.cp", source.as_bytes());
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"8 3\n");
    assert_eq!(out.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the input ended where a number was expected"), "{stderr}");
}

#[test]
fn judge_is_the_interactor_as_a_program_on_stdin_and_stdout() {
    let path = guess("interactor-judge", "10", "lo");
    let hidden = path.with_file_name("hidden.in");
    std::fs::write(&hidden, "8 3\n").unwrap();
    let args = ["judge".as_ref(), path.as_os_str(), hidden.as_os_str()];
    let out = cpl(&args, b"? 4\n? 2\n? 3\n! 3\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"8\n<\n>=\n<\n");
    let out = cpl(&args, b"! 5\n");
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("assertion failed: x == secret"));
}

#[test]
fn stress_judges_with_the_interactor_in_the_file() {
    let gen = "#!/bin/sh\nn=$(( ($1 * 37) % 1000 + 1 ))\necho \"$n $(( ($1 * 13) % n + 1 ))\"\n";
    let stress = |test: &str, answer: &str| {
        let path = guess(test, "10", answer);
        let script = path.with_file_name("gen.sh");
        std::fs::write(&script, gen).unwrap();
        #[cfg(unix)]
        std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
        let args = [
            "stress".as_ref(),
            path.as_os_str(),
            "--gen".as_ref(),
            script.as_os_str(),
            "--cases".as_ref(),
            "12".as_ref(),
        ];
        let out = cpl(&args, b"");
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
    };
    let right = stress("interactor-stress-ok", "lo");
    assert!(right.contains("12 cases agree"), "{right}");
    let wrong = stress("interactor-stress-wa", "lo + 1");
    assert!(wrong.contains("interactor: assertion failed: x == secret"), "{wrong}");
}

/// Wrong on a tree with a vertex of degree 3 whose value is above 50.
const TREE: &str = "\
let n: 1..=100 = read()
domain V = 1..=n
let edges: [n - 1](V, V) = read()
let a: [V]Int = read()

var deg: [V]Int = 0
for (u, v) in edges:
    deg[u] += 1
    deg[v] += 1
var bad = false
for v in V:
    if deg[v] >= 3 and a[v] > 50: bad = true
print(if bad: 0 else: sum(a[v] for v in V))

@brute(main)
fn slow():
    print(sum(a[v] for v in V))

@gen
fn make_tree(size: Int):
    let n = rand(1, size)
    print(n)
    for v in 2..=n:
        print(rand(1, v - 1), v)
    print([rand(1, 100) for _ in 0..n])
";

#[test]
fn stress_makes_inputs_with_the_gen_fn_in_the_file_and_shrinks_them() {
    let path = scratch("gen-stress", "tree.cp", TREE.as_bytes());
    let out = cpl(&["stress".as_ref(), path.as_os_str(), "--size".as_ref(), "30".as_ref()], b"");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    // The smallest such tree: a star of four, one value just above 50.
    assert!(stdout.contains(" numbers to 11:\n    4\n    1 2\n    1 3\n    1 4\n    51 1 1 1\n"), "{stdout}");
}

#[test]
fn a_gen_fn_that_uses_the_input_is_an_error() {
    let source = "let n: 1..=9 = read()\nprint(n)\n\n@gen\nfn make():\n    print(n)\n";
    let path = scratch("gen-input", "a.cp", source.as_bytes());
    let out = cpl(&["check".as_ref(), path.as_os_str()], b"");
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("`n` is read from the input, which a `@gen` fn is there to make"), "{text}");
}

const LAWS: &str = "\
fn add(a: Int, b: Int) -> Int = a + b
fn clamp(x: Int) -> Int:
    if x < 0: return 0
    return x

@law comm(a: Int, b: Int) = add(a, b) == add(b, a)
@law clamp_idem(x: Int) = clamp(clamp(x)) == clamp(x)
";

#[test]
fn test_checks_each_law_and_a_valid_law_passes() {
    let path = scratch("laws-hold", "laws.cp", LAWS.as_bytes());
    let out = cpl(&["test".as_ref(), path.as_os_str()], b"");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert!(stdout.contains("::law_comm") && stdout.contains("::law_clamp_idem"), "{stdout}");
    assert!(stdout.contains("2 passed, 0 failed"), "{stdout}");
}

#[test]
fn test_fails_a_broken_law_with_the_values_that_break_it() {
    let source = format!("{LAWS}@law clamp_id(x: Int) = clamp(x) == x\n");
    let path = scratch("laws-break", "laws.cp", source.as_bytes());
    let out = cpl(&["test".as_ref(), path.as_os_str()], b"");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_ne!(out.status.code(), Some(0), "{stdout}");
    assert!(
        stdout.contains("FAIL") && stdout.contains("law `clamp_id` does not hold for x = -1"),
        "{stdout}"
    );
    assert!(stdout.contains("2 passed, 1 failed"), "{stdout}");
}

#[test]
fn messages_call_the_float_type_f64() {
    let path = scratch("f64-name", "f.cp", b"let x: F64 = 1.5\nlet y: Int = x\n");
    let out = cpl(&["check".as_ref(), path.as_os_str()], b"");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("F64") && !stderr.contains("Float"), "{stderr}");
}

/// A table above `@!multitest` that takes far more steps than a trace
/// holds, and a `@checker` as the only judge. Wrong from `n = 3` on.
const TABLE: &str = "\
var f: [0..3000]Int = 0
for i in 1..3000: f[i] = f[i - 1] + i
@!multitest
let n: 1..=20 = read()
let a: [n](1..=9) = read()
print(if n >= 3: 0 else: sum(a))

@checker
fn check():
    let got: Int = read_output()
    assert got == sum(a)
";

#[test]
fn stress_traces_the_failing_test_case_of_a_file_judged_by_its_checker() {
    let path = scratch("checker-trace", "table.cp", TABLE.as_bytes());
    let out = cpl(&["stress".as_ref(), path.as_os_str(), "--trace".as_ref()], b"");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("  got:\n    0\n  checker: assertion failed: got == sum(a)"), "{stdout}");
    assert!(!stdout.contains("expected:"), "{stdout}");
    let trace = std::fs::read_to_string(path.with_extension("tests").join("stress-1.trace.html")).unwrap();
    // The steps of the test case are there, past the table's.
    assert!(trace.contains("\"line\":6"), "the trace stops above `@!multitest`");
}

/// `cpl build --checked` stops on what `cpl run` stops on, where the
/// sanitizers alone would go on with a wrong value: an `I128` shifted past
/// its top, a negative power, a `U64` divided by zero.
#[test]
fn a_checked_build_stops_on_wide_int_mistakes() {
    use std::io::Write;
    use std::process::Stdio;

    if Command::new("g++").arg("--version").output().is_err() {
        return;
    }
    let dir = std::env::temp_dir().join(format!("cpl-cli-checked-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let cases = [
        (
            "shift",
            "let a: I128 = 1\nlet k: Int = read()\nprint(Int((a << k) >> 100))\n",
            "127",
            "does not fit in an `I128`",
        ),
        (
            "power",
            "let k: Int = read()\nprint(2 ** k)\n",
            "-1",
            "needs a power that is not negative",
        ),
        (
            "divide",
            "let a: U64 = 7\nlet k: Int = read()\nlet z: U64 = k\nprint(a / z)\n",
            "0",
            "division by zero",
        ),
    ];
    for (name, source, input, message) in cases {
        let file = dir.join(format!("{name}.cp"));
        std::fs::write(&file, source).unwrap();
        let built = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(["build", "--checked"])
            .arg(&file)
            .output()
            .expect("cpl runs");
        assert!(built.status.success(), "{name}: {built:?}");
        let mut child = Command::new(dir.join(format!("{name}.checked")))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the checked binary runs");
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        let out = child.wait_with_output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "{name} went on: {out:?}");
        assert!(stderr.contains(message), "{name}: {stderr}");
    }
    std::fs::remove_dir_all(&dir).ok();
}
