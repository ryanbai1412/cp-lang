use crate::{checker_lints, lint, Config, CHECKER_LINTS};
use cpl_diag::{Diagnostic, Severity};

const PADDED: &str =
    "let a: [200000](w: Int, b: Bool) = [200000].filled((w: 0, b: false))\nprint(a[0].w)\n";

fn codes(src: &str, config: &Config) -> Vec<&'static str> {
    let mut out: Vec<_> = lint(src, config).iter().map(|d| d.code.0).collect();
    out.extend(checker_lints(None, src, config).iter().map(|d| d.code.0));
    out
}

#[test]
fn checker_lints_match_the_spec_table() {
    let doc = include_str!("../../../spec/lints.md");
    let mut spec = Vec::new();
    for row in doc.lines().filter(|l| l.starts_with("| L06")) {
        let cells: Vec<&str> = row.split('|').map(str::trim).collect();
        for name in cells[2].split('/') {
            spec.push((cells[1].to_string(), name.trim().to_string()));
        }
    }
    let mut ours: Vec<_> = CHECKER_LINTS
        .iter()
        .map(|l| (l.code.0.to_string(), l.name.to_string()))
        .collect();
    ours.sort();
    spec.sort();
    assert_eq!(ours, spec);
}

#[test]
fn every_checker_lint_has_a_registry_entry_for_cpl_explain() {
    for lint in crate::CHECKER_LINTS {
        assert!(
            cpl_diag::lookup(lint.code.0).is_some(),
            "{} {}",
            lint.code.0,
            lint.name
        );
    }
}

#[test]
fn cpl_lint_reports_a_checker_lint() {
    assert_eq!(codes(PADDED, &Config::default()), ["L0607"]);
}

#[test]
fn with_lint_allows_a_checker_lint_by_name() {
    let src = format!("with lint(padding: allow)\n{PADDED}");
    assert_eq!(codes(&src, &Config::default()), Vec::<&str>::new());
}

#[test]
fn cp_toml_sets_a_checker_lint_level() {
    let (config, errors) = Config::from_toml("[lints]\npadding = \"deny\"\n");
    assert!(errors.is_empty(), "{errors:?}");
    let found = checker_lints(None, PADDED, &config);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].severity, Severity::Error);
    let (config, _) = Config::from_toml("[lints]\nmissing_parens = \"allow\"\n");
    let src = "let a = [3, 1, 2]\nprint(a.sorted.len)\n";
    assert!(checker_lints(None, src, &Config::default())
        .iter()
        .any(|d| d.code.0 == "L0608"));
    assert!(checker_lints(None, src, &config).is_empty());
}

#[test]
fn a_misspelt_checker_lint_suggests_its_name() {
    let (_, errors) = Config::from_toml("[lints]\npaddng = \"allow\"\n");
    assert!(
        errors[0].message.contains("did you mean `padding`?"),
        "{errors:?}"
    );
}

#[test]
fn a_file_that_does_not_check_is_not_reported_clean() {
    let dir = std::env::temp_dir().join(format!("cpl-lint-unchecked-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("bad.cp");
    std::fs::write(&file, "let x: Int = \"a\"\nprint(x)\n").unwrap();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = crate::cli::run(&[file.display().to_string()], &mut out, &mut err);
    let err = String::from_utf8(err).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(code, 1, "{err}");
    let out = String::from_utf8(out).unwrap();
    assert!(
        out.contains("error[E0302]"),
        "the type error itself is shown: {out}"
    );
    assert!(
        err.contains("has 1 type error lints cannot see past"),
        "{err}"
    );
    assert!(
        err.ends_with("1 file checked, 0 warnings, 1 error, 1 not checking\n"),
        "{err}"
    );
}

/// Every checker lint has a record in `spec/checker-lints/<name>/` for its
/// `/lints/<name>` page: `before.cp` fires it, `after.cp` checks clean and
/// fires nothing, `near_miss.cp` checks clean and does not fire it.
#[test]
fn every_checker_lint_has_a_page_record_that_fires_on_before_only() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/checker-lints");
    let config = Config::default();
    for l in CHECKER_LINTS {
        let read = |f: &str| {
            std::fs::read_to_string(root.join(l.name).join(f))
                .unwrap_or_else(|e| panic!("spec/checker-lints/{}/{f}: {e}", l.name))
        };
        assert!(
            read("lint.toml").contains(&format!("code = \"{}\"", l.code.0)),
            "{}: lint.toml names another code",
            l.name
        );
        assert!(
            !read("explain.md").trim().is_empty(),
            "{}: no explain.md",
            l.name
        );
        let before = crate::checked(None, &read("before.cp"), &config);
        if l.name == "parens_on_o1" {
            // `xs.len()` is error E0304 (decision 140); the record is its page.
            assert_eq!(before.errors, 1, "parens_on_o1: before.cp is not one error");
        } else {
            assert!(
                before.lints.iter().any(|d| d.code == l.code),
                "{}: before.cp does not fire",
                l.name
            );
        }
        let src = read("after.cp");
        let after = crate::checked(None, &src, &config);
        assert!(
            after.errors == 0 && after.lints.is_empty() && lint(&src, &config).is_empty(),
            "{}: after.cp does not check clean",
            l.name
        );
        let near = crate::checked(None, &read("near_miss.cp"), &config);
        assert!(
            near.errors == 0 && !near.lints.iter().any(|d| d.code == l.code),
            "{}: near_miss.cp fires or does not check",
            l.name
        );
    }
}

#[test]
fn a_lint_does_not_repeat_the_error_at_its_place() {
    let span = cpl_diag::Span::new(3, 9);
    let error = Diagnostic::error(cpl_diag::Code("E0304"), span, "read without `()`");
    let mut fixed_error = error.clone();
    fixed_error.fix("drop `()`", span, "");
    let mut found = Diagnostic::warning(cpl_diag::Code("L0509"), span, "same root cause");
    let mut diags = vec![found.clone()];
    crate::drop_repeats(&mut diags, std::slice::from_ref(&error));
    assert!(diags.is_empty(), "{diags:?}");
    found.fixes.push(cpl_diag::Fix {
        title: "fix".into(),
        edits: vec![(span, String::new())],
    });
    let mut diags = vec![found];
    crate::drop_repeats(&mut diags, std::slice::from_ref(&error));
    assert_eq!(diags.len(), 1, "a lint keeps the quick fix the error lacks");
    crate::drop_repeats(&mut diags, &[fixed_error]);
    assert!(diags.is_empty(), "{diags:?}");
}

#[test]
fn a_getter_called_with_parens_is_the_checker_s_with_its_fix() {
    let src = "let ys = [1, 2]\nprint(ys.first())\n";
    let mut db = cpl_check::Database::new();
    db.set_file("probe.cp", src);
    let analysis = db.analyze("probe.cp").unwrap();
    let error = analysis
        .diagnostics
        .iter()
        .find(|d| d.code.0 == "E0304")
        .unwrap();
    assert!(!error.fixes.is_empty(), "{error:?}");
    let lints = crate::lint_analysis(&analysis, &crate::Config::default());
    assert!(lints.iter().all(|d| d.span != error.span), "{lints:?}");
}
