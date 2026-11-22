use std::collections::BTreeMap;

use cpl_conform::conformance::{Expect, parse_header};
use cpl_conform::scoreboard::{CaseOutcome, Scoreboard, SuiteReport, check_ratchet};
use cpl_conform::{Suite, default_scoreboard_path, find_repo_root, run_suites};

fn make_board(suite_name: &str, cases: &[(&str, &str)]) -> Scoreboard {
    let mut map = BTreeMap::new();
    for &(id, status) in cases {
        map.insert(
            id.to_string(),
            CaseOutcome {
                status: status.to_string(),
                detail: String::new(),
            },
        );
    }
    let mut sb = Scoreboard::default();
    sb.suites
        .insert(suite_name.to_string(), SuiteReport::from_outcomes(map));
    sb
}

#[test]
fn header_parser_handles_all_expect_forms() {
    let (exp, off) = parse_header("// expect: ok\n// rule: R-01\nlet x = 1\n").unwrap();
    assert_eq!(exp, Expect::Ok);
    assert_eq!(off, 2);

    let (exp, off) =
        parse_header("// expect: ok, output \"hello\\n\"\nprint(\"hello\")\n").unwrap();
    assert_eq!(exp, Expect::OkOutput("hello\n".into()));
    assert_eq!(off, 1);

    let (exp, _) = parse_header("// expect: E0601 at 4\n").unwrap();
    assert_eq!(
        exp,
        Expect::Diag {
            code: "E0601".into(),
            line: 4,
        }
    );

    let (exp, _) = parse_header("// expect: R0301 at 2 (checked)\n").unwrap();
    assert_eq!(
        exp,
        Expect::Trap {
            code: "R0301".into(),
            line: 2,
        }
    );

    assert!(parse_header("let x = 1\n").is_err());
}

#[test]
fn scoreboard_json_roundtrip() {
    let board = make_board(
        "conformance",
        &[("a.cp", "pass"), ("b.cp", "fail"), ("c.cp", "pass")],
    );
    let rendered = board.to_json_string();
    let parsed = Scoreboard::from_json_str(&rendered).expect("valid scoreboard JSON");
    assert_eq!(parsed.suites["conformance"].total, 3);
    assert_eq!(parsed.suites["conformance"].passed, 2);
    assert_eq!(parsed.suites["conformance"].cases["a.cp"], "pass");
    assert_eq!(parsed.suites["conformance"].cases["b.cp"], "fail");
}

#[test]
fn ratchet_fails_on_planted_regression() {
    let baseline = make_board(
        "conformance",
        &[("a.cp", "pass"), ("b.cp", "pass"), ("c.cp", "fail")],
    );

    let regressed = make_board(
        "conformance",
        &[("a.cp", "pass"), ("b.cp", "fail"), ("c.cp", "fail")],
    );
    let report = check_ratchet(&baseline, &regressed, false);
    assert!(
        !report.regressions.is_empty(),
        "ratchet must fail when a passing case regresses"
    );
    assert!(
        report.regressions.iter().any(|r| r.contains("b.cp")),
        "expected regression naming b.cp, got {:?}",
        report.regressions
    );
    assert!(
        report
            .regressions
            .iter()
            .any(|r| r.contains("pass count dropped from 2 to 1")),
        "expected pass-count drop, got {:?}",
        report.regressions
    );

    let swapped = make_board(
        "conformance",
        &[("a.cp", "pass"), ("b.cp", "fail"), ("c.cp", "pass")],
    );
    let swap_report = check_ratchet(&baseline, &swapped, false);
    assert!(
        !swap_report.regressions.is_empty(),
        "ratchet must fail when a previously passing case fails even if total pass count holds"
    );
    assert!(swap_report.regressions.iter().any(|r| r.contains("b.cp")));

    let improved = make_board(
        "conformance",
        &[("a.cp", "pass"), ("b.cp", "pass"), ("c.cp", "pass")],
    );
    let imp_report = check_ratchet(&baseline, &improved, false);
    assert!(
        imp_report.regressions.is_empty(),
        "improvements must pass the ratchet"
    );
    assert_eq!(imp_report.improvements.len(), 1);
}

#[test]
fn ratchet_respects_filtered_runs() {
    let baseline = make_board(
        "conformance",
        &[("a.cp", "pass"), ("b.cp", "pass"), ("c.cp", "fail")],
    );
    let filtered_ok = make_board("conformance", &[("a.cp", "pass")]);
    assert!(
        check_ratchet(&baseline, &filtered_ok, true)
            .regressions
            .is_empty()
    );

    let filtered_bad = make_board("conformance", &[("a.cp", "fail")]);
    assert!(
        !check_ratchet(&baseline, &filtered_bad, true)
            .regressions
            .is_empty()
    );
}

#[test]
fn committed_scoreboard_ratchet() {
    let root = find_repo_root().expect("repo root");
    let path = default_scoreboard_path(&root);
    let baseline = Scoreboard::load(&path)
        .unwrap_or_else(|e| panic!("failed to read baseline {}: {e}", path.display()));
    let current = run_suites(&root, Suite::FAST, None);
    let report = check_ratchet(&baseline, &current, false);
    if !report.regressions.is_empty() {
        panic!(
            "cpl-conform ratchet failed against {}:\n{}",
            path.display(),
            report.regressions.join("\n")
        );
    }
}
