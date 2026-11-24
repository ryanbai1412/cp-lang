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
