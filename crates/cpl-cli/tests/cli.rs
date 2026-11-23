use std::process::Command;

#[test]
fn binary_reports_its_version() {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("--version")
        .output()
        .expect("cpl runs");
    assert!(out.status.success());
    let line = String::from_utf8(out.stdout).unwrap();
    assert!(
        line.starts_with(&format!("cpl {}-dev+", env!("CARGO_PKG_VERSION"))),
        "{line}"
    );
    assert!(line.ends_with('\n') && line.lines().count() == 1, "{line}");
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
        "let n: 1..=2e5 = read()\\ndomain V = 1..=n\\nvar dist: [V]?Int = None\\nprint(dist[n])\\n";
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
fn lsp_reports_what_check_reports_against_the_spec() {
    use std::io::Write;
    use std::process::Stdio;

    let dir = std::env::temp_dir().join(format!("cpl-cli-spec-lsp-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("a.cp");
    let text = "domain V = 1..=3\nlet par: [V]V = [V] v => v\nlet f = Functional<V>.new(par)\nprint(yes(f.on_cycle(V(1))))\n";
    std::fs::write(&file, text).unwrap();
    let check = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("check")
        .arg(&file)
        .output()
        .expect("cpl check runs");
    let check_err = String::from_utf8_lossy(&check.stderr).into_owned();

    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["lsp", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cpl lsp starts");
    // `file:///D:/...` on Windows: a `\` in the path is not valid in the JSON either.
    let path = file.display().to_string().replace('\\', "/");
    let uri = format!("file:///{}", path.trim_start_matches('/'));
    let session = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.to_string(),
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"{uri}","languageId":"cp","version":1,"text":"{}"}}}}}}"#,
            text.replace('\n', "\\n")
        ),
        r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"exit","params":null}"#.to_string(),
    ];
    let mut stdin = child.stdin.take().unwrap();
    for message in &session {
        stdin.write_all(frame(message).as_bytes()).unwrap();
    }
    drop(stdin);
    let out = child.wait_with_output().expect("cpl lsp exits");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("publishDiagnostics"), "{stdout}");
    assert_eq!(
        stdout.contains(r#""severity":1"#),
        !check.status.success(),
        "check:\n{check_err}\nlsp:\n{stdout}"
    );
    for code in check_err
        .lines()
        .filter_map(|l| l.strip_prefix("error[")?.split(']').next())
    {
        assert!(
            stdout.contains(&format!(r#""code":"{code}""#)),
            "check reports {code}; lsp:\n{stdout}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
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
        std::fs::write(
            &path,
            format!("let (s, k): {ty} = read()\nprint(Pos.len, k)\n"),
        )
        .unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .arg("check")
            .arg(&path)
            .output()
            .expect("cpl runs");
        assert_eq!(out.status.code(), Some(1));
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(
            stderr.contains("error[E0501]: `read()` can't size a domain named inside"),
            "{stderr}"
        );
    }
}

#[test]
fn a_range_of_u64_is_unsupported_not_a_mismatch() {
    let dir = std::env::temp_dir().join(format!("cpl-cli-wide-range-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for src in [
        "let r = U64(0)..=U64(3)\nprint(r.len)\n",
        "let lo = U64(1)\nprint((lo..=U64(10)).first_true(x => x >= U64(3))!)\n",
    ] {
        let path = dir.join("wide_range.cp");
        std::fs::write(&path, src).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .arg("check")
            .arg(&path)
            .output()
            .expect("cpl runs");
        assert_eq!(out.status.code(), Some(1));
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(
            stderr.contains("can't handle a range of `U64` values"),
            "{stderr}"
        );
        assert!(!stderr.contains("expected `U64`, found `U64`"), "{stderr}");
    }
}

#[test]
fn a_statement_that_only_works_out_a_value_is_a_warning() {
    let dir = std::env::temp_dir().join(format!("cpl-cli-unused-value-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("unused.cp");
    let check = |body: &str| {
        let source = format!("domain V = 0..4\nvar xs = [3, 1, 2]\nvar s = Subset<V>.single(V(0))\nvar heap = MinHeap<Int>()\n@droppable\nfn loud(v: Int) -> Int:\n    print(v)\n    v + 1\n{body}\nprint(xs, s.len, heap.len)\n");
        std::fs::write(&path, source).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .arg("check")
            .arg(&path)
            .output()
            .expect("cpl runs");
        assert_eq!(out.status.code(), Some(0), "{body}");
        String::from_utf8(out.stderr).unwrap()
    };
    for dropped in [
        "xs.rev",
        "xs.sorted()",
        "sorted(xs)",
        "s.with(V(1))",
        "xs.deduped()",
        "xs.len",
        "xs[0] + 1",
    ] {
        let stderr = check(dropped);
        assert!(stderr.contains("warning[W0101]"), "{dropped}: {stderr}");
    }
    for kept in [
        "xs.sort()",
        "xs.reverse()",
        "xs.pop()",
        "heap.push(3)\nheap.pop()",
        "loud(3)",
        "xs.swap(0, 1)",
        "s.set(V(1), true)",
        "let ys = xs.sorted()",
    ] {
        let stderr = check(kept);
        assert_eq!(stderr, "", "{kept}");
    }
}

/// A fresh directory holding `name` with `source` in it.
#[test]
fn the_default_edition_checks_what_the_spec_rejects() {
    let path = scratch(
        "the_default_edition_checks_what_the_spec_rejects",
        "a.cp",
        b"let a: ?Int = None\nprint(a)\nprint(a, none: -1)\n",
    );
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("check")
        .arg(&path)
        .output()
        .expect("cpl runs");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("`a` has type `?Int`, which does not implement `Show`"),
        "{stderr}"
    );
    assert_eq!(stderr.matches("error[").count(), 1, "{stderr}");
}

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
        stderr.contains("error[R0101]: index 5 is outside a list of length 2"),
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
    assert!(
        stderr.contains("the source in the header is left out"),
        "{stderr}"
    );
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
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The hidden test is the judge's: the solution hears `8` from it.
    assert_eq!(out.stdout, b"? 4\n? 2\n? 3\n! 3\n");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("the interactor `judge` accepted the run")
    );
}

#[test]
fn run_reports_the_interactor_rejecting_a_wrong_answer() {
    let path = guess("interactor-rejects", "10", "lo + 1");
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"8 3\n");
    assert_eq!(out.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("the interactor `judge` rejected the run"),
        "{stderr}"
    );
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
    let source = GUESS
        .replace("LIMIT", "10")
        .replace("ask(\"!\", ANSWER)", "let more: Int = read()");
    let path = scratch("interactor-deadlock", "guess.cp", source.as_bytes());
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"8 3\n");
    assert_eq!(out.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("the input ended where a number was expected"),
        "{stderr}"
    );
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

// The generator is a `#!/bin/sh` script, which `cpl stress` runs through
// `sh` on every OS (Git for Windows' on Windows).
#[test]
fn stress_judges_with_the_interactor_in_the_file() {
    let gen = "#!/bin/sh\nn=$(( ($1 * 37) % 1000 + 1 ))\necho \"$n $(( ($1 * 13) % n + 1 ))\"\n";
    let stress = |test: &str, limit: &str, answer: &str| {
        let path = guess(test, limit, answer);
        let script = path.with_file_name("gen.sh");
        std::fs::write(&script, gen).unwrap();
        #[cfg(unix)]
        std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .unwrap();
        let args = [
            "stress".as_ref(),
            path.as_os_str(),
            "--gen".as_ref(),
            script.as_os_str(),
            "--cases".as_ref(),
            "12".as_ref(),
        ];
        let out = cpl(&args, b"");
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    };
    let right = stress("interactor-stress-ok", "10", "lo");
    assert!(right.contains("12 cases agree"), "{right}");
    let wrong = stress("interactor-stress-wa", "10", "lo + 1");
    assert!(
        wrong.contains("interactor: assertion failed: x == secret"),
        "{wrong}"
    );
    // research/dryrun/LOG.md p19: the judge starving was all stress said.
    let over = stress("interactor-stress-limit", "3", "lo");
    assert!(
        over.contains("query limit exceeded: the limit is 3"),
        "{over}"
    );
    assert!(!over.contains("output ended"), "{over}");
}

/// A plain `Int` that sizes a domain, an array, a loop or a list is
/// generated as a count, from 1: a negative one stopped every such
/// program on case 1.
#[test]
fn stress_makes_a_read_int_that_sizes_something_a_count() {
    let brute = "@brute(main)\nfn slow():\n    print(n)\n";
    let programs = [
        ("size-domain", "let n: Int = read()\ndomain V = 0..n\nlet a: [V]Int = read()\nassert n >= 1\nprint(n)\n"),
        ("size-array", "let n: Int = read()\nvar dp: [1..=n]Int = 0\nprint(n)\n"),
        ("size-grid", "let (n, m): (Int, Int) = read()\ndomain V = 1..=n\nvar dp: [V, 0..m]Int = 0\nprint(n)\n"),
        ("size-in-fn", "let n: Int = read()\nfn rows() -> Int:\n    var a: [0..n + 1]Int = 0\n    a.len\nassert rows() == n + 1\nprint(n)\n"),
        ("size-loop", "let n: Int = read()\nvar t = 0\nfor _ in 0..n: t += 1\nassert t == n\nprint(n)\n"),
        ("size-filled", "let n: Int = read()\nlet xs = List.filled(n, 0)\nprint(n)\n"),
        ("size-multitest", "@!multitest\nlet n: Int = read()\nvar seen: [0..n]Bool = false\nprint(n)\n"),
    ];
    for (test, source) in programs {
        let path = scratch(test, "sized.cp", format!("{source}{brute}").as_bytes());
        let out = cpl(
            &[
                "stress".as_ref(),
                path.as_os_str(),
                "--cases".as_ref(),
                "40".as_ref(),
            ],
            b"",
        );
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(text.contains("40 cases agree"), "{test}: {text}");
    }
}

/// A plain `Int` gives no range, so the one `cpl stress` draws it from
/// is a guess. An input that stops the solution and the brute alike is
/// thrown away and the guess narrowed; one that stops only the solution
/// is still a bug.
#[test]
fn stress_throws_away_inputs_that_stop_both_programs_and_narrows_the_guess() {
    let agree = [
        (
            "narrow-minus-one",
            "let n: Int = read()\n@brute(main)\nfn slow():\n    let m = n - 1\n    var c: [0..m]Int = 0\n    print(c.len)\nlet m = n - 1\nvar dp: [0..m]Int = 0\nprint(dp.len)\n",
            "n is small and 1 or more",
        ),
        (
            "narrow-fn-size",
            "fn table(k: Int) -> List<Int>:\n    var t: [0..k]Int = 0\n    return [x for x in t]\nlet n: Int = read()\n@brute(main)\nfn slow():\n    print(table(n).len)\nprint(table(n).len)\n",
            "n is small and 1 or more",
        ),
        (
            "narrow-index",
            "let n: Int = read()\nlet s: [0..5]Int = read()\n@brute(main)\nfn slow():\n    var at = 0\n    for i in 0..n: at = i\n    print(s[at])\nprint(s[n - 1])\n",
            "n: 5 or less",
        ),
        // A cell of a list read by a comprehension is used as an index.
        (
            "narrow-cells-read-in-a-comprehension",
            "let n: Int = read()\nlet a: List<Int> = [read() for _ in 0..n]\n@brute(main)\nfn slow():\n    var c: [0..1001]Int = 0\n    for x in a: c[x] += 1\n    print(c.max()!)\nvar c: [0..1001]Int = 0\nfor x in a: c[x] += 1\nprint(c.max()!)\n",
            "cells of the arrays read: 0 to 8",
        ),
        (
            "narrow-cells-from-one",
            "let n: Int = read()\nlet a: [0..n]Int = read()\n@brute(main)\nfn slow():\n    var seen: [0..8]Int = 0\n    for x in a: seen[x - 1] += 1\n    print(seen.max()!)\nvar seen: [0..8]Int = 0\nfor x in a: seen[x - 1] += 1\nprint(seen.max()!)\n",
            "cells of the arrays read: 1 to 8",
        ),
    ];
    let stress = |test: &str, source: &str| {
        let path = scratch(test, "narrow.cp", source.as_bytes());
        let out = cpl(
            &[
                "stress".as_ref(),
                path.as_os_str(),
                "--cases".as_ref(),
                "40".as_ref(),
            ],
            b"",
        );
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    };
    for (test, source, learned) in agree {
        let text = stress(test, source);
        assert!(text.contains("40 cases agree"), "{test}: {text}");
        assert!(
            text.contains("stopped the program thrown away"),
            "{test}: {text}"
        );
        assert!(text.contains(learned), "{test}: {text}");
    }
    // Only the solution stops: the brute has an answer, so it is a bug.
    let only_solution = "let n: Int = read()\n@brute(main)\nfn slow():\n    print(max(n - 1, 0))\nlet m = n - 1\nvar dp: [0..m]Int = 0\nprint(dp.len)\n";
    let text = stress("narrow-real-stop", only_solution);
    assert!(
        text.contains("FAIL on case") && text.contains("an array can't have length"),
        "{text}"
    );
    // A cell stops only the solution: still a bug, cells are not narrowed.
    let only_solution = "let n: Int = read()\nlet a: [0..n]Int = read()\n@brute(main)\nfn slow():\n    print(sum(a))\nvar seen: [0..10]Int = 0\nfor x in a: seen[x] += 1\nprint(sum(a))\n";
    let text = stress("narrow-real-cell-stop", only_solution);
    assert!(
        text.contains("FAIL on case") && !text.contains("thrown away"),
        "{text}"
    );
    // Both run and differ: a wrong answer, never thrown away.
    let wrong = "let n: Int = read()\nlet a: [0..n]Int = read()\n@brute(main)\nfn slow():\n    print(sum(a))\nprint(sum(a) - a[0])\n";
    let text = stress("narrow-real-wrong", wrong);
    assert!(
        text.contains("FAIL on case") && !text.contains("thrown away"),
        "{text}"
    );
    assert!(!text.contains("cpl locate"), "no part to locate: {text}");
    // A file that pairs a part with its alt is told which call of the part is wrong.
    let part = "fn f(n: Int) -> Int = if n == 3: 0 else: n\n@brute(f)\nfn f_slow(n: Int) -> Int = n\nlet n: 1..=9 = read()\nprint(f(n))\n";
    let text = stress("narrow-part-wrong", part);
    assert!(
        text.contains("first wrong call: `f(n: 3)`")
            && text.contains("solution f(n: 3) = 0 at line 1")
            && text.contains("brute    f(n: 3) = 3 at line 3"),
        "{text}"
    );
    assert!(
        !text.contains("cpl locate"),
        "the part is named already: {text}"
    );
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
    let out = cpl(
        &[
            "stress".as_ref(),
            path.as_os_str(),
            "--size".as_ref(),
            "30".as_ref(),
        ],
        b"",
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    // The smallest such tree: a star of four, one value just above 50.
    assert!(
        stdout.contains(" numbers to 11:\n    4\n    1 2\n    1 3\n    1 4\n    51 1 1 1\n"),
        "{stdout}"
    );
}

#[test]
fn a_gen_fn_that_uses_the_input_is_an_error() {
    let source = "let n: 1..=9 = read()\nprint(n)\n\n@gen\nfn make():\n    print(n)\n";
    let path = scratch("gen-input", "a.cp", source.as_bytes());
    let out = cpl(&["check".as_ref(), path.as_os_str()], b"");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(
        text.contains("`n` is read from the input, which a `@gen` fn is there to make"),
        "{text}"
    );
}

/// research/dryrun/LOG.md p07: `(a: V, b: V)` edges gave no hint, and
/// the call line twice where the declaration belonged.
#[test]
#[ignore = "spec/conformance/diagnostics/graph-edge-field-names.cp"]
fn edges_with_other_field_names_are_told_which_to_rename() {
    let source = "let (n, m): (1..=10, 1..=10) = read()\ndomain V = 1..=n\n\
                  let flights: [0..m](a: V, b: V, c: 1..=1e9) = read()\nlet g = Graph.directed(flights)\nprint(n)\n";
    let path = scratch("graph-field-names", "a.cp", source.as_bytes());
    let out = cpl(&["check".as_ref(), path.as_os_str()], b"");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(
        text.contains("help: the fields are read by name: rename `a` to `u`, `b` to `v`"),
        "{text}"
    );
    assert!(text.contains("3 | let flights"), "{text}");
    assert_eq!(text.matches("4 | let g").count(), 1, "{text}");
}

#[test]
fn a_test_that_never_ends_fails_at_the_time_limit_and_the_rest_still_run() {
    let source = "\
fn spin() -> Int:
    var i = 0
    while i >= 0:
        i = i
    return i

@test
fn never_ends():
    assert spin() == 1

@test
fn ends():
    assert 1 + 1 == 2
";
    let path = scratch("test-never-ends", "spin.cp", source.as_bytes());
    let started = std::time::Instant::now();
    let out = cpl(&["test".as_ref(), path.as_os_str()], b"");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(3), "{stdout}");
    assert!(
        stdout.contains("FAIL") && stdout.contains("::never_ends: still running after"),
        "{stdout}"
    );
    assert!(stdout.contains("1 passed, 1 failed"), "{stdout}");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(30),
        "{:?}",
        started.elapsed()
    );
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
    assert!(
        stdout.contains("::law_comm") && stdout.contains("::law_clamp_idem"),
        "{stdout}"
    );
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
    assert!(
        stderr.contains("F64") && !stderr.contains("Float"),
        "{stderr}"
    );
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
    let out = cpl(
        &["stress".as_ref(), path.as_os_str(), "--trace".as_ref()],
        b"",
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("  got:\n    0\n  checker: assertion failed: got == sum(a)"),
        "{stdout}"
    );
    assert!(!stdout.contains("expected:"), "{stdout}");
    let trace =
        std::fs::read_to_string(path.with_extension("tests").join("stress-1.trace.html")).unwrap();
    // The steps of the test case are there, past the table's.
    assert!(
        trace.contains("\"line\":6"),
        "the trace stops above `@!multitest`"
    );
}

/// A `@check fn ok() -> Bool` that returns `false` rejects, as a failing
/// `assert` does, and one that returns `true` accepts a differing answer.
#[test]
fn test_judges_with_a_check_that_returns_a_bool() {
    let source = "let n: Int = read()\nprint(n - 1)\n\n@check\nfn ok() -> Bool:\n    let n: Int = read()\n    let out: Int = read_output()\n    out < n\n";
    let path = scratch("check-bool", "a.cp", source.as_bytes());
    let tests = path.with_extension("tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("1.in"), "5\n").unwrap();
    std::fs::write(tests.join("1.out"), "0\n").unwrap();
    let out = cpl(&["test".as_ref(), path.as_os_str()], b"");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success() && stdout.contains("1/1 passed"),
        "{stdout}"
    );
    std::fs::write(&path, source.replace("out < n", "out > n")).unwrap();
    let out = cpl(&["test".as_ref(), path.as_os_str()], b"");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success()
            && stdout.contains("checker: `ok()` returned false: the check rejects this output"),
        "{stdout}"
    );
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
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "{name} went on: {out:?}");
        assert!(stderr.contains(message), "{name}: {stderr}");
    }
    std::fs::remove_dir_all(&dir).ok();
}

/// Negative `sqrt` traps with the same message in `cpl run` and `cpl build --checked`
/// while release C++ (`-O2`, no `-DLOCAL`) stays raw.
#[test]
fn sqrt_negative_traps_in_interp_and_checked_cpp_not_release() {
    use std::io::Write;
    use std::process::Stdio;

    if Command::new("g++").arg("--version").output().is_err() {
        return;
    }
    let dir = std::env::temp_dir().join(format!("cpl-cli-sqrt-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("s.cp");
    std::fs::write(&file, "let x: F64 = read()\nprint(sqrt(x))\n").unwrap();

    let built = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["build", "--checked"])
        .arg(&file)
        .output()
        .expect("cpl build --checked runs");
    assert!(built.status.success(), "{built:?}");

    let run_bin = |cmd: &mut Command, input: &str| {
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    };

    // Valid inputs (including 0.0 and -0.0) succeed and agree.
    for input in ["4.0\n", "0.0\n", "-0.0\n"] {
        let mut interp_cmd = Command::new(env!("CARGO_BIN_EXE_cpl"));
        interp_cmd.arg("run").arg(&file);
        let interp = run_bin(&mut interp_cmd, input);
        let mut chk_cmd = Command::new(dir.join("s.checked"));
        let chk = run_bin(&mut chk_cmd, input);
        assert!(
            interp.status.success(),
            "interp failed on {input:?}: {interp:?}"
        );
        assert!(chk.status.success(), "checked failed on {input:?}: {chk:?}");
        assert_eq!(
            String::from_utf8_lossy(&interp.stdout),
            String::from_utf8_lossy(&chk.stdout),
            "stdout mismatch on {input:?}"
        );
    }

    // Negative input traps identically in interpreter and checked C++.
    let mut interp_cmd = Command::new(env!("CARGO_BIN_EXE_cpl"));
    interp_cmd.arg("run").arg(&file);
    let interp = run_bin(&mut interp_cmd, "-1.0\n");
    let mut chk_cmd = Command::new(dir.join("s.checked"));
    let chk = run_bin(&mut chk_cmd, "-1.0\n");
    assert_eq!(interp.status.code(), Some(3), "interp: {interp:?}");
    assert_eq!(chk.status.code(), Some(3), "checked: {chk:?}");
    let interp_err = String::from_utf8_lossy(&interp.stderr);
    let chk_err = String::from_utf8_lossy(&chk.stderr);
    assert!(
        interp_err.contains("sqrt(-1) of a negative number"),
        "interp: {interp_err}"
    );
    assert!(
        chk_err.contains("sqrt(-1) of a negative number"),
        "checked: {chk_err}"
    );

    // Release C++ (no -DLOCAL) stays raw and does not trap.
    let rel_bin = dir.join("s.release");
    let gpp = Command::new("g++")
        .args(["-std=c++17", "-O2", "-o"])
        .arg(&rel_bin)
        .arg(dir.join("s.cpp"))
        .output()
        .expect("g++ runs");
    assert!(gpp.status.success(), "g++: {gpp:?}");
    let mut rel_cmd = Command::new(&rel_bin);
    let rel = run_bin(&mut rel_cmd, "-1.0\n");
    assert!(rel.status.success(), "release should stay raw: {rel:?}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn cpl_run_still_checks_an_index_inside_unchecked() {
    let dir = std::env::temp_dir().join(format!("cpl-cli-unchecked-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("u.cp");
    std::fs::write(
        &path,
        "var xs = [1, 2, 3]\nunchecked:\n    xs[5] = 1\nprint(xs)\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("cpl runs");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(3), "{stderr}");
    assert!(
        stderr.contains("index 5 is outside a list of length 3"),
        "{stderr}"
    );
}

#[test]
fn cpl_run_still_checks_a_domain_position_inside_unchecked() {
    let dir = std::env::temp_dir().join(format!("cpl-cli-unchecked-pos-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("u.cp");
    std::fs::write(
        &path,
        "domain D = 1..=10\nvar a: [D]Int = 0\nunchecked:\n    a[D.nth(50)] = 1\nprint(a)\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("cpl runs");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(3), "{stderr}");
    assert!(stderr.contains("`D` has no position 50"), "{stderr}");
}

/// What `cpl check` prints for `source`.
fn check_errors(test: &str, source: &str) -> String {
    let path = scratch(test, "a.cp", source.as_bytes());
    let out = cpl(&["check".as_ref(), path.as_os_str()], b"");
    assert_eq!(out.status.code(), Some(1));
    String::from_utf8(out.stderr).unwrap()
}

#[test]
fn an_error_inside_library_code_is_shown_at_the_call_that_asked_for_it() {
    let source =
        "domain S = 0..3\nlet s: [S]Char = read()\nlet h = RollingHash.new(s)\nprint(h.get(3))\n";
    let stderr = check_errors("library-span", source);
    assert!(stderr.contains("a.cp:4:7"), "{stderr}");
    assert!(!stderr.contains("a.cp:1:1"), "{stderr}");
}

#[test]
fn the_users_own_error_hides_the_ones_it_causes_in_library_code() {
    let source = "domain S = 0..3\nlet s: [S]Char = read()\nlet h = RollingHash.new(s)\nprint(h.get(S.first!!..))\n";
    let stderr = check_errors("library-noise", source);
    assert!(
        stderr.contains("`!` takes the value out of an optional"),
        "{stderr}"
    );
    assert!(!stderr.contains("has no member"), "{stderr}");
}

#[test]
fn a_parameter_type_sees_only_the_parameters_before_it() {
    let stderr = check_errors(
        "param-size-later",
        "fn valid(seq: [0..n]Int, n: Int) -> Int: n\nprint(valid([1], 1))\n",
    );
    assert!(stderr.contains("no `n` in scope"), "{stderr}");
}

#[test]
fn an_arm_leaves_only_where_the_match_is_the_whole_value() {
    let source = "for i in 0..3:\n    let x = 1 + match i:\n        1 => continue\n        v => v\n    print(x)\n";
    let stderr = check_errors("leaving-arm-inside", source);
    assert!(
        stderr.contains("`continue` out of a block that is used as a value"),
        "{stderr}"
    );
}

#[test]
fn a_fn_inside_a_fn_says_which_arguments_are_mut() {
    let head = "fn main_like():\n    var xs: List<Int> = []\n    fn add(mut ys: List<Int>, x: Int):\n        ys.push(x)\n";
    let stderr = check_errors(
        "nested-mut-missing",
        &format!("{head}    add(xs, 1)\nmain_like()\n"),
    );
    assert!(
        stderr.contains("the argument must be passed as `mut xs`"),
        "{stderr}"
    );
    let stderr = check_errors(
        "nested-mut-extra",
        &format!("{head}    var k = 1\n    add(mut xs, mut k)\nmain_like()\n"),
    );
    assert!(
        stderr.contains("`add` doesn't change `x`, so `mut` doesn't belong here"),
        "{stderr}"
    );
}

#[test]
fn an_arc_without_a_record_says_how_to_read_it() {
    let pairs = "use std.graph.{undirected_of_pairs}\ndomain V = 0..3\nlet adj = undirected_of_pairs([(V(0), V(1))])\nfor a in adj[V(0)]: print(a.e)\n";
    let stderr = check_errors("arc-e", pairs);
    assert!(
        stderr.contains("a graph built from a list of pairs numbers no edges"),
        "{stderr}"
    );
    let weighted =
        "domain V = 0..3\nlet adj: [V]List<(V, Int)> = []\nfor b in adj[V(0)]: print(b.e)\n";
    let stderr = check_errors("arc-tuple", weighted);
    assert!(
        stderr.contains("the parts of a tuple have numbers, not names: `.0` to `.1`"),
        "{stderr}"
    );
}

#[test]
#[ignore = "spec/conformance/diagnostics/sum-of-str.cp"]
fn what_cpl_run_cannot_do_is_an_error_in_cpl_check_that_says_what_to_write() {
    let indices = check_errors(
        "list_indices",
        "let xs = [5, 6]\nfor i in xs.indices: print(i)\n",
    );
    assert!(indices.contains("List<Int> has no `indices`"), "{indices}");

    let sort = check_errors(
        "sort_a_range",
        "domain E = 0..3\nlet w: [E]Int = [E] e => 3 - e.ord\nvar order = E.all\nsort(mut order, by: e => w[e])\nprint(order)\n",
    );
    assert!(
        sort.contains("`sort` orders a list or an array, not a `Range<E>`"),
        "{sort}"
    );
    assert!(sort.contains("`sorted(r, by: ..)`"), "{sort}");

    let sum = check_errors("sum_of_text", "let xs = [\"a\", \"b\"]\nprint(sum(xs))\n");
    assert!(sum.contains("`sum` adds numbers, not `Str`"), "{sum}");
    assert!(sum.contains("xs.join(\"\")"), "{sum}");
}

#[test]
fn reading_an_optional_says_how_none_is_written() {
    let bare = check_errors("read_optional", "let a: ?Int = read()\nprint(a ?? 0)\n");
    assert!(
        bare.contains("`read()` has no input format for a `?Int`"),
        "{bare}"
    );
    assert!(bare.contains("`read(none: 0)`"), "{bare}");

    let stray = check_errors("read_none_of_int", "let b: Int = read(none: 0)\nprint(b)\n");
    assert!(stray.contains("`none:` is for reading a `?T`"), "{stray}");
}

#[test]
fn a_part_of_a_row_past_its_end_is_not_called_a() {
    let source = b"var g: [0..2, 0..3]Int = 0\nlet hi: Int = read()\nprint(g[1, 2..hi])\n";
    let path = scratch("grid-part", "a.cp", source);
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"7\n");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(3), "{stderr}");
    assert!(
        stderr.contains("a part `lo..hi` of a row or column needs 0 <= lo <= hi <= its length"),
        "{stderr}"
    );
    assert!(!stderr.contains("a["), "{stderr}");
}

/// `max=` on a whole grid, or on rows whose cells may be `none`, says
/// what to write instead of only that the type has no order.
#[test]
fn max_assign_on_a_grid_or_optional_rows_says_what_to_write() {
    let grid = check_errors(
        "max_grid",
        "var g: [0..2, 0..3]Int = 0\nvar h: [0..2, 0..3]Int = 1\ng max= h\nprint(g)\n",
    );
    assert!(
        grid.contains("`max=` works on one row at a time, not on a whole grid"),
        "{grid}"
    );
    assert!(grid.contains("`g[r, ..] max= h[r, ..]`"), "{grid}");

    let rows = check_errors(
        "min_opt_rows",
        "var a: [0..3]?Int = None\nvar b: [0..3]?Int = None\na min= b\nprint(a[1] ?? 0)\n",
    );
    assert!(rows.contains("a `?Int` cell may be `none`"), "{rows}");
    assert!(rows.contains("`a[i] min= b[i]`"), "{rows}");
}

/// The C++ of a recursion 10^6 calls deep runs where the process has the
/// 8 MB stack a judge's `main` usually gets: the program has a thread with
/// the profile's 1 GB, and nothing counts how deep the calls go.
#[test]
fn a_million_calls_deep_run_in_the_built_cpp_on_an_8_mb_stack() {
    if Command::new("g++").arg("--version").output().is_err() {
        return;
    }
    let source = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/stack/path_dfs.cp"
    ))
    .unwrap();
    let path = scratch("deep-dfs", "path_dfs.cp", &source);
    let built = cpl(&["build".as_ref(), path.as_os_str()], b"");
    assert!(built.status.success(), "{built:?}");
    let binary = path.with_extension("bin");
    let compiled = Command::new("g++")
        .args(["-std=c++17", "-O2", "-o"])
        .arg(&binary)
        .arg(path.with_extension("cpp"))
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let input = path.with_extension("in");
    std::fs::write(&input, "1000000\n").unwrap();
    let run = Command::new("sh")
        .arg("-c")
        .arg("ulimit -s 8192 && exec \"$0\" < \"$1\"")
        .arg(&binary)
        .arg(&input)
        .output()
        .unwrap();
    assert!(run.status.success(), "{run:?}");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1000000 999999\n");
}

#[test]
fn a_judge_without_threads_gets_a_plain_main_and_a_warning() {
    let path = scratch("judge-threads", "a.cp", b"// judge: olympiad\nprint(1)\n");
    let toml = "[judge.olympiad]\nextends = \"generic\"\nthread_stack_mb = 0\n";
    std::fs::write(path.with_file_name("cp.toml"), toml).unwrap();
    let out = cpl(&["build".as_ref(), path.as_os_str()], b"");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{out:?}");
    assert!(
        stderr.contains("warning: judge `olympiad` allows no threads"),
        "{stderr}"
    );
    let cpp = std::fs::read_to_string(path.with_extension("cpp")).unwrap();
    assert!(
        !cpp.contains("pthread") && cpp.contains("int main() {"),
        "{cpp}"
    );
    let out = cpl(
        &[
            "build".as_ref(),
            path.as_os_str(),
            "--judge".as_ref(),
            "generic".as_ref(),
        ],
        b"",
    );
    assert!(out.status.success() && out.stderr.is_empty(), "{out:?}");
    let cpp = std::fs::read_to_string(path.with_extension("cpp")).unwrap();
    assert!(
        cpp.contains("pthread_attr_setstacksize") && !cpp.contains("CreateThread"),
        "{cpp}"
    );
}

#[test]
#[ignore = "cpl-ir: Deque from prelude not in scope during lowering of user library"]
fn user_library_sees_prelude_in_check_run_and_build() {
    let main = scratch(
        "user-lib-prelude",
        "main.cp",
        b"use mylib.dq.{firsts}\nprint(firsts(3))\n",
    );
    let root = main.parent().unwrap();
    let lib_dir = root.join("mylib");
    std::fs::create_dir_all(&lib_dir).unwrap();
    std::fs::write(root.join("cp.toml"), "[libs]\nmylib = \"mylib\"\n").unwrap();
    std::fs::write(lib_dir.join("lib.toml"), "name = \"mylib\"\n").unwrap();
    std::fs::write(
        lib_dir.join("dq.cp"),
        "pub fn firsts(n: Int) -> Int:\n    var d = Deque<Int>()\n    for i in 0..n:\n        d.push(i)\n    d.pop_front()!\n",
    )
    .unwrap();

    let checked = cpl(&["check".as_ref(), main.as_os_str()], b"");
    assert!(checked.status.success(), "{checked:?}");

    let ran = cpl(&["run".as_ref(), main.as_os_str()], b"");
    assert!(ran.status.success(), "{ran:?}");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "0\n");

    let built = cpl(&["build".as_ref(), main.as_os_str()], b"");
    assert!(built.status.success(), "{built:?}");
    let cpp = main.with_extension("cpp");
    let bin = root.join("main_bin");
    let compiled = Command::new("g++")
        .args(["-std=c++17", "-O2", "-o"])
        .arg(&bin)
        .arg(&cpp)
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let native = Command::new(&bin).output().unwrap();
    assert!(native.status.success(), "{native:?}");
    assert_eq!(String::from_utf8_lossy(&native.stdout), "0\n");
}

/// cf2178h never reads the `back` witness of `fixed_cost_paths`, a grid
/// over 4e6 states: the field, its allocation and its stores are left out,
/// and `cpl explain` says so (decision 178).
#[test]
fn a_witness_field_nothing_reads_is_left_out_of_the_cpp() {
    let source = "\
use std.graph.{fixed_cost_paths}
domain S = 0..5
let dist = fixed_cost_paths(
    from: [S(0)],
    moves: [1, 2],
    cost: m => if m == 1: 1 else: 3,
    step: (m, s) => if s.ord + m < 5: Some(S(s.ord + m)) else: None,
)
print(dist[S(4)]!)
";
    let port = scratch("witness-field-omitted", "a.cp", source.as_bytes());
    let explained = cpl(&["explain".as_ref(), port.as_os_str()], b"");
    assert!(explained.status.success(), "{explained:?}");
    let explained = String::from_utf8_lossy(&explained.stdout);
    assert!(explained.contains(".back -> never read"), "{explained}");
    let built = cpl(
        &[
            "build".as_ref(),
            port.as_os_str(),
            "-o".as_ref(),
            "-".as_ref(),
        ],
        b"",
    );
    assert!(built.status.success(), "{built:?}");
    let cpp = String::from_utf8_lossy(&built.stdout);
    let paths = &cpp[cpp.find("struct Paths").expect("Paths")..];
    let paths = &paths[..paths.find("};").unwrap()];
    assert!(
        paths.contains(" dist;") && !paths.contains("back"),
        "{paths}"
    );
}

#[test]
fn lsp_completes_and_hovers_the_v03_stdlib_with_no_edition_set() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["lsp", "--stdio"])
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cpl lsp starts");
    let text = "use std.heap.{MinMaxHeap}\\nvar h = MinMaxHeap<Int>()\\nh.push(3)\\nlet a = Lazy\\nlet b = bsearch\\n";
    let request = |id: u32, method: &str, line: u32, character: u32| {
        format!(
            r#"{{"jsonrpc":"2.0","id":{id},"method":"textDocument/{method}","params":{{"textDocument":{{"uri":"file:///a.cp"}},"position":{{"line":{line},"character":{character}}}}}}}"#
        )
    };
    let session = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.to_string(),
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file:///a.cp","languageId":"cp","version":1,"text":"{text}"}}}}}}"#
        ),
        request(2, "hover", 1, 10),
        request(3, "hover", 2, 3),
        request(4, "completion", 3, 12),
        request(5, "completion", 4, 15),
        r#"{"jsonrpc":"2.0","id":6,"method":"shutdown","params":null}"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"exit","params":null}"#.to_string(),
    ];
    let mut stdin = child.stdin.take().unwrap();
    for message in &session {
        stdin.write_all(frame(message).as_bytes()).unwrap();
    }
    drop(stdin);
    let out = child.wait_with_output().expect("cpl lsp exits");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let mut reader = std::io::BufReader::new(stdout.as_bytes());
    let mut replies = std::collections::BTreeMap::new();
    while let Some(body) = cpl_lsp::read_message(&mut reader).unwrap() {
        let message = cpl_json::parse(&body).unwrap();
        if let Some(id) = message.at(&["id"]).as_u64() {
            replies.insert(id, message.at(&["result"]).clone());
        }
    }
    let hover = |id: u64| {
        replies[&id]
            .at(&["contents", "value"])
            .as_str()
            .unwrap_or("")
            .to_string()
    };
    let labels = |id: u64| -> Vec<String> {
        let items = replies[&id]
            .get("items")
            .unwrap_or(&replies[&id])
            .as_array()
            .unwrap_or(&[]);
        items
            .iter()
            .filter_map(|i| i.at(&["label"]).as_str().map(str::to_string))
            .collect()
    };
    assert!(hover(2).contains("struct MinMaxHeap<T>"), "{}", hover(2));
    assert!(hover(3).contains("MinMaxHeap.of([2, 5])"), "{}", hover(3));
    let lazy = labels(4);
    assert!(lazy.iter().any(|l| l == "LazyTreap"), "{lazy:?}");
    assert!(
        !lazy
            .iter()
            .any(|l| l == "LazySegTree" || l == "DualSegTree"),
        "{lazy:?}"
    );
    let search = labels(5);
    assert!(
        !search
            .iter()
            .any(|l| l == "bsearch_first" || l == "bsearch_last"),
        "{search:?}"
    );
}

/// An error inside std reached through a user library names each library
/// call on the way, innermost first, below the std line it fails on.
#[test]
fn an_error_inside_std_names_the_library_calls_that_led_there() {
    let main = scratch(
        "std-error-chain",
        "main.cp",
        b"use mylib.text.{suffix_count}\n\nlet n: 1..=10 = read()\ndomain Pos = 0..n\nlet s: [Pos]Char = read()\nprint(suffix_count(s))\n",
    );
    let root = main.parent().unwrap();
    let lib_dir = root.join("mylib");
    std::fs::create_dir_all(&lib_dir).unwrap();
    std::fs::write(root.join("cp.toml"), "[libs]\nmylib = \"mylib\"\n").unwrap();
    std::fs::write(lib_dir.join("lib.toml"), "name = \"mylib\"\n").unwrap();
    std::fs::write(
        lib_dir.join("text.cp"),
        "use std.strings.{SuffixArray}\n\npub fn suffix_count<D>(s: [D]Char) -> Int:\n    let sa = SuffixArray(s)\n    return sa.len\n",
    )
    .unwrap();
    let checked = cpl(&["check".as_ref(), main.as_os_str()], b"");
    let err = String::from_utf8_lossy(&checked.stderr);
    let inside = err.find("inside `SuffixArray`").expect(&err);
    let via = err
        .find("mylib/text.cp:4: called from `suffix_count`")
        .expect(&err);
    assert!(inside < via && err.contains("main.cp:6:7"), "{err}");
}

#[test]
fn a_reader_that_stops_early_is_not_a_cpl_bug() {
    let dir = std::env::temp_dir().join(format!("cpl-closed-reader-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let lines = dir.join("lines.cp");
    std::fs::write(&lines, "for i in 0..1000000:\n    print(i)\n").unwrap();
    let bad = dir.join("bad.cp");
    std::fs::write(&bad, "let x: Int = \"no\"\nprint(x)\n").unwrap();
    let lines = lines.to_str().unwrap();
    let bad = bad.to_str().unwrap();
    let closed = || {
        let (reader, writer) = std::io::pipe().unwrap();
        drop(reader);
        writer
    };
    // `cpl run` of a program printing a million lines, as in `cpl run p | head`.
    for args in [
        &["--version"][..],
        &["--help"],
        &["run", lines],
        &["explain", lines],
        &["build", lines],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(args)
            .stdout(closed())
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!stderr.contains("cpl bug"), "{args:?}: {stderr}");
        assert!(out.status.success(), "{args:?}: {:?} {stderr}", out.status);
    }
    // Errors to a closed stderr, as in `cpl check bad.cp 2>&1 | head`.
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["check", bad])
        .stderr(closed())
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(101), "{:?}", out.status);
    assert!(out.status.code().is_some(), "{:?}", out.status);
    let _ = std::fs::remove_dir_all(&dir);
}
