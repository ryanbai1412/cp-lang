use crate::cli;
use std::path::{Path, PathBuf};

/// `testdata/library-lint` copied to a fresh directory: a program using a
/// library whose fn declares `@lint`.
fn fixture(name: &str) -> PathBuf {
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/library-lint");
    let to = std::env::temp_dir().join(format!("cpl-library-lint-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&to);
    for rel in ["cp.toml", "main.cp", "mylib/lib.toml", "mylib/geo.cp"] {
        std::fs::create_dir_all(to.join(rel).parent().unwrap()).unwrap();
        std::fs::copy(from.join(rel), to.join(rel)).unwrap();
    }
    to
}

/// A library's `@lint` fires in a program that uses it, keeps quiet where the
/// two sides differ, where a repeated side calls something, and under
/// `with lint(name: allow)`; `cpl fix` writes the call.
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn a_library_lint_fires_and_fixes_in_a_program_using_the_library() {
    let dir = fixture("fires");
    let main = dir.join("main.cp").to_string_lossy().into_owned();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    cli::run(std::slice::from_ref(&main), &mut out, &mut err);
    let report = String::from_utf8_lossy(&out).into_owned() + &String::from_utf8_lossy(&err);
    assert_eq!(
        report.matches("warning[square_by_hand]").count(),
        2,
        "{report}"
    );
    assert!(
        report.contains("write `geo.sq(xa - xb)`") && report.contains("write `geo.sq(xa)`"),
        "{report}"
    );
    assert!(
        report.contains("`mylib.geo` declares this lint"),
        "{report}"
    );
    cli::run_fix(
        std::slice::from_ref(&main),
        &mut Vec::new(),
        &mut Vec::new(),
    );
    let fixed = std::fs::read_to_string(&main).unwrap();
    assert!(
        fixed.contains("let d2 = geo.sq(xa - xb)\n") && fixed.contains("let f = geo.sq(xa) + 1\n"),
        "{fixed}"
    );
    assert!(
        fixed.contains("let e = xa * xb\n") && fixed.contains("let g: Int = read() * read()\n"),
        "{fixed}"
    );
    assert!(fixed.contains("print(d2, e, f, g, xb * xb)"), "{fixed}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `cp.toml`'s `[lints]` sets a library lint's level like a built-in one's,
/// without calling it unknown; a name no library declares still is.
/// Rule: lints/R19.
#[test]
fn cp_toml_sets_a_library_lints_level() {
    let dir = fixture("level");
    let toml = dir.join("cp.toml");
    let mut text = std::fs::read_to_string(&toml).unwrap();
    text += "\n[lints]\nsquare_by_hand = \"deny\"\nnope = \"allow\"\n";
    std::fs::write(&toml, text).unwrap();
    let main = dir.join("main.cp").to_string_lossy().into_owned();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    cli::run(std::slice::from_ref(&main), &mut out, &mut err);
    let report = String::from_utf8_lossy(&out).into_owned() + &String::from_utf8_lossy(&err);
    assert_eq!(
        report.matches("error[square_by_hand]").count(),
        2,
        "{report}"
    );
    assert!(
        !report.contains("no lint called `square_by_hand`"),
        "{report}"
    );
    assert!(report.contains("no lint called `nope`"), "{report}");
}

/// A `from` may be a run of statements: it matches consecutive statements of
/// one block, whole, with each bound name the same across the run, and its
/// fix replaces the run. A wider finding it rewrites (L0521) gives way.
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn a_library_lint_matches_a_run_of_statements() {
    let decl = "@lint(totals, from: \"var p = [0]\\nfor x in xs:\\n    p.push(p.last! + x)\", to: \"let p = totals(xs)\")\npub fn totals(xs: List<Int>) -> Int:\n    xs.len\n";
    let found = |body: &str| -> Vec<(String, String)> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .map(|d| {
                let fix = d.fixes.first().map(|f| f.edits[0].1.clone());
                (d.code.0.to_string(), fix.unwrap_or_default())
            })
            .collect()
    };
    let run =
        "let a = [3, 1, 4]\nvar s = [0]\nfor v in a:\n    s.push(s.last! + v)\nprint(s.len)\n";
    // std declares the same run on `scan` (decision 512).
    assert_eq!(
        found(run),
        [
            ("prefix_totals_list".into(), "let s = a.scan(+)".into()),
            ("totals".into(), "let s = totals(a)".into())
        ]
    );
    let nested = "fn f(a: List<Int>):\n    var s = [0]\n    for v in a:\n        s.push(s.last! + v)\n    print(s.len)\nf([1])\n";
    assert!(found(nested).contains(&("totals".into(), "let s = totals(a)".into())));
    // Another statement between, or a different list pushed to, is not the run.
    let apart = "let a = [3, 1, 4]\nvar s = [0]\nprint(a)\nfor v in a:\n    s.push(s.last! + v)\n";
    let other =
        "let a = [3, 1, 4]\nvar s = [0]\nvar t = [0]\nfor v in a:\n    t.push(s.last! + v)\n";
    for near in [apart, other] {
        assert!(
            found(near).iter().all(|(c, _)| !c.contains("totals")),
            "{near}"
        );
    }
}

/// `holes:` names more holes than the fn's parameters; `to` may drop one
/// when it matched code that calls nothing, `to` may use the fn as a getter
/// (`xs.walk`), and a lint named like a `spec/lints` record reports as it.
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn declared_holes_match_free_names_and_a_record_name_keeps_its_code() {
    let decl = "@lint(binom_by_hand, from: \"for i in 0..n:\\n    print(xs[i])\", to: \"for x in xs.walk:\\n    print(x)\", holes: \"n\")\npub fn walk(xs: List<Int>) -> List<Int> = xs\n";
    let found = |body: &str| -> Vec<(String, String)> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "L0557")
            .map(|d| (d.code.0.to_string(), d.message.clone()))
            .collect()
    };
    let hit = "let a = [3, 1]\nlet m = a.len\nfor j in 0..m:\n    print(a[j])\n";
    assert_eq!(
        found(hit),
        [(
            "L0557".into(),
            "write `for x in a.walk:\n    print(x)`".into()
        )]
    );
    let calls = "let a = [3, 1]\nfn m() -> Int = 2\nfor j in 0..m():\n    print(a[j])\n";
    assert!(found(calls).is_empty());
}

/// One fn declares a lint twice, one `from` per spelling, and `from` does
/// not match an assigned place. Rule: R06.4.9.
#[test]
fn a_lint_has_spellings_and_skips_assigned_places() {
    let decl = "@lint(at_end, from: \"xs[xs.len - 1]\", to: \"end(xs)\")\n@lint(at_end, from: \"xs[-1 + xs.len]\", to: \"end(xs)\")\npub fn end(xs: List<Int>) -> Int = xs[0]\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "at_end")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let read = "var a = [3, 1]\nprint(a[a.len - 1] + a[-1 + a.len])\n";
    assert_eq!(found(read), ["end(a)", "end(a)"]);
    assert!(found("var a = [3, 1]\na[a.len - 1] = 0\nprint(a)\n").is_empty());
}

/// A lint named like a record with `no_fix` still names `to`, but offers no
/// fix: `yes(ok)` prints the judge profile's words, which may not be the
/// ones written (decision 301).
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn a_record_with_no_fix_keeps_a_library_lint_to_advice() {
    let decl = "@lint(bool_ternary_print, from: \"if ok: \\\"YES\\\" else: \\\"NO\\\"\", to: \"word(ok)\")\npub fn word(ok: Bool) -> Str = yes(ok)\n";
    let src = format!("{decl}let n: Int = read()\nprint(if n > 0: \"YES\" else: \"NO\")\n");
    let found: Vec<_> = crate::lint(&src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.message == "write `word(n > 0)`")
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].code.0, "L0524");
    assert!(found[0].fixes.is_empty(), "{:?}", found[0].fixes);
}

/// A `fn` a `from` declares is bound, its name and its parameters, so a
/// recursive helper matches under any names. Rule: R06.4.9.
#[test]
fn a_fn_from_declares_binds_its_name_and_parameters() {
    let decl = "@lint(walk_all, from: \"fn go(u: Int):\\n    if u > 0:\\n        go(u - 1)\\ngo(n)\", to: \"walk(n)\")\npub fn walk(n: Int) = ()\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "walk_all")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let renamed = "let k = 3\nfn down(x: Int):\n    if x > 0:\n        down(x - 1)\ndown(k)\n";
    assert_eq!(found(renamed), ["walk(k)"]);
    let mixed = "let k = 3\nfn down(x: Int):\n    if x > 0:\n        go(x - 1)\nfn go(y: Int) = ()\ndown(k)\n";
    assert!(found(mixed).is_empty());
}

/// `when:` keeps a finding only where what a hole matched is proven in
/// bounds: an integer literal, or a range type whose ends lie in them.
/// Rule: R06.4.9.
#[test]
fn when_keeps_a_finding_only_where_the_hole_is_proven_in_bounds() {
    let decl = "@lint(up, from: \"(self + b - 1) / b\", to: \"up(self, b)\", when: \"b >= 1\")\npub fn up(self: Int, b: Int) -> Int = self\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "up")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let src = "let n: 1..=100 = read()\nlet k: 1..=10 = read()\nlet m: 0..=10 = read()\nlet j: Int = read()\nprint((n + k - 1) / k, (n + 4 - 1) / 4, (n + m - 1) / m, (n + j - 1) / j, (n + 0 - 1) / 0)\n";
    assert_eq!(found(src), ["up(n, k)", "up(n, 4)"]);
}

/// A run ending in a block is replaced up to its last token, so the fix
/// keeps the next statement's line and indent. Rule: R06.4.9.
#[test]
fn a_run_ending_in_a_block_is_replaced_up_to_its_last_token() {
    let decl = "@lint(totals, from: \"var p = [0]\\nfor x in xs:\\n    p.push(p.last! + x)\", to: \"let p = totals(xs)\")\npub fn totals(xs: List<Int>) -> Int:\n    xs.len\n";
    let src = format!("{decl}fn f(a: List<Int>):\n    var s = [0]\n    for v in a:\n        s.push(s.last! + v)\n    print(s.len)\nf([1])\n");
    let spans: Vec<&str> = crate::lint(&src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.code.0 == "totals")
        .map(|d| &src[d.fixes[0].edits[0].0.start as usize..d.fixes[0].edits[0].0.end as usize])
        .collect();
    assert_eq!(
        spans,
        ["var s = [0]\n    for v in a:\n        s.push(s.last! + v)"]
    );
}
