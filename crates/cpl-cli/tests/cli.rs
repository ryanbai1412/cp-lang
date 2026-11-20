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
