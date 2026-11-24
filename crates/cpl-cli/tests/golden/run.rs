//! The contest loop: `test`, `run`, `build`, `judge`, `submit`, `report`, `watch`, `replay`.

use super::*;

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
        "let n: 1..=5 = read()\nprint(n)\n@check\nfn check():\n    let x: Int = read_output()\n    assert x == n\n",
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
fn test_names_an_option_it_does_not_have_instead_of_a_missing_file() {
    let dir = scratch("test-unknown-option");
    std::fs::write(dir.join("a.cp"), "test \"one\":\n    assert 1 + 1 == 2\n").unwrap();
    let out = cpl(&dir, &["test", "a.cp", "--judge", "generic"]);
    let err = text(&out.stderr);
    assert!(err.contains("`cpl test` has no `--judge` option"), "{err}");
    assert!(err.contains("cpl build a.cp --judge generic"), "{err}");
    assert!(!err.contains("does not exist"), "{err}");
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
        assert_eq!(json.at(&["passed_count"]).as_i64(), Some(1), "{args:?}");
        assert_eq!(json.at(&["case_count"]).as_i64(), Some(1), "{args:?}");
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
        assert_eq!(json.at(&["test", "passed_count"]).as_i64(), Some(1));
    }
    std::fs::write(dir.join("a.tests/1.out"), "4\n").unwrap();
    let (code, json) = submit(&["submit", "a.cp", "--json"]);
    assert_eq!(code, Some(EXIT_ERRORS.into()));
    assert_eq!(json.at(&["ready"]).as_bool(), Some(false));
    assert_eq!(json.at(&["test", "passed_count"]).as_i64(), Some(0));
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
    // With no `// judge:` line, submit would check `generic`'s limits only.
    assert!(text(&out.stderr)
        .contains("for judge `generic` until the file starts with `// judge: codeforces`"));
    std::fs::write(
        dir.join("a.cp"),
        "// judge: codeforces\nlet n: Int = read()\nprint(n + 1)\n",
    )
    .unwrap();
    let out = cpl(&dir, &["test", "a"]);
    assert!(
        !text(&out.stderr).contains("generic"),
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
        text(&out.stderr)
            .contains("cpl: `build` has no option `-x`; `cpl build --help` lists them"),
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

    std::fs::write(dir.join("bare.in"), "5").unwrap();
    std::fs::write(dir.join("bare.out"), "10").unwrap();
    let out = cpl(
        &dir,
        &["test", "a", "--add", "bare.in", "--expect", "bare.out"],
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        std::fs::read_to_string(dir.join("a.tests/3.in")).unwrap(),
        "5\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("a.tests/3.out")).unwrap(),
        "10\n"
    );

    let out = cpl(&dir, &["test", "a", "--add", "nope.in"]);
    assert_eq!(out.status.code(), Some(EXIT_ERRORS as i32));
    assert!(
        text(&out.stderr).contains("can't read `nope.in`"),
        "{}",
        text(&out.stderr)
    );
    assert!(!dir.join("a.tests/4.in").exists());
}

/// The contest flow from a Competitive Companion problem to the clipboard:
/// `new --companion`, `run` (every sample), `test --add`, `stress`, `b -c`.
#[cfg(unix)]
#[test]
fn contest_flow_from_companion_to_clipboard() {
    let dir = scratch("contest-flow");
    std::fs::write(
        dir.join("p.json"),
        r#"{"name":"A. Sum","group":"Codeforces Round","url":"https://codeforces.com/problemset/problem/1/A","memoryLimit":256,"timeLimit":1000,"tests":[{"input":"1 2\n","output":"3\n"},{"input":"5 7\n","output":"12\n"}],"testType":"single","input":{"type":"stdin"},"output":{"type":"stdout"}}"#,
    )
    .unwrap();
    let out = cpl(&dir, &["new", "--companion", "p.json"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(dir.join("a.tests/2.out").is_file());

    std::fs::write(
        dir.join("a.cp"),
        "let (a, b): (1..=9, 1..=9) = read()\nprint(a + b)\n\n@alt(program)\nfn brute():\n    let (a, b): (1..=9, 1..=9) = read()\n    var s = a\n    for _ in 0..b: s += 1\n    print(s)\n",
    )
    .unwrap();
    let out = cpl(&dir, &["run", "a"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).contains("2/2 passed"),
        "{}",
        text(&out.stdout)
    );

    std::fs::write(dir.join("mine.in"), "9 9\n").unwrap();
    std::fs::write(dir.join("mine.out"), "18\n").unwrap();
    let out = cpl(
        &dir,
        &["test", "a", "--add", "mine.in", "--expect", "mine.out"],
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        std::fs::read_to_string(dir.join("a.tests/3.out")).unwrap(),
        "18\n"
    );
    let out = cpl(&dir, &["t", "a"]);
    assert!(
        text(&out.stdout).contains("3/3 passed"),
        "{}",
        text(&out.stdout)
    );

    let out = cpl(&dir, &["stress", "a", "--cases", "50"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).contains("50 cases agree"),
        "{}",
        text(&out.stdout)
    );

    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["b", "a", "-c"])
        .current_dir(&dir)
        .env("CPL_CLIPBOARD", "cat > clip")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let cpp = std::fs::read_to_string(dir.join("a.cpp")).unwrap();
    assert_eq!(std::fs::read_to_string(dir.join("clip")).unwrap(), cpp);
    assert!(cpp.len() < 64 * 1024);
}

/// BLOCKERS run 6 #23: a solution that reads its queries in a loop keeps a
/// brute that reads them the same way, sharing the header's `n`, `q`, `a`.
#[test]
fn a_brute_reading_queries_in_a_loop_shares_the_header() {
    let dir = scratch("brute-query-reads");
    std::fs::write(
        dir.join("a.cp"),
        "@!multitest\nlet (n, q): (1..=9, 1..=9) = read()\nlet a: [0..n]Int = read()\nfor _ in 0..q:\n    let i: 1..=n = read()\n    print(a[i - 1])\n\n@alt(program)\nfn brute():\n    for _ in 0..q:\n        let i: 1..=n = read()\n        print(a[i - 1])\n",
    )
    .unwrap();
    let out = cpl(&dir, &["check", "a.cp"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
}
