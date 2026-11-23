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
}

#[test]
fn explain_shows_a_code_and_rejects_an_unknown_one() {
    let out = cpl(Path::new("."), &["explain", "E0302"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).starts_with("E0302: "),
        "{}",
        text(&out.stdout)
    );
    let unknown = cpl(Path::new("."), &["explain", "E7777"]);
    assert_eq!(unknown.status.code(), Some(EXIT_USAGE.into()));
    assert!(text(&unknown.stderr).contains("unknown diagnostic or lint code `E7777`"));
    let json = cpl(Path::new("."), &["explain", "--json"]);
    let json = cpl_json::parse(&text(&json.stdout)).expect("cpl explain --json is JSON");
    assert!(json
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e.at(&["code"]).as_str() == Some("E0302")));
}
