//! End-to-end tests for the commands no other test ran through the binary:
//! each one's `--help`, a success, and a usage error with its own usage.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use cpl_cli::{EXIT_ERRORS, EXIT_FAILED, EXIT_USAGE};

fn cpl(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("cpl runs")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("golden")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `--help` prints the command's own usage on stdout and exits 0.
/// Whether `text` holds `command`'s usage line, `usage: cpl <command>` alone
/// or with arguments.
fn shows_usage(text: &str, command: &str) -> bool {
    text.lines().any(|line| {
        line.split_whitespace()
            .take(3)
            .eq(["usage:", "cpl", command])
    })
}

fn helps(command: &str) {
    let out = cpl(Path::new("."), &[command, "--help"]);
    assert_eq!(out.status.code(), Some(0), "{command} --help");
    let usage = text(&out.stdout);
    assert!(
        shows_usage(usage.lines().next().unwrap_or(""), command),
        "{usage}"
    );
    assert!(out.stderr.is_empty());
}

/// A usage error names the problem, then shows the command's usage, not
/// the list of every command.
fn misused(command: &str, args: &[&str], message: &str) {
    let out = cpl(Path::new("."), args);
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()), "{args:?}");
    let stderr = text(&out.stderr);
    assert!(stderr.contains(message), "{stderr}");
    assert!(shows_usage(&stderr, command), "{stderr}");
    assert!(!stderr.contains("commands:"), "{stderr}");
}

#[test]
fn new_creates_the_solution_and_its_tests_folder() {
    helps("new");
    misused("new", &["new"], "`new` needs a name");
    let dir = scratch("new");
    let out = cpl(&dir, &["new", "sum", "--constraints", "n: 1..=10"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("created sum.cp"));
    let source = std::fs::read_to_string(dir.join("sum.cp")).unwrap();
    assert!(source.contains("1..=10"), "{source}");
    assert!(dir.join("sum.tests").is_dir());
    let check = cpl(&dir, &["check", "sum.cp"]);
    assert_eq!(check.status.code(), Some(0), "{}", text(&check.stdout));
    std::fs::write(dir.join("sum.cp"), "print(1)\n").unwrap();
    let again = cpl(&dir, &["new", "sum"]);
    assert_eq!(again.status.code(), Some(EXIT_ERRORS.into()));
    assert!(text(&again.stderr).contains("won't overwrite"));
    let kept = std::fs::read_to_string(dir.join("sum.cp")).unwrap();
    assert_eq!(kept, "print(1)\n");
}

#[test]
fn doc_writes_each_comment_as_json_or_a_site() {
    helps("doc");
    misused("doc", &["doc", "--bogus"], "unknown option `--bogus`");
    let dir = scratch("doc");
    std::fs::write(
        dir.join("inc.cp"),
        "/// Adds one.\nfn inc(x: Int) -> Int:\n    return x + 1\n",
    )
    .unwrap();
    let json = cpl(&dir, &["doc", "--format", "json", "inc.cp"]);
    assert_eq!(json.status.code(), Some(0), "{}", text(&json.stderr));
    let json = text(&json.stdout);
    assert!(
        json.contains(r#""signature":"fn inc(x: Int) -> Int","doc":"Adds one.""#),
        "{json}"
    );
    let site = cpl(&dir, &["doc", "--out", "site", "inc.cp"]);
    assert_eq!(site.status.code(), Some(0), "{}", text(&site.stderr));
    assert!(dir.join("site/index.html").is_file());
    assert!(dir.join("site/inc.html").is_file());
}

#[test]
fn judges_lists_the_profiles_and_marks_the_default() {
    helps("judges");
    misused(
        "judges",
        &["judges", "a.cp", "b.cp"],
        "`judges` takes at most a file",
    );
    let out = cpl(Path::new("."), &["judges"]);
    assert_eq!(out.status.code(), Some(0));
    let list = text(&out.stdout);
    assert!(list.contains("* generic    C++17 with g++"), "{list}");
}

#[test]
fn judge_runs_the_checker_on_an_output() {
    helps("judge");
    misused(
        "judge",
        &["judge"],
        "`judge` takes a file and the hidden test",
    );
    let dir = scratch("judge");
    std::fs::write(
        dir.join("echo.cp"),
        "let n: 1..=5 = read()\nprint(n)\n@checker\nfn check():\n    let x: Int = read_output()\n    assert x == n\n",
    )
    .unwrap();
    std::fs::write(dir.join("in.txt"), "3\n").unwrap();
    std::fs::write(dir.join("right.txt"), "3\n").unwrap();
    std::fs::write(dir.join("wrong.txt"), "4\n").unwrap();
    let accepted = cpl(&dir, &["judge", "echo.cp", "in.txt", "right.txt"]);
    assert_eq!(
        accepted.status.code(),
        Some(0),
        "{}",
        text(&accepted.stderr)
    );
    let rejected = cpl(&dir, &["judge", "echo.cp", "in.txt", "wrong.txt"]);
    assert_eq!(rejected.status.code(), Some(EXIT_FAILED.into()));
    assert!(text(&rejected.stderr).contains("assertion failed: x == n"));
    let verdict = |args: &[&str]| {
        let out = cpl(&dir, args);
        let json = cpl_json::parse(&text(&out.stdout)).expect("cpl judge --json is JSON");
        (out.status.code(), json)
    };
    let (code, json) = verdict(&["judge", "echo.cp", "in.txt", "right.txt", "--json"]);
    assert_eq!(code, Some(0));
    assert_eq!(json.at(&["accepted"]).as_bool(), Some(true));
    assert_eq!(*json.at(&["reason"]), cpl_json::Value::Null);
    let (code, json) = verdict(&["judge", "echo.cp", "in.txt", "wrong.txt", "--json"]);
    assert_eq!(code, Some(EXIT_FAILED.into()));
    assert_eq!(json.at(&["accepted"]).as_bool(), Some(false));
    let reason = json.at(&["reason"]).as_str().unwrap();
    assert!(reason.contains("assertion failed: x == n"), "{reason}");
    assert_eq!(
        json.at(&["diagnostic", "start", "line"]).as_i64(),
        Some(6),
        "{json:?}"
    );
    let spelled = verdict(&[
        "judge",
        "echo.cp",
        "in.txt",
        "wrong.txt",
        "--format",
        "json",
    ]);
    assert_eq!(spelled, (code, json));
}

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

/// Every command `cpl --help` lists: its own `--help`, and an option it does
/// not have is a usage error that shows its usage, not a missing file.
#[test]
fn every_command_has_help_and_rejects_an_unknown_option() {
    let help = text(&cpl(Path::new("."), &["--help"]).stdout);
    let commands: Vec<&str> = help
        .lines()
        .skip_while(|line| *line != "commands:")
        .skip(1)
        .take_while(|line| line.starts_with("  "))
        .filter(|line| !line.starts_with("   "))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert_eq!(commands.len(), 23, "{help}");
    for command in commands {
        helps(command);
        let out = cpl(Path::new("."), &[command, "--no-such-option"]);
        let stderr = text(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(EXIT_USAGE.into()),
            "{command}: {stderr}"
        );
        assert!(stderr.contains("--no-such-option"), "{command}: {stderr}");
        assert!(shows_usage(&stderr, command), "{command}: {stderr}");
        assert!(!stderr.contains("can't read"), "{command}: {stderr}");
    }
}

#[test]
fn help_json_lists_every_command_with_its_usage_summary_and_help() {
    let out = cpl(Path::new("."), &["--help", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let json = cpl_json::parse(&text(&out.stdout)).expect("cpl --help --json is JSON");
    let commands = json.at(&["commands"]).as_array().unwrap();
    assert_eq!(commands.len(), 23);
    let listed = text(&cpl(Path::new("."), &["--help"]).stdout);
    for command in commands {
        let name = command.at(&["name"]).as_str().unwrap();
        let usage = command.at(&["usage"]).as_str().unwrap();
        assert!(listed.contains(&format!("\n  {usage}")), "{name}: {usage}");
        assert!(
            !command.at(&["summary"]).as_str().unwrap().is_empty(),
            "{name}"
        );
        assert!(
            shows_usage(command.at(&["help"]).as_str().unwrap_or(""), name),
            "{name}"
        );
    }
}

#[test]
fn check_json_reports_each_file_and_its_diagnostics() {
    let dir = scratch("check-json");
    std::fs::write(
        dir.join("ok.cp"),
        "fn inc(x: Int) -> Int:\n    return x + 1\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("bad.cp"),
        "fn inc(x: Int) -> Int:\n    return y + 1\n",
    )
    .unwrap();
    let out = cpl(&dir, &["check", "ok.cp", "bad.cp", "--json"]);
    assert_eq!(
        out.status.code(),
        Some(EXIT_ERRORS.into()),
        "{}",
        text(&out.stderr)
    );
    assert!(out.stderr.is_empty(), "{}", text(&out.stderr));
    let json = cpl_json::parse(&text(&out.stdout)).expect("cpl check --json is JSON");
    let files = json.at(&["files"]).as_array().unwrap();
    assert_eq!(files[0].at(&["ok"]).as_bool(), Some(true));
    assert_eq!(
        files[0].at(&["diagnostics"]).as_array().map(<[_]>::len),
        Some(0)
    );
    assert_eq!(files[1].at(&["path"]).as_str(), Some("bad.cp"));
    assert_eq!(files[1].at(&["ok"]).as_bool(), Some(false));
    let first = &files[1].at(&["diagnostics"]).as_array().unwrap()[0];
    assert!(
        cpl_diag::lookup(first.at(&["code"]).as_str().unwrap()).is_some(),
        "{first:?}"
    );
    assert_eq!(first.at(&["severity"]).as_str(), Some("error"));
    assert_eq!(first.at(&["start", "line"]).as_i64(), Some(2));
    assert_eq!(first.at(&["start", "col"]).as_i64(), Some(12));
    assert_eq!(
        cpl(&dir, &["check", "--json"]).status.code(),
        Some(EXIT_USAGE.into())
    );
}

#[test]
fn judges_json_lists_every_profile_and_marks_the_chosen_one() {
    let out = cpl(Path::new("."), &["judges", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let json = cpl_json::parse(&text(&out.stdout)).expect("cpl judges --json is JSON");
    let judges = json.at(&["judges"]).as_array().unwrap();
    let listed = text(&cpl(Path::new("."), &["judges"]).stdout);
    assert_eq!(
        judges.len(),
        listed.lines().filter(|l| !l.starts_with("    ")).count()
    );
    let chosen: Vec<_> = judges
        .iter()
        .filter(|j| j.at(&["chosen"]).as_bool() == Some(true))
        .collect();
    assert_eq!(chosen.len(), 1, "{judges:?}");
    for judge in judges {
        assert!(
            !judge.at(&["command"]).as_str().unwrap().is_empty(),
            "{judge:?}"
        );
        assert!(
            judge.at(&["source_limit_kb"]).as_i64().unwrap() > 0,
            "{judge:?}"
        );
    }
}

#[test]
fn json_and_format_json_are_one_flag_on_every_command_with_json() {
    let dir = scratch("json-spelling");
    std::fs::write(
        dir.join("inc.cp"),
        "/// Adds one.\nfn inc(x: Int) -> Int:\n    return x + 1\n",
    )
    .unwrap();
    for (json, format) in [
        (
            &["check", "inc.cp", "--json"][..],
            &["check", "inc.cp", "--format", "json"][..],
        ),
        (&["judges", "--json"], &["judges", "--format", "json"]),
        (
            &["fix", "inc.cp", "--json"][..],
            &["fix", "inc.cp", "--format", "json"][..],
        ),
        (
            &["doc", "--json", "inc.cp"],
            &["doc", "--format", "json", "inc.cp"],
        ),
    ] {
        let (json, format) = (cpl(&dir, json), cpl(&dir, format));
        assert_eq!(json.status.code(), Some(0), "{}", text(&json.stderr));
        assert!(
            cpl_json::parse(&text(&json.stdout)).is_ok(),
            "{}",
            text(&json.stdout)
        );
        assert_eq!(text(&json.stdout), text(&format.stdout));
    }
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

#[test]
fn explain_shows_a_code_and_rejects_an_unknown_one() {
    misused("explain", &["explain"], "`explain` takes a code or a file");
    misused(
        "explain",
        &["explain", "E7777"],
        "`E7777` is not a diagnostic or lint code",
    );
    let out = cpl(Path::new("."), &["explain", "E0302"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).starts_with("E0302: "),
        "{}",
        text(&out.stdout)
    );
    let unknown = cpl(Path::new("."), &["explain", "E7777"]);
    assert_eq!(unknown.status.code(), Some(EXIT_USAGE.into()));
    assert!(text(&unknown.stderr).contains("`E7777` is not a diagnostic or lint code"));
    let json = cpl(Path::new("."), &["explain", "--json"]);
    let json = cpl_json::parse(&text(&json.stdout)).expect("cpl explain --json is JSON");
    assert!(json
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e.at(&["code"]).as_str() == Some("E0302")));
}

/// Help fits an 80-column terminal. `stress` is the debugger domain's, which
/// has the request (plan/domains/debugger/worklog.md).
#[test]
fn every_help_fits_80_columns() {
    let json = cpl_json::parse(&text(&cpl(Path::new("."), &["--help", "--json"]).stdout)).unwrap();
    let mut helps = vec![(
        "cpl".to_string(),
        text(&cpl(Path::new("."), &["--help"]).stdout),
    )];
    for command in json.at(&["commands"]).as_array().unwrap() {
        let name = command.at(&["name"]).as_str().unwrap();
        if name != "stress" {
            helps.push((
                name.into(),
                command.at(&["help"]).as_str().unwrap_or("").into(),
            ));
        }
    }
    let wide: Vec<String> = helps
        .iter()
        .flat_map(|(name, help)| help.lines().map(move |line| (name, line)))
        .filter(|(_, line)| line.chars().count() > 80)
        .map(|(name, line)| format!("{name}: {line}"))
        .collect();
    assert!(wide.is_empty(), "{wide:#?}");
}

#[test]
fn watch_once_runs_one_round_with_its_exit_code() {
    helps("watch");
    misused("watch", &["watch"], "`watch` needs a problem");
    let dir = scratch("watch");
    let source = "let (a, b): (1..=10, 1..=10) = read()\nprint(a + b)\n";
    std::fs::write(dir.join("sum.cp"), source).unwrap();
    std::fs::create_dir_all(dir.join("sum.tests")).unwrap();
    std::fs::write(dir.join("sum.tests/1.in"), "1 2\n").unwrap();
    std::fs::write(dir.join("sum.tests/1.out"), "3\n").unwrap();
    let out = cpl(&dir, &["watch", "sum", "--once"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let report = text(&out.stdout) + &text(&out.stderr);
    assert!(report.contains("1/1 passed"), "{report}");
    std::fs::write(dir.join("sum.tests/1.out"), "4\n").unwrap();
    let out = cpl(&dir, &["watch", "sum", "--once"]);
    assert_eq!(out.status.code(), Some(1));
    let report = text(&out.stdout) + &text(&out.stderr);
    assert!(
        report.contains("WA") && report.contains("0/1 passed"),
        "{report}"
    );
}

#[test]
fn replay_names_what_it_needs() {
    helps("replay");
    misused(
        "replay",
        &["replay", "a.cp"],
        "`replay` needs a file, --trace and --step",
    );
    let dir = scratch("replay");
    let source = "fn sq(x: Int) -> Int:\n    return x * x\nprint(sq(3))\n";
    std::fs::write(dir.join("b.cp"), source).unwrap();
    let trace = cpl(&dir, &["run", "b.cp", "--trace", "--format", "json"]);
    assert_eq!(trace.status.code(), Some(0), "{}", text(&trace.stderr));
    std::fs::write(dir.join("t.json"), &trace.stdout).unwrap();
    let call = cpl(
        &dir,
        &["replay", "b.cp", "--trace", "t.json", "--step", "1"],
    );
    assert_eq!(call.status.code(), Some(0), "{}", text(&call.stderr));
    assert!(text(&call.stdout).contains("returns 9 (as before)"));
    let none = cpl(
        &dir,
        &["replay", "b.cp", "--trace", "t.json", "--step", "3"],
    );
    assert_eq!(none.status.code(), Some(EXIT_ERRORS.into()));
    assert!(text(&none.stderr).contains("no fn call is running at step 3"));
}

#[test]
fn a_program_with_an_error_or_a_failing_case_exits_1() {
    let dir = scratch("exits");
    let source = "let n: 1..=10 = read()\nif n > 5:\n    print(1)\nelse:\n    print(0)\n";
    std::fs::write(dir.join("a.cp"), source).unwrap();
    std::fs::write(dir.join("bad.cp"), "let x: Int = \"a\"\n").unwrap();
    std::fs::create_dir_all(dir.join("a.tests")).unwrap();
    std::fs::write(dir.join("a.tests/1.in"), "7\n").unwrap();
    std::fs::write(dir.join("a.tests/1.out"), "1\n").unwrap();
    std::fs::create_dir_all(dir.join("bad.tests")).unwrap();
    std::fs::write(dir.join("bad.tests/1.in"), "\n").unwrap();
    std::fs::write(dir.join("bad.tests/1.out"), "\n").unwrap();
    let passes = |args: &[&str], says: &str| {
        let out = cpl(&dir, args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?}: {}",
            text(&out.stderr)
        );
        assert!(
            text(&out.stdout).contains(says),
            "{args:?}: {}",
            text(&out.stdout)
        );
    };
    let fails = |args: &[&str], says: &str| {
        let out = cpl(&dir, args);
        assert_eq!(out.status.code(), Some(EXIT_ERRORS.into()), "{args:?}");
        let report = text(&out.stdout) + &text(&out.stderr);
        assert!(report.contains(says), "{args:?}: {report}");
    };
    passes(&["submit", "a.cp"], "ready to send: a.cpp");
    passes(&["cover", "a.cp"], "3 of 4 lines covered");
    fails(&["explain", "bad.cp"], "error[E0302]");
    fails(&["profile", "bad.cp"], "error[E0302]");
    fails(&["cover", "bad.cp"], "error[E0302]");
    fails(&["submit", "bad.cp"], "error[E0302]");
    fails(&["test", "bad.cp"], "error[E0302]");
    fails(&["doc", "missing.cp"], "can't read missing.cp");
    std::fs::write(dir.join("a.tests/1.out"), "0\n").unwrap();
    fails(&["test", "a.cp"], "0/1 passed");
    fails(&["submit", "a.cp"], "not ready to send");
}

#[test]
fn judges_rejects_a_judge_it_does_not_know_as_misuse() {
    let dir = scratch("judges_unknown");
    std::fs::write(dir.join("a.cp"), "// judge: nope\nprint(1)\n").unwrap();
    let out = cpl(&dir, &["judges", "a.cp"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()));
    let says = text(&out.stderr);
    assert!(
        says.contains("cpl: the `// judge:` line names judge `nope`"),
        "{says}"
    );
    assert!(says.contains("generic, codeforces"), "{says}");
    std::fs::write(dir.join("b.cp"), "print(1)\n").unwrap();
    std::fs::write(dir.join("cp.toml"), "[judge]\njudge = \"nope\"\n").unwrap();
    let out = cpl(&dir, &["judges", "b.cp"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()));
    assert!(text(&out.stderr).contains("names judge `nope`"));
    let out = cpl(&dir, &["judges", "b.cp", "--json"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()));
}

#[test]
fn build_json_says_where_the_bytes_of_the_cpp_go() {
    let dir = scratch("build-json");
    std::fs::write(dir.join("a.cp"), "let n: 1..=10 = read()\nprint(n)\n").unwrap();
    let out = cpl(&dir, &["build", "a.cp", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let json = cpl_json::parse(&text(&out.stdout)).expect("cpl build --json is JSON");
    let bytes = std::fs::read(dir.join("a.cpp")).unwrap().len() as i64;
    assert_eq!(json.at(&["path"]).as_str(), Some("a.cpp"));
    assert_eq!(json.at(&["bytes"]).as_i64(), Some(bytes));
    assert_eq!(json.at(&["judge"]).as_str(), Some("generic"));
    assert_eq!(json.at(&["limit"]).as_i64(), Some(65536));
    let sizes = json.at(&["sizes"]).as_array().unwrap();
    let parts: i64 = sizes
        .iter()
        .map(|s| s.at(&["bytes"]).as_i64().unwrap())
        .sum();
    assert_eq!(parts, bytes, "the sizes add up to the file");
    assert!(sizes
        .iter()
        .any(|s| s.at(&["part"]).as_str() == Some("your code")));
    let spelled = cpl(&dir, &["build", "a.cp", "--format", "json"]);
    assert_eq!(text(&spelled.stdout), text(&out.stdout));
    for args in [
        &["build", "a.cp", "--json", "-o", "-"][..],
        &["build", "a.cp", "--json", "--checked"],
    ] {
        let out = cpl(&dir, args);
        assert_eq!(out.status.code(), Some(EXIT_USAGE.into()), "{args:?}");
        assert!(
            text(&out.stderr).contains("`--json` prints the report"),
            "{args:?}"
        );
    }
}

#[test]
fn test_json_runs_the_cases_of_a_file_given_by_path_or_name() {
    let dir = scratch("test-json");
    std::fs::write(dir.join("a.cp"), "let n: 1..=10 = read()\nprint(n)\n").unwrap();
    std::fs::create_dir_all(dir.join("a.tests")).unwrap();
    std::fs::write(dir.join("a.tests/1.in"), "3\n").unwrap();
    std::fs::write(dir.join("a.tests/1.out"), "3\n").unwrap();
    for args in [
        &["test", "a.cp", "--json"][..],
        &["test", "a.cp", "--format", "json"],
        &["test", "a", "--json"],
    ] {
        let out = cpl(&dir, args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?}: {}",
            text(&out.stderr)
        );
        let json = cpl_json::parse(&text(&out.stdout)).expect("cpl test --json is JSON");
        assert_eq!(json.at(&["passed"]).as_i64(), Some(1), "{args:?}");
        assert_eq!(json.at(&["total"]).as_i64(), Some(1), "{args:?}");
    }
}

#[test]
fn submit_json_says_whether_it_is_ready_and_why() {
    let dir = scratch("submit-json");
    std::fs::write(dir.join("a.cp"), "let n: 1..=10 = read()\nprint(n)\n").unwrap();
    std::fs::create_dir_all(dir.join("a.tests")).unwrap();
    std::fs::write(dir.join("a.tests/1.in"), "3\n").unwrap();
    std::fs::write(dir.join("a.tests/1.out"), "3\n").unwrap();
    let submit = |args: &[&str]| {
        let out = cpl(&dir, args);
        let json = cpl_json::parse(&text(&out.stdout)).expect("cpl submit --json is JSON");
        (out.status.code(), json)
    };
    for args in [
        &["submit", "a.cp", "--json"][..],
        &["submit", "a.cp", "--format", "json"],
    ] {
        let (code, json) = submit(args);
        assert_eq!(code, Some(0), "{args:?}: {json:?}");
        assert_eq!(json.at(&["ready"]).as_bool(), Some(true));
        assert_eq!(json.at(&["build", "path"]).as_str(), Some("a.cpp"));
        assert_eq!(json.at(&["test", "passed"]).as_i64(), Some(1));
    }
    std::fs::write(dir.join("a.tests/1.out"), "4\n").unwrap();
    let (code, json) = submit(&["submit", "a.cp", "--json"]);
    assert_eq!(code, Some(EXIT_ERRORS.into()));
    assert_eq!(json.at(&["ready"]).as_bool(), Some(false));
    assert_eq!(json.at(&["test", "passed"]).as_i64(), Some(0));
    std::fs::write(dir.join("bad.cp"), "let x: Int = \"a\"\n").unwrap();
    let (code, json) = submit(&["submit", "bad.cp", "--json"]);
    assert_eq!(code, Some(EXIT_ERRORS.into()));
    assert_eq!(json.at(&["ready"]).as_bool(), Some(false));
    assert_eq!(*json.at(&["build"]), cpl_json::Value::Null);
}
