use crate::{checker_lints, lint, Config, CHECKER_LINTS};
use cpl_diag::Severity;

const PADDED: &str =
    "let a: [200000](w: Int, b: Bool) = [200000].filled((w: 0, b: false))\nprint(a[0].w)\n";

fn codes(src: &str, config: &Config) -> Vec<&'static str> {
    let mut out: Vec<_> = lint(src, config).iter().map(|d| d.code.0).collect();
    out.extend(checker_lints(None, src, config).iter().map(|d| d.code.0));
    out
}

#[test]
fn checker_lints_match_the_spec_table() {
    let doc = include_str!("../../../spec/diagnostics.md");
    let mut spec = Vec::new();
    for row in doc.lines().filter(|l| l.starts_with("| L06")) {
        let cells: Vec<&str> = row.split('|').map(str::trim).collect();
        for name in cells[2].split('/') {
            spec.push((cells[1].to_string(), name.trim().to_string()));
        }
    }
    let ours: Vec<_> = CHECKER_LINTS
        .iter()
        .map(|l| (l.code.0.to_string(), l.name.to_string()))
        .collect();
    assert_eq!(ours, spec);
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
