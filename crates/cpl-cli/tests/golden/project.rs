//! Making and moving projects: `new`, `fetch`, `migrate`, `doc`, finding the file, the pinned toolchain.

use super::*;

/// `f(c) + λ * c`: the rejected `λ` stood for an operand, so the `*`
/// multiplies; `cpl migrate` must not drop it as a C dereference.
#[test]
fn migrate_keeps_a_product_after_a_rejected_character() {
    let dir = scratch("migrate-lambda");
    let src = "let c = 3\nlet x = c + λ * c\nprint(x)\n";
    std::fs::write(dir.join("x.cp"), src).unwrap();
    let out = cpl(&dir, &["migrate", "x.cp"]);
    assert_eq!(
        std::fs::read_to_string(dir.join("x.cp")).unwrap(),
        src,
        "{}",
        text(&out.stdout)
    );
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
    assert_eq!(text(&plain.stdout), "1 file uses the current spellings\n");
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
        "is not a Codeforces, AtCoder or CSES page",
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
        "{\"path\":\"a.cp\",\"created\":false,\"tests\":\"a.tests\",\"sample_count\":2}\n"
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
            format!("{{\"path\":\"{name}.cp\",\"created\":true,\"tests\":\"{name}.tests\",\"sample_count\":0}}\n")
        );
    }
}

#[test]
fn new_lays_out_a_project_by_its_template_and_the_tools_find_it() {
    // commands/R15: a cp.toml project with its own template.cp, a problem
    // made a folder down, and a corpus-style sibling `tests/` beside it.
    let dir = scratch("new-template");
    std::fs::write(dir.join("cp.toml"), "[limits]\ntime_ms = 4000\n").unwrap();
    let template = "let n: 1..=10 = read()\nprint(n * 2)\n";
    std::fs::write(dir.join("template.cp"), template).unwrap();
    let round = dir.join("round1");
    std::fs::create_dir_all(round.join("tests")).unwrap();
    let out = cpl(&round, &["new", "a", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "{\"path\":\"a.cp\",\"created\":true,\"tests\":\"a.tests\",\"sample_count\":0}\n"
    );
    let mut layout = Vec::new();
    let mut stack = vec![dir.clone()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap().flatten() {
            let path = entry.path();
            let rel = path
                .strip_prefix(&dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                stack.push(path);
                layout.push(format!("{rel}/"));
            } else {
                layout.push(rel);
            }
        }
    }
    layout.sort();
    assert_eq!(
        layout,
        [
            "cp.toml",
            "round1/",
            "round1/a.cp",
            "round1/a.tests/",
            "round1/tests/",
            "template.cp"
        ]
    );
    assert_eq!(
        std::fs::read_to_string(round.join("a.cp")).unwrap(),
        template
    );
    for (case, input, output) in [("a.tests/1", "3\n", "6\n"), ("tests/1", "4\n", "9\n")] {
        std::fs::write(round.join(format!("{case}.in")), input).unwrap();
        std::fs::write(round.join(format!("{case}.out")), output).unwrap();
    }
    let test = cpl(&round, &["test", "a.cp"]);
    assert_eq!(test.status.code(), Some(0), "{}", text(&test.stdout));
    assert!(
        text(&test.stdout).ends_with("  1/1 passed\n"),
        "{}",
        text(&test.stdout)
    );
    let run = cpl(&round, &["run", "a.cp", "--samples"]);
    assert_eq!(run.status.code(), Some(0), "{}", text(&run.stderr));
    assert_eq!(
        text(&run.stdout).split(' ').next(),
        Some("1:"),
        "{}",
        text(&run.stdout)
    );
    assert!(
        text(&run.stdout).contains("passed"),
        "{}",
        text(&run.stdout)
    );
    // Without `a.tests/`, the sibling `tests/` holds the cases.
    std::fs::remove_dir_all(round.join("a.tests")).unwrap();
    let test = cpl(&round, &["test", "a.cp"]);
    assert_ne!(test.status.code(), Some(0), "{}", text(&test.stdout));
    assert!(
        text(&test.stdout).contains("0/1 passed"),
        "{}",
        text(&test.stdout)
    );
}

#[test]
fn doc_writes_each_comment_as_json_or_a_site() {
    helps("doc");
    misused(
        "doc",
        &["doc", "--bogus"],
        "`doc` has no option `--bogus`; `cpl doc --help` lists them",
    );
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
    std::fs::write(dir.join("b.cp"), "print(2)\n").unwrap();
    let a_time = std::fs::metadata(dir.join("a.cp"))
        .unwrap()
        .modified()
        .unwrap();
    std::fs::File::options()
        .write(true)
        .open(dir.join("b.cp"))
        .unwrap()
        .set_modified(a_time + std::time::Duration::from_secs(1))
        .unwrap();
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

/// `cpl build --compiler <toolchain>`: the running toolchain builds itself;
/// another one goes through the installed launcher, and with none it says so.
#[test]
fn build_compiler_rebuilds_with_the_pinned_toolchain() {
    let dir = scratch("build-compiler");
    std::fs::write(dir.join("a.cp"), "print(1)\n").unwrap();
    let version = env!("CARGO_PKG_VERSION");
    let out = cpl(&dir, &["build", "a.cp", "-o", "-", "--compiler", version]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains(&format!("cpl {version}")));
    misused(
        "build",
        &["build", "a.cp", "--compiler"],
        "`--compiler` takes a toolchain",
    );
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["build", "a.cp", "--compiler", "0.0.1"])
        .current_dir(&dir)
        .env("CPL_HOME", dir.join("no-home"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()));
    assert!(text(&out.stderr).contains("through the installed launcher, and there is none"));
}
