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

/// `cpl migrate` over a folder: rewrites removed spellings with a diff,
/// leaves those without a rewrite to a hand edit, and reaches the brutes
/// and generators in `a.tests/`.
#[test]
fn migrate_rewrites_removed_spellings_under_a_folder() {
    helps("migrate");
    let dir = scratch("migrate");
    let old = "if true:\n    print(1)\nelse if false:\n    print(2)\n";
    let stuck = "for mut i in 0..3:\n    print(i)\n";
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("a.tests")).unwrap();
    std::fs::write(dir.join("a.cp"), old).unwrap();
    std::fs::write(dir.join("lib/b.cp"), stuck).unwrap();
    std::fs::write(dir.join("a.tests/c.cp"), old).unwrap();
    std::fs::write(dir.join("ok.cp"), "print(1)\n").unwrap();

    let checked = cpl(&dir, &["migrate", "--check"]);
    assert_eq!(checked.status.code(), Some(EXIT_ERRORS.into()));
    assert_eq!(std::fs::read_to_string(dir.join("a.cp")).unwrap(), old);

    let out = cpl(&dir, &["migrate"]);
    assert_eq!(out.status.code(), Some(EXIT_ERRORS.into()));
    assert_eq!(
        text(&out.stdout),
        "--- ./a.cp\n+++ ./a.cp\n@@ line 3 @@\n-else if false:\n+elif false:\n\
         --- ./a.tests/c.cp\n+++ ./a.tests/c.cp\n@@ line 3 @@\n-else if false:\n+elif false:\n"
    );
    assert!(
        text(&out.stderr).contains("`for mut` was removed"),
        "{}",
        text(&out.stderr)
    );
    let a = std::fs::read_to_string(dir.join("a.cp")).unwrap();
    assert_eq!(a, old.replace("else if", "elif"));
    assert_eq!(
        std::fs::read_to_string(dir.join("a.tests/c.cp")).unwrap(),
        a
    );

    std::fs::write(dir.join("lib/b.cp"), "for i in 0..3:\n    print(i)\n").unwrap();
    let done = cpl(&dir, &["migrate", "--format", "json"]);
    assert_eq!(done.status.code(), Some(0), "{}", text(&done.stderr));
    let json = cpl_json::parse(&text(&done.stdout)).expect("migrate --json is JSON");
    let files = json.at(&["files"]).as_array().unwrap();
    assert_eq!(files.len(), 4);
    assert!(files
        .iter()
        .all(|f| f.at(&["changed"]).as_bool() == Some(false)));
    let plain = cpl(&dir, &["migrate", "lib"]);
    assert_eq!(text(&plain.stdout), "1 files use the current spellings\n");
}

/// `cpl fetch` on a saved AtCoder page: creates the problem, then refreshes
/// its samples, and won't overwrite a test that says something else.
#[test]
fn fetch_writes_a_pages_samples_and_keeps_other_tests() {
    helps("fetch");
    misused("fetch", &["fetch"], "`fetch` needs a problem page");
    misused(
        "fetch",
        &["fetch", "https://example.com/p/1"],
        "is not a Codeforces or AtCoder page",
    );
    let dir = scratch("fetch");
    std::fs::write(
        dir.join("page.html"),
        "<span class=\"h2\">\nA - Sum\n<a href=\"/e\">Editorial</a></span>\n\
         <p>Time Limit: 2 sec / Memory Limit: 1024 MiB</p>\n<span class=\"lang-en\">\n\
         <h3>Sample Input 1</h3><pre>1 2\n</pre><h3>Sample Output 1</h3><pre>3\n</pre>\n\
         <h3>Sample Input 2</h3><pre>5 &amp; 6\n</pre><h3>Sample Output 2</h3><pre>11\n</pre></span>",
    )
    .unwrap();
    let url = "https://atcoder.jp/contests/abc999/tasks/abc999_a";
    let out = cpl(&dir, &["fetch", url, "--html", "page.html"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "  created a.cp\n  created a.tests/ (2 samples)\n  next: write it; `cpl test a.cp` runs the samples\n"
    );
    let source = std::fs::read_to_string(dir.join("a.cp")).unwrap();
    assert!(
        source.starts_with(&format!("// A - Sum\n// {url}\n")),
        "{source}"
    );
    let sample = |name: &str| std::fs::read_to_string(dir.join("a.tests").join(name)).unwrap();
    assert_eq!(
        (sample("1.in"), sample("1.out")),
        ("1 2\n".into(), "3\n".into())
    );
    assert_eq!(sample("2.in"), "5 & 6\n");

    let again = cpl(
        &dir,
        &["fetch", url, "--html", "page.html", "--format", "json"],
    );
    assert_eq!(again.status.code(), Some(0), "{}", text(&again.stderr));
    assert_eq!(
        text(&again.stdout),
        "{\"path\":\"a.cp\",\"created\":false,\"tests\":\"a.tests\",\"samples\":2}\n"
    );
    std::fs::write(dir.join("a.tests/2.out"), "12\n").unwrap();
    let kept = cpl(&dir, &["fetch", url, "--html", "page.html"]);
    assert_eq!(kept.status.code(), Some(EXIT_ERRORS.into()));
    assert!(
        text(&kept.stderr).contains("says something else"),
        "{}",
        text(&kept.stderr)
    );
    assert_eq!(sample("2.out"), "12\n");
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
    for json in [&["--json"][..], &["--format", "json"]] {
        let name = format!("j{}", json.len());
        let out = cpl(&dir, &[&["new", name.as_str()][..], json].concat());
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        assert_eq!(
            text(&out.stdout),
            format!("{{\"path\":\"{name}.cp\",\"created\":true,\"tests\":\"{name}.tests\",\"samples\":0}}\n")
        );
    }
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
    assert_eq!(commands.len(), 27, "{help}");
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
fn help_json_lists_every_command_with_its_usage_summary_json_and_help() {
    let out = cpl(Path::new("."), &["--help", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let json = cpl_json::parse(&text(&out.stdout)).expect("cpl --help --json is JSON");
    let commands = json.at(&["commands"]).as_array().unwrap();
    assert_eq!(commands.len(), 27);
    let listed = text(&cpl(Path::new("."), &["--help"]).stdout);
    let no_json: Vec<_> = commands
        .iter()
        .filter(|c| c.at(&["json"]).as_str().is_none())
        .map(|c| c.at(&["name"]).as_str().unwrap())
        .collect();
    assert_eq!(no_json, ["watch", "lsp", "dap", "setup", "self"]);
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
    // With no file named, the newest `.cp` file in the folder is checked.
    let newest = cpl(&dir, &["check", "--json"]);
    let json = cpl_json::parse(&text(&newest.stdout)).expect("JSON");
    assert_eq!(json.at(&["files"]).as_array().unwrap().len(), 1);
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

/// A file's explanation names the representation sections it can't fill
/// yet (plan/QUESTIONS.md tools-3), on a file with closures and on one
/// with none.
#[test]
fn explain_a_file_names_the_sections_not_shown_yet() {
    let dir = scratch("explain_not_shown");
    std::fs::write(
        dir.join("f.cp"),
        "let xs = [3, 1, 2]\nprint(xs.map(x => x * 2))\n",
    )
    .unwrap();
    std::fs::write(dir.join("plain.cp"), "print(1)\n").unwrap();
    for file in ["f.cp", "plain.cp"] {
        let out = cpl(&dir, &["explain", file]);
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        let stdout = text(&out.stdout);
        assert!(
            stdout.ends_with(
                "Not shown yet:\n  types: bytes and niches\n  arrays and peak memory\n  \
                 specializations chosen\n  checks left per build\n"
            ),
            "{file}: {stdout}"
        );
    }
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
fn a_judge_it_does_not_know_is_the_files_error_or_misuse_on_the_command_line() {
    let dir = scratch("judges_unknown");
    std::fs::write(dir.join("a.cp"), "// judge: nope\nprint(1)\n").unwrap();
    let out = cpl(&dir, &["judges", "a.cp"]);
    assert_eq!(out.status.code(), Some(EXIT_ERRORS.into()));
    let says = text(&out.stderr);
    assert!(
        says.contains("cpl: the `// judge:` line names judge `nope`"),
        "{says}"
    );
    assert!(says.contains("generic, codeforces"), "{says}");
    std::fs::write(dir.join("b.cp"), "print(1)\n").unwrap();
    std::fs::write(dir.join("cp.toml"), "[judge]\njudge = \"nope\"\n").unwrap();
    let out = cpl(&dir, &["judges", "b.cp"]);
    assert_eq!(out.status.code(), Some(EXIT_ERRORS.into()));
    assert!(text(&out.stderr).contains("names judge `nope`"));
    let out = cpl(&dir, &["judges", "b.cp", "--json"]);
    assert_eq!(out.status.code(), Some(EXIT_ERRORS.into()));
    let out = cpl(&dir, &["build", "b.cp"]);
    assert_eq!(out.status.code(), Some(EXIT_ERRORS.into()));
    std::fs::remove_file(dir.join("cp.toml")).unwrap();
    let out = cpl(&dir, &["build", "b.cp", "--judge", "nope"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()));
    assert!(
        text(&out.stderr).contains("names judge `nope`"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn build_checks_names_what_the_judge_build_still_checks() {
    let dir = scratch("build-checks");
    std::fs::write(dir.join("a.cp"), "let n: Int = read()\nprint(10 / n)\n").unwrap();
    for (level, trap, local) in [
        ("none", false, false),
        ("traps", true, false),
        ("all", true, true),
    ] {
        let out = cpl(&dir, &["build", "a.cp", "--checks", level, "-o", "-"]);
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        let cpp = text(&out.stdout);
        assert_eq!(cpp.contains("division by zero"), trap, "{level}");
        assert_eq!(cpp.contains("#define LOCAL"), local, "{level}");
        let json = cpl(&dir, &["build", "a.cp", "--checks", level, "--json"]);
        let json = cpl_json::parse(&text(&json.stdout)).expect("JSON");
        assert_eq!(json.at(&["checks"]).as_str(), Some(level));
    }
    let default = cpl(&dir, &["build", "a.cp", "-o", "-"]);
    let none = cpl(&dir, &["build", "a.cp", "--checks", "none", "-o", "-"]);
    assert_eq!(
        text(&default.stdout),
        text(&none.stdout),
        "`none` is the default"
    );
    for args in [&["--checks", "some"][..], &["--checks"]] {
        let out = cpl(&dir, &[&["build", "a.cp"][..], args].concat());
        assert_eq!(out.status.code(), Some(EXIT_USAGE.into()), "{args:?}");
        assert!(
            text(&out.stderr).contains("`--checks` takes one of `none`, `traps`, `all`"),
            "{}",
            text(&out.stderr)
        );
    }
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

#[test]
fn report_json_names_the_report_it_wrote() {
    let dir = scratch("report-json");
    std::fs::write(dir.join("a.cp"), "let n: Int = read()\nprint(n * 2)\n").unwrap();
    std::fs::write(dir.join("in.txt"), "21\n").unwrap();
    let mut sections = Vec::new();
    for args in [
        &["report", "a.cp", "--input", "in.txt", "--json"][..],
        &["report", "a.cp", "--input", "in.txt", "--format", "json"],
    ] {
        let out = cpl(&dir, args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?}: {}",
            text(&out.stderr)
        );
        let json = cpl_json::parse(&text(&out.stdout)).expect("cpl report --json is JSON");
        let path = json.at(&["path"]).as_str().expect("a path").to_string();
        assert!(
            path.starts_with("cpl-report-") && path.ends_with(".md"),
            "{path}"
        );
        let written = std::fs::read(dir.join(&path))
            .expect("the report is written")
            .len() as i64;
        assert_eq!(json.at(&["bytes"]).as_i64(), Some(written));
        assert!(
            json.at(&["issue_url"]).as_str().is_some(),
            "a small report has an issue URL"
        );
        sections.push(json.at(&["sections"]).clone());
    }
    assert_eq!(sections[0], sections[1]);
    assert!(
        sections[0].to_string().contains("source"),
        "{}",
        sections[0]
    );
}

/// A problem is named `a` or `a.cp` alike by every command that takes a
/// file, as `cpl test a` always did; `cpl a.cp` says which command runs it.
#[test]
fn a_problem_name_without_cp_names_its_file() {
    let dir = scratch("bare-name");
    std::fs::write(dir.join("a.cp"), "let n: Int = read()\nprint(n * 2)\n").unwrap();
    std::fs::write(dir.join("case.in"), "21\n").unwrap();
    let out = cpl(&dir, &["run", "a", "case.in"]);
    assert_eq!(
        (out.status.code(), text(&out.stdout)),
        (Some(0), "42\n".into()),
        "{}",
        text(&out.stderr)
    );
    let out = cpl(&dir, &["check", "a"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("a.cp: no errors"));
    let out = cpl(&dir, &["build", "a", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(dir.join("a.cpp").is_file());
    // A file of that very name, or a missing one, is left as it is.
    let out = cpl(&dir, &["check", "b"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()));
    assert!(
        text(&out.stderr).contains("can't read `b`"),
        "{}",
        text(&out.stderr)
    );
    let out = cpl(&dir, &["a.cp"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()));
    assert!(text(&out.stderr)
        .starts_with("cpl: `a.cp` is a file, not a command: `cpl run a.cp` runs it"));
}

/// With no file named, a command reads the most recently edited `.cp` file
/// in the folder (spec/tools/cli.md); with none there, it says what it needs.
#[test]
fn no_file_named_reads_the_newest_cp_file() {
    let dir = scratch("newest");
    let out = cpl(&dir, &["check"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()));
    assert!(
        text(&out.stderr).contains("needs a file"),
        "{}",
        text(&out.stderr)
    );
    std::fs::write(dir.join("a.cp"), "print(1)\n").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(dir.join("b.cp"), "print(2)\n").unwrap();
    let out = cpl(&dir, &["check"]);
    assert_eq!(
        text(&out.stdout),
        "b.cp: no errors\n",
        "{}",
        text(&out.stderr)
    );
    let out = cpl(&dir, &["run"]);
    assert_eq!(text(&out.stdout), "2\n", "{}", text(&out.stderr));
    let out = cpl(&dir, &["run", "--help"]);
    assert!(text(&out.stdout).starts_with("usage: cpl run"));
}

/// `cpl locate`: exit 0 when every part agrees with its alt on the input,
/// 1 naming the part that does not, 2 on misuse.
#[test]
fn locate_names_the_part_whose_alt_disagrees() {
    let dir = scratch("locate");
    std::fs::write(
        dir.join("p.cp"),
        "fn sq(n: Int) -> Int = if n == 3: 10 else: n * n\n@alt(sq)\nfn sq_slow(n: Int) -> Int = n * n\nlet n: Int = read()\nprint(sq(n))\n",
    )
    .unwrap();
    std::fs::write(dir.join("fine.in"), "4\n").unwrap();
    std::fs::write(dir.join("bad.in"), "3\n").unwrap();
    let out = cpl(&dir, &["locate", "p", "fine.in"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(text(&out.stdout).starts_with("every part agrees"));
    let out = cpl(&dir, &["locate", "p", "bad.in"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout)
            .starts_with("`sq` disagrees with its alt `sq_slow` on bad.in: line 1 is \"10\""),
        "{}",
        text(&out.stdout)
    );
    let out = cpl(&dir, &["locate", "p", "a", "b"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()));
}

/// Each step of the contest flow names the next: `new` names `test`, a
/// passing `test` names `submit`; JSON output stays one object.
#[test]
fn passing_samples_point_at_submit() {
    let dir = scratch("next-step");
    std::fs::write(dir.join("a.cp"), "let n: Int = read()\nprint(n + 1)\n").unwrap();
    std::fs::create_dir_all(dir.join("a.tests")).unwrap();
    std::fs::write(dir.join("a.tests/1.in"), "1\n").unwrap();
    std::fs::write(dir.join("a.tests/1.out"), "2\n").unwrap();
    let out = cpl(&dir, &["test", "a"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stderr).contains("next: `cpl submit a` builds"),
        "{}",
        text(&out.stderr)
    );
    let out = cpl(&dir, &["test", "a", "--json"]);
    assert_eq!(text(&out.stderr), "");
    std::fs::write(dir.join("a.tests/1.out"), "3\n").unwrap();
    let out = cpl(&dir, &["test", "a"]);
    assert!(
        !text(&out.stderr).contains("next:"),
        "{}",
        text(&out.stderr)
    );
    // With a brute to compare against, a failure points at stress.
    std::fs::write(
        dir.join("a.cp"),
        "let n: 1..=9 = read()\nprint(n + 1)\n\n@alt(program)\nfn brute():\n    let n: 1..=9 = read()\n    print(n + 1)\n",
    )
    .unwrap();
    let out = cpl(&dir, &["test", "a"]);
    assert!(
        text(&out.stderr).contains("next: `cpl stress a` looks for"),
        "{}",
        text(&out.stderr)
    );
}

/// `cpl run a 2` runs the sample `2` from `a.tests/` and judges it; an
/// unknown sample name is still read as an input file.
#[test]
fn run_names_a_sample_to_run_and_judge() {
    let dir = scratch("run-sample");
    std::fs::write(dir.join("a.cp"), "let n: Int = read()\nprint(n * 2)\n").unwrap();
    std::fs::create_dir_all(dir.join("a.tests")).unwrap();
    for (name, input, output) in [("1", "1", "2"), ("2", "5", "10")] {
        std::fs::write(dir.join(format!("a.tests/{name}.in")), format!("{input}\n")).unwrap();
        std::fs::write(
            dir.join(format!("a.tests/{name}.out")),
            format!("{output}\n"),
        )
        .unwrap();
    }
    let out = cpl(&dir, &["run", "a", "2"]);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{stdout}{}", text(&out.stderr));
    assert!(
        stdout.starts_with("  2  ok ") && stdout.ends_with("  1/1 passed\n"),
        "{stdout}"
    );

    let out = cpl(&dir, &["run", "a", "3"]);
    assert_eq!(
        out.status.code(),
        Some(EXIT_USAGE as i32),
        "{}",
        text(&out.stdout)
    );
    assert!(
        text(&out.stderr).contains("can't read `3`"),
        "{}",
        text(&out.stderr)
    );
}

/// `cpl b` is `cpl build`, with the same help and the same file.
#[test]
fn b_is_build() {
    let dir = scratch("b");
    std::fs::write(dir.join("a.cp"), "print(1)\n").unwrap();
    assert_eq!(
        cpl(&dir, &["b", "--help"]).stdout,
        cpl(&dir, &["build", "--help"]).stdout
    );
    let out = cpl(&dir, &["b", "a"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(dir.join("a.cpp").is_file());
    let out = cpl(&dir, &["b", "a", "-x"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE as i32));
    assert!(
        text(&out.stderr).contains("unknown option `-x` for `build`"),
        "{}",
        text(&out.stderr)
    );
}

/// `cpl test a --add in --expect out` writes the next numbered case and
/// runs only it.
#[test]
fn test_adds_a_custom_case_and_runs_it() {
    let dir = scratch("test-add");
    std::fs::write(dir.join("a.cp"), "let n: Int = read()\nprint(n * 2)\n").unwrap();
    std::fs::create_dir_all(dir.join("a.tests")).unwrap();
    std::fs::write(dir.join("a.tests/1.in"), "1\n").unwrap();
    std::fs::write(dir.join("a.tests/1.out"), "2\n").unwrap();
    std::fs::write(dir.join("mine.in"), "21\n").unwrap();
    std::fs::write(dir.join("mine.out"), "42\n").unwrap();
    let out = cpl(
        &dir,
        &["test", "a", "--add", "mine.in", "--expect", "mine.out"],
    );
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{stdout}{}", text(&out.stderr));
    assert!(
        stdout.starts_with("  2  ok ") && stdout.ends_with("  1/1 passed\n"),
        "{stdout}"
    );
    assert!(
        text(&out.stderr).contains("cpl: added "),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("a.tests/2.out")).unwrap(),
        "42\n"
    );

    let out = cpl(&dir, &["test", "a", "--add", "nope.in"]);
    assert_eq!(out.status.code(), Some(EXIT_ERRORS as i32));
    assert!(
        text(&out.stderr).contains("can't read `nope.in`"),
        "{}",
        text(&out.stderr)
    );
    assert!(!dir.join("a.tests/3.in").exists());
}

/// `cpl setup neovim` writes the plugin, filetype, syntax and queries where
/// Neovim loads a start package; `vim` leaves out what only Neovim reads.
#[test]
fn setup_writes_the_editor_files() {
    let dir = scratch("setup");
    let out = cpl(&dir, &["setup", "neovim", "--dir", "nvim"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).starts_with("installed CP Lang for neovim in nvim"),
        "{}",
        text(&out.stdout)
    );
    for file in [
        "plugin/cplang.lua",
        "ftdetect/cp.vim",
        "syntax/cp.vim",
        "queries/cp/highlights.scm",
    ] {
        assert!(dir.join("nvim").join(file).is_file(), "{file}");
    }
    let out = cpl(&dir, &["setup", "vim", "--dir", "vim"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        dir.join("vim/plugin/cplang.vim").is_file() && !dir.join("vim/plugin/cplang.lua").exists()
    );

    let out = cpl(&dir, &["setup", "emacs"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE as i32));
    assert!(
        text(&out.stderr).contains("no setup for `emacs`"),
        "{}",
        text(&out.stderr)
    );
}
