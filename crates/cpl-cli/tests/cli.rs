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
fn build_refuses_cpp_over_64_kb() {
    let mut source = String::new();
    for i in 0..4000 {
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
