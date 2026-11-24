//! `cpl fmt`, `cpl lint` and `cpl fix`.

use super::*;

#[test]
fn fix_applies_every_fix_and_reports_what_is_left() {
    helps("fix");
    misused("fix", &["fix"], "`fix` needs a file");
    let dir = scratch("fix");
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixes");
    std::fs::copy(cases.join("c_operators.cp"), dir.join("a.cp")).unwrap();
    let out = cpl(&dir, &["fix", "a.cp"]);
    let fixed = std::fs::read_to_string(cases.join("c_operators.fixed")).unwrap();
    assert_eq!(std::fs::read_to_string(dir.join("a.cp")).unwrap(), fixed);
    let left = fixed.contains("not in [1, 2, 3]");
    let code = if left { EXIT_ERRORS } else { 0 };
    assert_eq!(
        out.status.code(),
        Some(code.into()),
        "{}",
        text(&out.stdout)
    );
    let clean = cpl(&dir, &["fix", "a.cp"]);
    assert_eq!(
        std::fs::read_to_string(dir.join("a.cp")).unwrap(),
        fixed,
        "fix is idempotent"
    );
    assert_eq!(clean.status.code(), out.status.code());
}

#[test]
fn fix_json_reports_what_it_applied_and_what_is_left() {
    misused("fix", &["fix", "--json"], "`fix` needs a file");
    let dir = scratch("fix-json");
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixes");
    std::fs::copy(cases.join("c_operators.cp"), dir.join("a.cp")).unwrap();
    std::fs::write(dir.join("bad.cp"), "let x: Int = \"a\"\n").unwrap();
    let out = cpl(&dir, &["fix", "a.cp", "bad.cp", "--json"]);
    assert_eq!(out.status.code(), Some(EXIT_ERRORS.into()));
    let json = cpl_json::parse(&text(&out.stdout)).expect("cpl fix --json is JSON");
    let files = json.at(&["files"]).as_array().unwrap();
    assert_eq!(files.len(), 2);
    let fixed = std::fs::read_to_string(cases.join("c_operators.fixed")).unwrap();
    assert_eq!(std::fs::read_to_string(dir.join("a.cp")).unwrap(), fixed);
    let (a, bad) = (&files[0], &files[1]);
    assert_eq!(a.at(&["path"]).as_str(), Some("a.cp"));
    assert!(a.at(&["applied"]).as_i64().unwrap() > 0, "{a:?}");
    assert_eq!(bad.at(&["applied"]).as_i64(), Some(0));
    assert_eq!(bad.at(&["ok"]).as_bool(), Some(false));
    let left = bad.at(&["diagnostics"]).as_array().unwrap();
    assert_eq!(left[0].at(&["code"]).as_str(), Some("E0302"), "{bad:?}");
    let check = cpl(&dir, &["check", "bad.cp", "--json"]);
    let check = cpl_json::parse(&text(&check.stdout)).unwrap();
    assert_eq!(
        bad.at(&["diagnostics"]),
        check.at(&["files"]).as_array().unwrap()[0].at(&["diagnostics"]),
        "fix --json reports what is left as check --json does"
    );
}

#[test]
fn fmt_check_reports_without_writing_and_fmt_rewrites() {
    let dir = scratch("fmt");
    std::fs::write(dir.join("a.cp"), "fn inc(x: Int) -> Int:\n    return x+1\n").unwrap();
    let check = cpl(&dir, &["fmt", "--check", "a.cp"]);
    assert_eq!(check.status.code(), Some(1), "{}", text(&check.stderr));
    assert!(text(&check.stdout).contains("would reformat a.cp"));
    let written = || std::fs::read_to_string(dir.join("a.cp")).unwrap();
    assert!(written().contains("x+1"), "--check writes nothing");
    let out = cpl(&dir, &["fmt", "a.cp"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(written().contains("return x + 1"), "{}", written());
    assert_eq!(
        cpl(&dir, &["fmt", "--check", "a.cp"]).status.code(),
        Some(0)
    );
}

#[test]
fn fmt_with_no_path_formats_stdin_and_rewrites_no_file() {
    let dir = scratch("fmt-no-path");
    let messy = "fn inc(x: Int) -> Int:\n    return x+1\n";
    std::fs::write(dir.join("a.cp"), messy).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("fmt")
        .current_dir(&dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("cpl runs");
    std::io::Write::write_all(&mut child.stdin.take().unwrap(), b"let x=1\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(text(&out.stdout), "let x = 1\n");
    assert_eq!(std::fs::read_to_string(dir.join("a.cp")).unwrap(), messy);
    let check = cpl(&dir, &["fmt", "--check"]);
    assert_eq!(
        check.status.code(),
        Some(1),
        "--check with no path checks the directory"
    );
    assert_eq!(std::fs::read_to_string(dir.join("a.cp")).unwrap(), messy);
}

#[test]
fn fmt_json_says_which_files_changed_and_why_one_was_not_formatted() {
    let dir = scratch("fmt-json");
    std::fs::write(dir.join("a.cp"), "fn inc(x: Int) -> Int:\n    return x+1\n").unwrap();
    std::fs::write(dir.join("b.cp"), "fn one() -> Int:\n    return 1\n").unwrap();
    std::fs::write(dir.join("c.cp"), "fn bad() -> Int:\n    return \"open\n").unwrap();
    let file =
        |report: &cpl_json::Value, i: usize| report.at(&["files"]).as_array().unwrap()[i].clone();

    let check = cpl(&dir, &["fmt", "--check", "--json", "a.cp", "b.cp", "c.cp"]);
    assert_eq!(check.status.code(), Some(1), "{}", text(&check.stderr));
    assert_eq!(text(&check.stderr), "", "--json reports in the object");
    let report = cpl_json::parse(&text(&check.stdout)).unwrap();
    let (a, b, c) = (file(&report, 0), file(&report, 1), file(&report, 2));
    assert_eq!(a.at(&["path"]).as_str(), Some("a.cp"));
    assert_eq!(a.at(&["changed"]).as_bool(), Some(true));
    assert_eq!(a.at(&["ok"]).as_bool(), Some(true));
    assert!(a.at(&["error"]).is_null());
    assert_eq!(b.at(&["changed"]).as_bool(), Some(false));
    assert_eq!(c.at(&["ok"]).as_bool(), Some(false));
    assert!(c
        .at(&["error"])
        .as_str()
        .unwrap()
        .starts_with("not formatted"));
    let diag = &c.at(&["diagnostics"]).as_array().unwrap()[0];
    assert!(diag.at(&["start", "line"]).as_i64().is_some(), "{diag:?}");
    let written = std::fs::read_to_string(dir.join("a.cp")).unwrap();
    assert!(written.contains("x+1"), "--check writes nothing");

    let again = cpl(&dir, &["fmt", "--check", "--json", "a.cp", "b.cp", "c.cp"]);
    assert_eq!(text(&again.stdout), text(&check.stdout), "deterministic");
    let format = cpl(
        &dir,
        &["fmt", "--check", "--format", "json", "a.cp", "b.cp", "c.cp"],
    );
    assert_eq!(
        text(&format.stdout),
        text(&check.stdout),
        "--format json is --json"
    );

    let out = cpl(&dir, &["fmt", "--json", "a.cp", "b.cp"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let report = cpl_json::parse(&text(&out.stdout)).unwrap();
    assert_eq!(file(&report, 0).at(&["changed"]).as_bool(), Some(true));
    let written = std::fs::read_to_string(dir.join("a.cp")).unwrap();
    assert!(written.contains("return x + 1"), "{written}");

    let stdin = cpl(&dir, &["fmt", "--stdin", "--json"]);
    assert_eq!(stdin.status.code(), Some(2), "{}", text(&stdin.stderr));
}

#[test]
fn lint_warns_with_its_code_and_exits_0_on_warnings_alone() {
    let dir = scratch("lint");
    std::fs::write(
        dir.join("w.cp"),
        "fn f() -> Int:\n    var x = 1\n    return x\n",
    )
    .unwrap();
    let out = cpl(&dir, &["lint", "w.cp"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let report = text(&out.stdout);
    assert!(
        report.contains("warning[L0201]") && report.contains("w.cp:2:5"),
        "{report}"
    );
    let list = cpl(&dir, &["lint", "--list"]);
    assert_eq!(list.status.code(), Some(0));
    assert!(text(&list.stdout).contains("L0201  var_never_reassigned"));
    std::fs::write(
        dir.join("cp.toml"),
        "[lints]\nvar_never_reassigned = \"deny\"\n",
    )
    .unwrap();
    let denied = cpl(&dir, &["lint", "w.cp"]);
    assert_eq!(denied.status.code(), Some(EXIT_ERRORS.into()));
    let report = text(&denied.stdout) + &text(&denied.stderr);
    assert!(report.contains("error[L0201]"), "{report}");
}

#[test]
fn lint_json_prints_the_files_cpl_check_json_prints() {
    let dir = scratch("lint-json");
    std::fs::write(
        dir.join("w.cp"),
        "fn f() -> Int:\n    var x = 1\n    return x\n",
    )
    .unwrap();
    std::fs::write(dir.join("t.cp"), "let x: Int = \"a\"\nprint(x)\n").unwrap();
    let out = cpl(&dir, &["lint", "--json", "w.cp", "t.cp"]);
    assert_eq!(
        out.status.code(),
        Some(EXIT_ERRORS.into()),
        "{}",
        text(&out.stderr)
    );
    let report = text(&out.stdout);
    let format = cpl(&dir, &["lint", "--format", "json", "w.cp", "t.cp"]);
    assert_eq!(text(&format.stdout), report, "{}", text(&format.stderr));
    assert!(
        report.starts_with("{\"files\":[{\"diagnostics\":[],\"ok\":false,\"path\":\"t.cp\"},"),
        "{report}"
    );
    assert!(
        report.contains("\"code\":\"L0201\"")
            && report.ends_with("\"ok\":true,\"path\":\"w.cp\"}]}\n"),
        "{report}"
    );
}
