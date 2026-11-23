use std::collections::BTreeMap;

use cpl_conform::conformance::{parse_header, Expect};
use cpl_conform::scoreboard::{check_ratchet, CaseOutcome, Scoreboard, SuiteReport};
use cpl_conform::{default_scoreboard_path, find_repo_root, run_suites, Suite};

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
fn causes_grouping_and_roundtrip() {
    let mut map = BTreeMap::new();
    map.insert(
        "a.cp".into(),
        CaseOutcome {
            status: "fail".into(),
            detail: "wanted ok; got: error[E0302]: expected `Int`, found `Bool`".into(),
        },
    );
    map.insert(
        "b.cp".into(),
        CaseOutcome {
            status: "fail".into(),
            detail: "wanted ok; got: error[E0302]: expected `Str`, found `Int`".into(),
        },
    );
    map.insert(
        "c.cp".into(),
        CaseOutcome {
            status: "fail".into(),
            detail: "wanted ok; got: error[R0402]: `todo` reached".into(),
        },
    );
    map.insert(
        "d.cp".into(),
        CaseOutcome {
            status: "pass".into(),
            detail: String::new(),
        },
    );
    let mut sb = Scoreboard::default();
    sb.suites
        .insert("conformance".into(), SuiteReport::from_outcomes(map));
    let causes = &sb.suites["conformance"].causes;
    assert_eq!(causes.len(), 2);
    assert_eq!(causes[0].cause, "E0302: expected `<id>`, found `<id>`");
    assert_eq!(causes[0].count, 2);
    assert_eq!(causes[1].cause, "R0402: `todo` reached");
    assert_eq!(causes[1].count, 1);

    let parsed = Scoreboard::from_json_str(&sb.to_json_string()).unwrap();
    assert_eq!(parsed.suites["conformance"].causes, *causes);
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
    assert!(check_ratchet(&baseline, &filtered_ok, true)
        .regressions
        .is_empty());

    let filtered_bad = make_board("conformance", &[("a.cp", "fail")]);
    assert!(!check_ratchet(&baseline, &filtered_bad, true)
        .regressions
        .is_empty());
}

#[test]
fn spec_blocks_extract_and_strict_roundtrip() {
    let md = "intro\n```cp\nlet x = 1\n```\n```text\nignored\n```\n```python\nprint(2)\n```\n";
    let blocks = cpl_conform::spec_blocks::extract_blocks(md);
    assert_eq!(blocks, vec!["let x = 1\n", "print(2)\n"]);

    let mut sb = make_board("spec-blocks", &[("core/a.md#0123456789", "pass")]);
    sb.suites.get_mut("spec-blocks").unwrap().strict_passed = Some(1);
    let parsed = Scoreboard::from_json_str(&sb.to_json_string()).expect("roundtrip");
    assert_eq!(parsed.suites["spec-blocks"].strict_passed, Some(1));
}

#[test]
fn spec_blocks_markers_above_a_fence() {
    use cpl_conform::spec_blocks::{extract_marked_blocks, Marker};
    let md = "<!-- spec-only: a tour -->\n```cp\nf(...)\n```\n\n<!-- given:\n  let n = 3\n  let a = [1, 2] -->\n```cp\nprint(n)\n```\n<!-- spec-only: not adjacent -->\n\n```cp\nprint(1)\n```\n";
    let blocks = extract_marked_blocks(md);
    assert_eq!(blocks[0].1, Marker::SpecOnly("a tour".into()));
    assert_eq!(
        blocks[1].1,
        Marker::Given("let n = 3\nlet a = [1, 2]".into())
    );
    assert_eq!(blocks[2].1, Marker::None);
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
