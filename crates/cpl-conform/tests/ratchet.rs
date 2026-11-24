use std::collections::BTreeMap;

use cpl_conform::conformance::{parse_header, Expect};
use cpl_conform::scoreboard::{check_prev, check_ratchet, CaseOutcome, Scoreboard, SuiteReport};
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
    let exp = parse_header("// expect: ok\n// rule: R-01\nlet x = 1\n").unwrap();
    assert_eq!(exp, Expect::Ok);

    let exp = parse_header("// expect: ok, output \"hello\\n\"\nprint(\"hello\")\n").unwrap();
    assert_eq!(exp, Expect::OkOutput("hello\n".into()));

    let exp =
        parse_header("// expect: E0601\n// rule: 05.1\nlet a = 1\nf(a) // <- here\n").unwrap();
    assert_eq!(
        exp,
        Expect::Diag {
            code: "E0601".into(),
            line: 4,
        }
    );

    let exp = parse_header("// expect: R0301 (checked)\nlet a = [1]\nprint(a[3])   // <- here  \n")
        .unwrap();
    assert_eq!(
        exp,
        Expect::Trap {
            code: "R0301".into(),
            line: 3,
        }
    );

    let exp = parse_header("// expect: square_by_hand\nx * x // <- here\n").unwrap();
    assert_eq!(
        exp,
        Expect::Diag {
            code: "square_by_hand".into(),
            line: 2,
        }
    );

    assert!(parse_header("let x = 1\n").is_err());
}

#[test]
fn header_parser_rejects_line_counts_and_stray_markers() {
    let why = parse_header("// expect: E0601 at 4\nlet a = 1\n").unwrap_err();
    assert!(why.contains("`at N` is gone"), "{why}");
    let why = parse_header("// expect: E0601\nlet a = 1\n").unwrap_err();
    assert!(why.contains("needs the line it points at"), "{why}");
    let why = parse_header("// expect: E0601\na // <- here\nb // <- here\n").unwrap_err();
    assert!(why.contains("mark only"), "{why}");
    let why = parse_header("// expect: ok\nlet a = 1 // <- here\n").unwrap_err();
    assert!(why.contains("names no diagnostic"), "{why}");
    assert!(parse_header("// expect: E0601 (bad tag!)\na // <- here\n").is_err());
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
    assert!(matches!(&blocks[0].1, Marker::Invalid(why) if why.contains("spec-only")));
    assert_eq!(
        blocks[1].1,
        Marker::Given("let n = 3\nlet a = [1, 2]".into())
    );
    assert_eq!(blocks[2].1, Marker::None);
    let md = "<!-- rejects: E0608 -->\n```cp\nf()\n```\n";
    assert_eq!(
        extract_marked_blocks(md)[0].1,
        Marker::Rejects("E0608".into())
    );
    let md = "1. item\n\n   <!-- spec-only: listed -->\n   ```cp\n   let q = 1\n     q\n   ```\n";
    let blocks = extract_marked_blocks(md);
    assert_eq!(blocks[0].0, "let q = 1\n  q\n");
    assert!(matches!(&blocks[0].1, Marker::Invalid(_)));
    let md = "<!-- given: let a = [1] -->\n<!-- rejects: E0603 -->\n```cp\nf(a)\n```\n";
    assert_eq!(
        extract_marked_blocks(md)[0].1,
        Marker::GivenRejects("let a = [1]".into(), "E0603".into())
    );
}

#[test]
fn spec_blocks_std_markers() {
    use cpl_conform::spec_blocks::{
        excerpt_unmet, extract_marked_blocks, signatures_unmet, Marker,
    };
    let md = "<!-- signatures: std/heap.cp std/bag.cp -->\n```cp\nfn f()\n```\n<!-- excerpt: std/io.cp -->\n```cp\nx\n```\n";
    let blocks = extract_marked_blocks(md);
    assert_eq!(
        blocks[0].1,
        Marker::Signatures("std/heap.cp std/bag.cp".into())
    );
    assert_eq!(blocks[1].1, Marker::Excerpt("std/io.cp".into()));
    let std = "/// doc\npub fn scan<D, T>(\n    self: [D]T,\n    from: ?T = None,\n) -> Prefix<D, T> =\n    todo\npub struct Pt(pub x: Int, y: Int)\n";
    assert_eq!(
        signatures_unmet(
            "fn scan<D, T>(self: [D]T, from: ?T = None) -> Prefix<D, T>\nstruct Pt(x: Int, y: Int)",
            std
        ),
        None
    );
    let why = signatures_unmet("fn scan<D>(self: [D]Int) -> Int", std).unwrap();
    assert!(why.contains("std has `fn scan<D, T>("), "{why}");
    assert!(signatures_unmet("// nothing", std).is_some());
    assert_eq!(
        excerpt_unmet("pub struct Pt(pub x: Int, y: Int)  // a point", std),
        None
    );
    assert!(excerpt_unmet("pub struct Pt(pub x: Int, y: Int)\npub fn scan<D, T>(", std).is_some());
}

#[test]
fn spec_blocks_fences_are_closed() {
    use cpl_conform::spec_blocks::{extract_marked_blocks, Marker};
    for md in [
        "```\nx\n```\n",
        "```rust\nx\n```\n",
        "<!-- given: let a = 1 -->\n```cpp\nx\n```\n",
    ] {
        let blocks = extract_marked_blocks(md);
        assert!(
            matches!(&blocks[0].1, Marker::Invalid(_)),
            "{md}: {:?}",
            blocks[0].1
        );
    }
    assert!(extract_marked_blocks("```cpp\nint x;\n```\n").is_empty());
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

#[test]
fn an_edited_doc_example_is_not_a_regression_but_a_lost_one_is() {
    let base = make_board("docs", &[("docs/a.cp#1", "pass"), ("docs/a.cp#2", "pass")]);
    let edited = make_board("docs", &[("docs/a.cp#1", "pass"), ("docs/a.cp#3", "pass")]);
    assert!(check_ratchet(&base, &edited, false).regressions.is_empty());
    let lost = make_board("docs", &[("docs/a.cp#1", "pass"), ("docs/b.cp#9", "pass")]);
    let r = check_ratchet(&base, &lost, false);
    assert_eq!(
        r.regressions,
        vec!["docs/docs/a.cp#2: previously passing case disappeared"]
    );
}

#[test]
fn docs_case_ids_survive_an_edited_example() {
    use cpl_conform::docs::example_ids;
    let before = "\
domain Dir4 = 0..4:
    /// example: `Dir4.up().delta == (-1, 0)`
    pub fn up() -> Dir4 = Dir4(0)
    /// example: `Dir4.left.delta == (0, -1)`
    /// example: `Dir4.left.turn == Dir4.down`
    @inline
    pub fn left() -> Dir4 = Dir4(3)
/// example: `(1, 1).up`
pub fn up<R, C>(self: (R, C)) -> ?(R, C) = None
";
    let after = before.replace("Dir4.up().delta", "Dir4.up.delta");
    let ids = |text: &str| -> Vec<String> {
        example_ids("grid.cp", text)
            .into_iter()
            .map(|(id, _)| id)
            .collect()
    };
    assert_eq!(
        ids(before),
        [
            "grid.cp#Dir4.up:1",
            "grid.cp#Dir4.left:1",
            "grid.cp#Dir4.left:2",
            "grid.cp#up:1",
        ]
    );
    assert_eq!(ids(before), ids(&after), "editing an example keeps its id");

    let base = make_board("docs", &[("grid.cp#Dir4.up:1", "pass")]);
    let edited_ok = make_board("docs", &[("grid.cp#Dir4.up:1", "pass")]);
    assert!(check_ratchet(&base, &edited_ok, false)
        .regressions
        .is_empty());
    let edited_broken = make_board("docs", &[("grid.cp#Dir4.up:1", "fail")]);
    let report = check_ratchet(&base, &edited_broken, false);
    assert!(
        report
            .regressions
            .iter()
            .any(|r| r.contains("grid.cp#Dir4.up:1: regressed from pass to fail")),
        "an edited example that breaks is a regression of the same case, got {:?}",
        report.regressions
    );
}

#[test]
fn std_docs_case_ids_are_unique() {
    let root = find_repo_root().expect("repo root");
    let examples = cpl_conform::docs::discover(&root, None);
    let mut seen = std::collections::BTreeSet::new();
    for ex in &examples {
        assert!(
            seen.insert(ex.id.clone()),
            "duplicate docs case id {}",
            ex.id
        );
    }
}

#[test]
fn docs_fenced_blocks_are_examples_of_their_item() {
    use cpl_conform::docs::doc_examples;
    let text = "\
struct Rooted<V>(root: V):
    /// The DP for every root.
    ///
    /// ```cp
    /// let sums = t.reroot(
    ///     +,
    /// )
    /// sums == [1]
    /// ```
    ///
    /// example: `1 == 1`
    pub fn reroot(self) -> Int = 0
";
    assert_eq!(
        doc_examples("tree.cp", text),
        [
            (
                "tree.cp#Rooted.reroot:1".to_string(),
                "let sums = t.reroot(\n    +,\n)\nsums == [1]".to_string(),
                true
            ),
            (
                "tree.cp#Rooted.reroot:2".to_string(),
                "`1 == 1`".to_string(),
                false
            ),
        ]
    );
}

#[test]
fn spec_block_ids_survive_an_edited_block() {
    use cpl_conform::spec_blocks::block_ids;
    let md = "# 4 Keys\n```cp\nprint(1)\n```\n## 4.2 Slices\n```cp\nprint(2)\n```\n\n```cp\nprint(3)\n```\n## Open items\n```cp\n# not a heading\n```\n";
    let ids: Vec<String> = block_ids("core/a.md", md)
        .into_iter()
        .map(|b| b.0)
        .collect();
    assert_eq!(
        ids,
        [
            "core/a.md#4:1",
            "core/a.md#4.2:1",
            "core/a.md#4.2:2",
            "core/a.md#open-items:1"
        ]
    );
    let edited = md.replace("print(2)", "print(20)\nprint(21)");
    let again: Vec<String> = block_ids("core/a.md", &edited)
        .into_iter()
        .map(|b| b.0)
        .collect();
    assert_eq!(ids, again);
}

#[test]
fn handbook_blocks_are_cp_only() {
    use cpl_conform::spec_blocks::block_ids;
    let md = "# Why\n```python\nb = a\n```\n```cp\nprint(1)\n```\n";
    let ids: Vec<String> = block_ids("handbook/language/a.md", md)
        .into_iter()
        .map(|b| b.0)
        .collect();
    assert_eq!(ids, ["handbook/language/a.md#why:1"]);
    assert_eq!(block_ids("core/a.md", md).len(), 2);
}

// 331aa3f44 broke 10 lambda-graph cases the committed scoreboard didn't have
// yet, and `--check` still said 0 regressions.
#[test]
fn a_case_newer_than_the_scoreboard_cannot_regress_unseen() {
    let committed = make_board("conformance", &[("old.cp", "pass"), ("gap.cp", "fail")]);
    let prev = make_board(
        "conformance",
        &[
            ("old.cp", "pass"),
            ("gap.cp", "pass"),
            ("new.cp", "pass"),
            ("gone.cp", "pass"),
        ],
    );
    let current = make_board(
        "conformance",
        &[("old.cp", "fail"), ("gap.cp", "fail"), ("new.cp", "error")],
    );
    assert_eq!(
        check_prev(&committed, &prev, &current),
        vec!["conformance/new.cp: regressed from pass on main's last green run to error"]
    );
    let fixed = make_board("conformance", &[("new.cp", "pass")]);
    assert!(check_prev(&committed, &prev, &fixed).is_empty());
}

#[test]
fn docs_example_tags_name_how_an_example_stops() {
    use cpl_conform::docs::parse_example_body;
    let code = |s: &str| Some(s.to_string());
    assert_eq!(
        parse_example_body("`a[5]` (traps R0101)"),
        (code("a[5]"), String::new(), code("R0101"))
    );
    assert_eq!(
        parse_example_body("`read()` (stdin `3`) (error E0302)"),
        (code("read()"), "3\n".to_string(), code("E0302"))
    );
    assert_eq!(
        parse_example_body("`x` and then some prose"),
        (None, String::new(), None),
        "prose after the code is not an example"
    );
}

#[test]
fn docs_examples_run_or_stop_as_tagged() {
    use cpl_conform::docs::{run_one, DocExample};
    let root = find_repo_root().expect("repo root");
    let mut db = cpl_check::Session::new();
    let ex = |snippet: Option<&str>, expect: Option<&str>| DocExample {
        id: "t.cp#x:1".into(),
        module: String::new(),
        snippet: snippet.map(str::to_string),
        stdin: String::new(),
        block: false,
        expect: expect.map(str::to_string),
    };
    let status = |db: &mut cpl_check::Session, e: DocExample| run_one(db, &root, &e).status;
    assert_eq!(status(&mut db, ex(Some("1 + 1 == 2"), None)), "pass");
    assert_eq!(
        status(&mut db, ex(Some("[1, 2][5] == 0"), Some("R0101"))),
        "pass"
    );
    assert_eq!(status(&mut db, ex(Some("[1, 2][5] == 0"), None)), "trap");
    assert_eq!(status(&mut db, ex(Some("1 == 1"), Some("R0101"))), "fail");
    assert_eq!(status(&mut db, ex(Some("1 == 1"), Some("R0402"))), "bad");
    assert_eq!(status(&mut db, ex(None, None)), "prose");
}

#[test]
fn snapshot_outlives_a_relinked_binary() {
    let root = std::env::temp_dir().join(format!("cpl-conform-snapshot-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let bin = root.join("cpl");
    std::fs::write(&bin, "old").unwrap();
    let held = cpl_conform::conformance::snapshot(&bin, &root.join("held")).unwrap();
    std::fs::remove_file(&bin).unwrap();
    std::fs::write(&bin, "new").unwrap();
    assert_eq!(held.file_name().unwrap(), "cpl");
    assert_eq!(std::fs::read_to_string(&held).unwrap(), "old");
    std::fs::remove_dir_all(&root).unwrap();
}
